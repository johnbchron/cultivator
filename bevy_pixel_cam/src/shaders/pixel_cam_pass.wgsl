#import bevy_pbr::mesh_types
#import bevy_pbr::mesh_view_types

#import bevy_pbr::utils
#import bevy_pbr::prepass_utils

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

#import bevy_core_pipeline::fullscreen_vertex_shader

struct PixelCamSettings {
  window_size: vec2<f32>,
  new_pixel_size: f32,
  sample_spread: f32,
  dither_strength: f32,
}

const PIXEL_NEAR_FIELD: f32 = 10.0;

@group(0) @binding(0)
var<uniform> view: View;
@group(0) @binding(1)
var screen_texture: texture_2d<f32>;
@group(0) @binding(2)
var texture_sampler: sampler;
@group(0) @binding(3)
var<uniform> settings: PixelCamSettings;
@group(0) @binding(4)
var depth_prepass_texture: texture_depth_2d;
@group(0) @binding(5)
var normal_prepass_texture: texture_2d<f32>;

fn pixel_space_dither(frag_coord: vec2<f32>, pixel_size: f32) -> vec3<f32> {
  let dither = ((floor(frag_coord.x % (2.0 * pixel_size)) + floor(frag_coord.y % (2.0 * pixel_size))) / pixel_size) % 2.0;
  return vec3<f32>(dither, dither, dither) * 2.0 - 1.0;
}

fn depthed_pixel_size(in: FullscreenVertexOutput, template_pixel_size: f32) -> f32 {
  let near = view.projection[3][2];
  let linear_depth = near / prepass_depth(in.position, 0u);
  let pixel_size_at_depth = template_pixel_size / linear_depth * near * 100.0;
  return floor(pixel_size_at_depth * 10.0) / 10.0;
}

@fragment
fn fragment(in: FullscreenVertexOutput) -> @location(0) vec4<f32> {
  let screen_size = settings.window_size;
  // let pixel_size = settings.new_pixel_size;
  let pixel_size = depthed_pixel_size(in, settings.new_pixel_size);
  // let pixel_size = floor(in.uv.x * settings.new_pixel_size * 10.0) / 10.0;
  
  // when sample_spread is 0, samples are placed at the corners of the pixel
  // when sample_spread is 1, samples are placed at the center of the pixel
  let sample_spread = max(min(settings.sample_spread, 1.0), 0.0);
  // dither strength from 0 to 1
  let dither_strength = settings.dither_strength;
  
  var result_color: vec3<f32> = vec3<f32>(0.0, 0.0, 0.0);
    
  let pixel_uv_l = floor(in.uv.x * screen_size.x / pixel_size) * pixel_size / screen_size.x;
  let pixel_uv_r = ceil(in.uv.x * screen_size.x / pixel_size) * pixel_size / screen_size.x;
  let pixel_uv_b = floor(in.uv.y * screen_size.y / pixel_size) * pixel_size / screen_size.y;
  let pixel_uv_t = ceil(in.uv.y * screen_size.y / pixel_size) * pixel_size / screen_size.y;
  let pixel_coords = vec2<f32>((pixel_uv_l + pixel_uv_r) / 2.0, (pixel_uv_b + pixel_uv_t) / 2.0);
    
  if sample_spread != 1.0 {
    let sample_uv_l = pixel_uv_l + (pixel_uv_r - pixel_uv_l) * sample_spread / 2.0;
    let sample_uv_r = pixel_uv_r - (pixel_uv_r - pixel_uv_l) * sample_spread / 2.0;
    let sample_uv_b = pixel_uv_b + (pixel_uv_t - pixel_uv_b) * sample_spread / 2.0;
    let sample_uv_t = pixel_uv_t - (pixel_uv_t - pixel_uv_b) * sample_spread / 2.0;
    
    let pixel_color_tl = textureSample(screen_texture, texture_sampler, vec2<f32>(sample_uv_l, sample_uv_t));
    let pixel_color_tr = textureSample(screen_texture, texture_sampler, vec2<f32>(sample_uv_r, sample_uv_t));
    let pixel_color_bl = textureSample(screen_texture, texture_sampler, vec2<f32>(sample_uv_l, sample_uv_b));
    let pixel_color_br = textureSample(screen_texture, texture_sampler, vec2<f32>(sample_uv_r, sample_uv_b));
    
    result_color = (pixel_color_tl.rgb + pixel_color_tr.rgb + pixel_color_bl.rgb + pixel_color_br.rgb) / 4.0;
  } else {
    result_color = textureSample(screen_texture, texture_sampler, pixel_coords).rgb;
  }
  
  let dithered_color = result_color + pixel_space_dither(pixel_coords * screen_size, pixel_size) * dither_strength;
  
  return vec4<f32>(dithered_color.rgb, 1.0);
}