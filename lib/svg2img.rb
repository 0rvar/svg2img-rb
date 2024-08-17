# frozen_string_literal: true

require_relative "svg2img/version"

begin
  RUBY_VERSION =~ /(\d+\.\d+)/
  require "svg2img/#{Regexp.last_match(1)}/svg2img"
rescue LoadError
  require 'svg2img/svg2img'
end


module Svg2Img
  class Error < StandardError; end
  # Your code goes here...
end
