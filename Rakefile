# frozen_string_literal: true

require "bundler/gem_tasks"
require "rb_sys/extensiontask"

task build: :compile

GEMSPEC = Gem::Specification.load("svg2img.gemspec")

RbSys::ExtensionTask.new("svg2img", GEMSPEC) do |ext|
  ext.lib_dir = "lib/svg2img"
end

task default: :compile
