# frozen_string_literal: true

require_relative "lib/svg2img/version"

Gem::Specification.new do |spec|
  spec.name = "svg2img"
  spec.version = Svg2Img::VERSION
  spec.authors = ["Orvar Segerström"]
  spec.email = ["orvarsegerstrom@gmail.com"]

  spec.summary = "Convert svg to png, jpg, or webp, with no runtime dependencies."
  # spec.description = "TODO: Write a longer description or delete this line."
  spec.homepage = "https://github.com/0rvar/svg2img-rb"
  spec.license = "MIT"
  spec.required_ruby_version = ">= 3.1.0"
  spec.required_rubygems_version = ">= 3.3.11"

  # spec.metadata["allowed_push_host"] = "TODO: Set to your gem server 'https://example.com'"

  spec.metadata["homepage_uri"] = spec.homepage
  spec.metadata["source_code_uri"] = "https://github.com/0rvar/svg2img-rb"
  spec.metadata["changelog_uri"] = spec.homepage + "/blob/master/CHANGELOG.md"

  # Specify which files should be added to the gem when it is released.
  # The `git ls-files -z` loads the files in the RubyGem that have been added into git.
  gemspec = File.basename(__FILE__)
  spec.files = Dir['lib/**/*.rb', 'ext/**/*.{rs,rb}', '**/Cargo.*', 'LICENSE.txt', 'README.md']
  spec.bindir = "exe"
  spec.executables = []
  spec.require_paths = ["lib"]
  spec.extensions = ["ext/svg2img/extconf.rb"]

  spec.add_dependency 'rb_sys', '~> 0.9'
end
