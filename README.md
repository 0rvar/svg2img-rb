# Svg2img

Convert SVG to image. Uses a bundled native binary, and requires no external dependencies.
Supported output formats for now:

- png
- jpg
- webp
- gif

## Installation

Add the `svg2img` gem to your Gemfile and run `bundle install`:

```ruby
gem "svg2img"
```

Alternatively, you can install the gem manually:

```sh
gem install svg2img
```

### Precompiled gems

We recommend installing the `svg2img` precompiled gems available for Linux and macOS. Installing a precompiled gem avoids the need to compile from source code, which is generally slower and less reliable.

When installing the `svg2img` gem for the first time using `bundle install`, Bundler will automatically download the precompiled gem for your current platform. However, you will need to inform Bundler of any additional platforms you plan to use.

To do this, lock your Bundle to the required platforms you will need from the list of supported platforms below:

```sh
bundle lock --add-platform x86_64-linux # Standard Linux (e.g. Heroku, GitHub Actions, etc.)
bundle lock --add-platform x86_64-linux-musl # MUSL Linux deployments (i.e. Alpine Linux)
bundle lock --add-platform aarch64-linux # ARM64 Linux deployments (i.e. AWS Graviton2)
bundle lock --add-platform x86_64-darwin # Intel MacOS (i.e. pre-M1)
bundle lock --add-platform arm64-darwin # Apple Silicon MacOS  (i.e. M1)
```

## Usage

Example usage:

```ruby
require "svg2img"

circle_svg = <<~SVG
  <svg xmlns="http://www.w3.org/2000/svg" width="100" height="100">
    <circle cx="50" cy="50" r="40" stroke="black" stroke-width="3" fill="red" />
  </svg>
SVG
png_path = Svg2Img.process_svg(circle_svg, output_format: :png)
# png_path is a path to the generated PNG file

# Rails example
data = Rails.cache.fetch([some, deps]) do
  png_path = Svg2Img.process_svg(circle_svg, output_format: :png)
  png_data = File.binread(png_path)
  File.delete(png_path)
  png_data
end
send_data(png_data, type: 'image/png', disposition: 'inline')
```

## Development

After checking out the repo, run `bin/setup` to install dependencies. You can also run `bin/console` for an interactive prompt that will allow you to experiment.

To install this gem onto your local machine, run `bundle exec rake install`. To release a new version, update the version number in `version.rb`, and then run `bundle exec rake release`, which will create a git tag for the version, push git commits and the created tag, and push the `.gem` file to [rubygems.org](https://rubygems.org).

## Contributing

Bug reports and pull requests are welcome on GitHub at https://github.com/0rvar/svg2img-rb.

## License

The gem is available as open source under the terms of the [MIT License](https://opensource.org/licenses/MIT).
