// This shader computes the chromatic aberration effect

#import bevy_pbr::utils

// Since post processing is a fullscreen effect, we use the fullscreen vertex shader provided by bevy.
// This will import a vertex shader that renders a single fullscreen triangle.
//
// A fullscreen triangle is a single triangle that covers the entire screen.
// The box in the top left in that diagram is the screen. The 4 x are the corner of the screen
//
// Y axis
//  1 |  x-----x......
//  0 |  |  s  |  . ´
// -1 |  x_____x´
// -2 |  :  .´
// -3 |  :´
//    +---------------  X axis
//      -1  0  1  2  3
//
// As you can see, the triangle ends up bigger than the screen.
//
// You don't need to worry about this too much since bevy will compute the correct UVs for you.
#import bevy_core_pipeline::fullscreen_vertex_shader

@group(0) @binding(0)
var screen_texture: texture_2d<f32>;
@group(0) @binding(1)
var texture_sampler: sampler;
struct PixelCamSettings {
  window_size: vec2<f32>,
  new_pixel_size: f32,
  sample_spread: f32,
  dither_strength: f32,
  n_colors: f32,
}
@group(0) @binding(2)
var<uniform> settings: PixelCamSettings;

// source: line 267 of
//  bevy/crates/bevy_core_pipeline/src/tonemapping/tonemapping_shared.wgsl
fn screen_space_dither(frag_coord: vec2<f32>) -> vec3<f32> {
  var dither = vec3<f32>(dot(vec2<f32>(171.0, 231.0), frag_coord)).xxx;
  dither = fract(dither.rgb / vec3<f32>(103.0, 71.0, 97.0));
  return (dither - 0.5) / 255.0;
}

@fragment
fn fragment(in: FullscreenVertexOutput) -> @location(0) vec4<f32> {
  let screen_size = settings.window_size;
  let pixel_size = settings.new_pixel_size;
  // when sample_spread is 0, samples are placed at the corners of the pixel
  // when sample_spread is 1, samples are placed at the center of the pixel
  let sample_spread = settings.sample_spread;
  // dither strength from 0 to 1
  let dither_strength = settings.dither_strength;
  let n_colors = settings.n_colors;
  
  let pixel_uv_l = floor(in.uv.x * screen_size.x / pixel_size) * pixel_size / screen_size.x;
  let pixel_uv_r = ceil(in.uv.x * screen_size.x / pixel_size) * pixel_size / screen_size.x;
  let pixel_uv_b = floor(in.uv.y * screen_size.y / pixel_size) * pixel_size / screen_size.y;
  let pixel_uv_t = ceil(in.uv.y * screen_size.y / pixel_size) * pixel_size / screen_size.y;
  let pixel_coords = vec2<f32>(pixel_uv_l, pixel_uv_b) * screen_size;
  
  let sample_uv_l = pixel_uv_l + (pixel_uv_r - pixel_uv_l) * sample_spread / 2.0;
  let sample_uv_r = pixel_uv_r - (pixel_uv_r - pixel_uv_l) * sample_spread / 2.0;
  let sample_uv_b = pixel_uv_b + (pixel_uv_t - pixel_uv_b) * sample_spread / 2.0;
  let sample_uv_t = pixel_uv_t - (pixel_uv_t - pixel_uv_b) * sample_spread / 2.0;
  
  let pixel_color_tl = textureSample(screen_texture, texture_sampler, vec2<f32>(sample_uv_l, sample_uv_t));
  let pixel_color_tr = textureSample(screen_texture, texture_sampler, vec2<f32>(sample_uv_r, sample_uv_t));
  let pixel_color_bl = textureSample(screen_texture, texture_sampler, vec2<f32>(sample_uv_l, sample_uv_b));
  let pixel_color_br = textureSample(screen_texture, texture_sampler, vec2<f32>(sample_uv_r, sample_uv_b));
  
  let blended_color = (pixel_color_tl.rgb + pixel_color_tr.rgb + pixel_color_bl.rgb + pixel_color_br.rgb) / 4.0;
  let dithered_color = (blended_color * (dither_strength * 255.0) + screen_space_dither(pixel_coords)) / (dither_strength * 255.0);
  
  let quantized_color = floor(dithered_color * n_colors + 0.5) / n_colors;
  
  return vec4<f32>(quantized_color, 1.0);
}