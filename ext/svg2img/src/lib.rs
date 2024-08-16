use anyhow::Context;
use image::{DynamicImage, GenericImageView, ImageBuffer};
use magnus::{function, prelude::*, Error, Ruby, Symbol};
use std::{
    fmt::Display,
    panic::{self, AssertUnwindSafe},
};

#[magnus::init]
fn init(ruby: &Ruby) -> Result<(), Error> {
    let module = ruby.define_module("Svg2Img")?;
    module.define_singleton_method("process_svg", function!(process_svg_rb, 2))?;
    Ok(())
}

fn process_svg_rb(svg: String, options: magnus::RHash) -> Result<String, magnus::Error> {
    let mut output_format = image::ImageFormat::Png;

    let output_format_option = get_option::<Symbol>(&options, "output_format").or_else(|_| {
        get_option::<String>(&options, "output_format")
            .map(|format| format.map(|format| Symbol::new(format.as_str())))
    });

    if let Some(format) = output_format_option? {
        output_format = match unsafe { format.to_s()?.into_owned() }.as_ref() {
            "png" => image::ImageFormat::Png,
            "jpeg" | "jpg" => image::ImageFormat::Jpeg,
            "gif" => image::ImageFormat::Gif,
            "webp" => image::ImageFormat::WebP,
            format => {
                return Err(magnus::Error::new(
                    magnus::exception::arg_error(),
                    format!("svg2img: Invalid output format: {format}"),
                ));
            }
        };
    }
    let options = Options {
        max_width: get_option(&options, "max_width")?,
        max_height: get_option(&options, "max_height")?,
        output_format,
        output_path: get_option(&options, "output_path")?,
    };
    process_svg(svg, options)
        .map_err(|err| magnus::Error::new(magnus::exception::runtime_error(), format!("{err:?}")))
}

fn get_option<T>(options: &magnus::RHash, key: &str) -> Result<Option<T>, magnus::Error>
where
    T: magnus::TryConvert,
{
    fn get_option_inner<K, T>(
        options: &magnus::RHash,
        key: K,
        key_name: &str,
    ) -> Result<Option<T>, magnus::Error>
    where
        K: magnus::IntoValue + Display + Clone,
        T: magnus::TryConvert,
    {
        let option_exists = options.get(key.clone()).is_some();
        if !option_exists {
            return Ok(None);
        }
        match options.fetch::<_, T>(key) {
            Ok(value) => Ok(Some(value)),
            Err(err) => Err(magnus::Error::new(
                magnus::exception::arg_error(),
                format!("svg2img: Invalid option {key_name}: {err:?}"),
            )),
        }
    }
    let symbol_result = get_option_inner(options, magnus::Symbol::new(key), key)?;
    if symbol_result.is_some() {
        return Ok(symbol_result);
    }
    get_option_inner(options, key, key)
}

struct Options {
    max_width: Option<u32>,
    max_height: Option<u32>,
    output_format: image::ImageFormat,
    output_path: Option<String>,
}

fn process_svg(svg: String, options: Options) -> Result<String, anyhow::Error> {
    let image = image_from_svg(svg.as_bytes())?;
    let (old_width, old_height) = image.dimensions();

    let (mut width, mut height) = (old_width, old_height);
    if let Some(max_width) = options.max_width {
        if width > max_width {
            height = height * max_width / width;
            width = max_width;
        }
    }
    if let Some(max_height) = options.max_height {
        if height > max_height {
            width = width * max_height / height;
            height = max_height;
        }
    }

    let mut image = image.resize_exact(width, height, image::imageops::FilterType::Lanczos3);

    if options.output_format == image::ImageFormat::Jpeg {
        // Convert from rgba8 to rgb8
        image = DynamicImage::ImageRgb8(image.into_rgb8())
    }

    let mut buf = Vec::new();
    let mut cursor = std::io::Cursor::new(&mut buf);
    image
        .write_to(&mut cursor, options.output_format)
        .context("Failed to write image to buffer")?;

    let output_path = options.output_path.unwrap_or_else(|| {
        let random_filename = format!(
            "svg2img-{}.{}",
            uuid::Uuid::new_v4(),
            options
                .output_format
                .extensions_str()
                .first()
                .unwrap_or(&"whatever")
        );
        std::env::temp_dir()
            .join(random_filename)
            .to_string_lossy()
            .to_string()
    });
    std::fs::write(&output_path, buf).context("Failed to write image to file")?;

    Ok(output_path)
}

fn image_from_svg(bytes: &[u8]) -> Result<DynamicImage, anyhow::Error> {
    let tree = resvg::usvg::Tree::from_data(bytes, &resvg::usvg::Options::default())
        .context("Failed to parse SVG")?;

    const TARGET_SIZE: u32 = 512;
    let mut pixmap = resvg::tiny_skia::Pixmap::new(TARGET_SIZE, TARGET_SIZE)
        .context("Failed to create Pixmap for SVG rendering")?;
    let ratio = tree.size().width() / tree.size().height();
    let scaled_width = if ratio > 1.0 {
        TARGET_SIZE
    } else {
        (TARGET_SIZE as f32 * ratio).round() as u32
    };
    let scaled_height = if ratio > 1.0 {
        (TARGET_SIZE as f32 / ratio).round() as u32
    } else {
        TARGET_SIZE
    };

    // Scale svg and place it centered
    let transform = resvg::tiny_skia::Transform {
        sx: scaled_width as f32 / tree.size().width(),
        sy: scaled_height as f32 / tree.size().height(),
        tx: (TARGET_SIZE - scaled_width) as f32 / 2.0,
        ty: (TARGET_SIZE - scaled_height) as f32 / 2.0,
        ..Default::default()
    };

    let result = panic::catch_unwind(AssertUnwindSafe(|| {
        resvg::render(&tree, transform, &mut pixmap.as_mut());
    }));
    if let Err(panic) = result {
        let panic_message = panic
            .downcast_ref::<String>()
            .map(String::as_str)
            .or_else(|| {
                panic
                    .downcast_ref::<&'static str>()
                    .map(std::ops::Deref::deref)
            })
            .unwrap_or("Box<Any>");
        return Err(anyhow::anyhow!("SVG rendering panicked: {}", panic_message));
    }

    rgba_to_image(pixmap.width(), pixmap.height(), pixmap.data())
}

fn rgba_to_image(width: u32, height: u32, data: &[u8]) -> Result<DynamicImage, anyhow::Error> {
    let mut image_data = Vec::with_capacity((width * height * 4) as usize);
    for pixel in data.chunks(4) {
        image_data.push(pixel[0]); // R
        image_data.push(pixel[1]); // G
        image_data.push(pixel[2]); // B
        image_data.push(pixel[3]); // A
    }
    let buffer = ImageBuffer::from_raw(width, height, image_data)
        .context("Failed to convert to ImageBuffer")?;
    Ok(DynamicImage::ImageRgba8(buffer))
}
