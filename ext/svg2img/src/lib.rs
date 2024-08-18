use anyhow::Context;
use image::{DynamicImage, ImageBuffer, ImageEncoder};
use magnus::{block::Proc, function, prelude::*, Error, Ruby};
use std::panic::{self, AssertUnwindSafe};

#[magnus::init]
fn init(ruby: &Ruby) -> Result<(), Error> {
    let module = ruby.define_module("Svg2Img")?;
    module.define_singleton_method("process_svg", function!(process_svg_rb, 2))?;
    Ok(())
}

fn process_svg_rb(svg: String, options: magnus::RHash) -> Result<String, magnus::Error> {
    let mut format = image::ImageFormat::Png;

    if let Some(format_option) = get_string_option(&options, "format")? {
        format = match format_option.as_str() {
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

    let super_sampling = get_option::<u32>(&options, "super_sampling")?.unwrap_or(2);
    // super_sampling must me a power of 2
    if super_sampling == 0 || super_sampling & (super_sampling - 1) != 0 {
        return Err(magnus::Error::new(
            magnus::exception::arg_error(),
            "svg2img: Invalid super_sampling value, must be a power of 2",
        ));
    }

    let options = Options {
        size: get_option::<Proc>(&options, "size")?
            .map(convert_size_proc)
            .unwrap_or_else(default_size),
        output_path: get_option(&options, "output_path")?,
        format,
        super_sampling,
    };
    process_svg(svg, options)
        .map_err(|err| magnus::Error::new(magnus::exception::runtime_error(), format!("{err:?}")))
}

fn get_option<T>(options: &magnus::RHash, key: &str) -> Result<Option<T>, magnus::Error>
where
    T: magnus::TryConvert,
{
    let value = options
        .get(magnus::Symbol::new(key))
        .or_else(|| options.get(key));
    let Some(value) = value else {
        return Ok(None);
    };
    match T::try_convert(value) {
        Ok(value) => Ok(Some(value)),
        Err(err) => Err(magnus::Error::new(
            magnus::exception::arg_error(),
            format!("svg2img: Invalid option {key}: {err:?}"),
        )),
    }
}
fn get_string_option(options: &magnus::RHash, key: &str) -> Result<Option<String>, magnus::Error> {
    // Try get_option with String, then try again with Symbol
    let string_option = get_option::<String>(options, key);
    if let Ok(Some(string_option)) = string_option {
        return Ok(Some(string_option));
    }

    let symbol_option = get_option::<magnus::Symbol>(options, key)?;
    if let Some(symbol) = symbol_option {
        let symbol_string = unsafe {
            symbol.to_s().map_err(|err| magnus::Error::new(
            magnus::exception::arg_error(),
            format!("svg2img: Invalid option {key} (could not convert from Symbol to string): {err:?}"),
        ))?.into_owned()
        };
        return Ok(Some(symbol_string));
    }

    Ok(None)
}

type ProcessSize = Box<dyn FnOnce(u32, u32) -> Result<(u32, u32), anyhow::Error>>;
fn default_size() -> ProcessSize {
    Box::new(|width, height| Ok((width, height)))
}
fn convert_size_proc(proc: Proc) -> ProcessSize {
    Box::new(move |width, height| {
        let result: Result<(i64, i64), magnus::Error> = proc.call((width as i64, height as i64));
        match result {
            Ok((width, height)) => Ok((width as u32, height as u32)),
            Err(err) => Err(anyhow::anyhow!(
                "svg2img: Failed to call size proc: {err:?}"
            )),
        }
    })
}

struct Options {
    size: ProcessSize,
    format: image::ImageFormat,
    output_path: Option<String>,
    super_sampling: u32,
}

fn process_svg(svg: String, options: Options) -> Result<String, anyhow::Error> {
    let image = image_from_svg(svg.as_bytes(), options.size, options.super_sampling)?;

    let mut image = image.resize_exact(
        image.width() / options.super_sampling,
        image.height() / options.super_sampling,
        image::imageops::FilterType::Lanczos3,
    );

    if options.format == image::ImageFormat::Jpeg {
        // Convert from rgba8 to rgb8
        image = DynamicImage::ImageRgb8(image.into_rgb8())
    }

    let mut buf = Vec::new();
    let mut cursor = std::io::Cursor::new(&mut buf);
    match options.format {
        image::ImageFormat::Png => {
            image::codecs::png::PngEncoder::new_with_quality(
                &mut cursor,
                image::codecs::png::CompressionType::Best,
                image::codecs::png::FilterType::Adaptive,
            )
            .write_image(
                &image.to_rgba8().into_raw(),
                image.width(),
                image.height(),
                image.color().into(),
            )
            .context("Failed to encode PNG")?;
        }
        _ => {
            image
                .write_to(&mut cursor, options.format)
                .context("Failed to write image to buffer")?;
        }
    };

    let output_path = options.output_path.unwrap_or_else(|| {
        let random_filename = format!(
            "svg2img-{}.{}",
            uuid::Uuid::new_v4(),
            options
                .format
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

fn image_from_svg(
    bytes: &[u8],
    size: ProcessSize,
    super_sampling: u32,
) -> Result<DynamicImage, anyhow::Error> {
    let svg = resvg::usvg::Tree::from_data(bytes, &resvg::usvg::Options::default())
        .context("Failed to parse SVG")?;
    let svg_width = svg.size().width();
    let svg_height = svg.size().height();
    let svg_ratio = svg_width / svg_height;

    let (image_width, image_height) = size(svg_width as u32, svg_height as u32)?;
    let image_width = image_width * super_sampling;
    let image_height = image_height * super_sampling;
    let image_ratio = image_width as f32 / image_height as f32;

    let scale = if svg_ratio > image_ratio {
        image_width as f32 / svg_width
    } else {
        image_height as f32 / svg_height
    };
    let rendered_width = svg_width * scale;
    let rendered_height = svg_height * scale;
    let tx = (image_width as f32 - rendered_width) / 2.0;
    let ty = (image_height as f32 - rendered_height) / 2.0;

    // Scale svg and place it centered
    let transform: resvg::usvg::Transform = resvg::tiny_skia::Transform {
        sx: scale,
        sy: scale,
        tx,
        ty,
        ..Default::default()
    };

    let mut pixmap = resvg::tiny_skia::Pixmap::new(image_width, image_height)
        .context("Failed to create Pixmap for SVG rendering")?;

    let result = panic::catch_unwind(AssertUnwindSafe(|| {
        resvg::render(&svg, transform, &mut pixmap.as_mut());
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

    pixmap_to_image(pixmap.width(), pixmap.height(), pixmap.data())
}

fn pixmap_to_image(width: u32, height: u32, data: &[u8]) -> Result<DynamicImage, anyhow::Error> {
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
