require_relative '../lib/svg2img'

RSpec.describe 'Svg2Img' do
  let(:circle_svg) do
    File.read(File.join(File.dirname(__FILE__), 'fixtures', 'circle.svg'))
  end
  it 'rejects invalid options' do
    expect{ Svg2Img.process_svg('', output_format: :monkey) }.to raise_error(ArgumentError)
    expect{ Svg2Img.process_svg('', output_format: 'monkey') }.to raise_error(ArgumentError)
    expect{ Svg2Img.process_svg('', max_width: :sym) }.to raise_error(ArgumentError)
    expect{ Svg2Img.process_svg('', max_height: "str") }.to raise_error(ArgumentError)
  end

  it 'converts svg to png' do
    png_path = Svg2Img.process_svg(circle_svg, output_format: :png)
    expect(File.exist?(png_path)).to be true
    expect(File.size(png_path)).to be > 0
    expect(File.extname(png_path)).to eq('.png')
  end

  it 'converts svg with specified output path' do
    png_path = Svg2Img.process_svg(circle_svg, output_format: :png, output_path: 'tmp/circle.png')
    expect(png_path).to eq('tmp/circle.png')
    expect(File.exist?(png_path)).to be true
    expect(File.size(png_path)).to be > 0
    expect(File.extname(png_path)).to eq('.png')
    File.delete(png_path)
  end

  it ('converts svg to jpg') do
    jpg_path = Svg2Img.process_svg(circle_svg, output_format: :jpg)
    expect(File.exist?(jpg_path)).to be true
    expect(File.size(jpg_path)).to be > 0
    expect(File.extname(jpg_path)).to eq('.jpg')
  end

  it 'converts svg to webp' do
    webp_path = Svg2Img.process_svg(circle_svg, output_format: :webp)
    expect(File.exist?(webp_path)).to be true
    expect(File.size(webp_path)).to be > 0
    expect(File.extname(webp_path)).to eq('.webp')
  end

  it 'converts svg to gif' do
    gif_path = Svg2Img.process_svg(circle_svg, output_format: :gif)
    expect(File.exist?(gif_path)).to be true
    expect(File.size(gif_path)).to be > 0
    expect(File.extname(gif_path)).to eq('.gif')
  end
end