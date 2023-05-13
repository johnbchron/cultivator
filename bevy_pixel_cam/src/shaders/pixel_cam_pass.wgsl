#import bevy_pbr::mesh_types
#import bevy_pbr::mesh_view_types
#import bevy_pbr::utils
// #import bevy_pbr::prepass_utils

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
  max_pixel_size: f32,
  artificial_near_field: f32,
  decay_rate: f32,
}
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

fn stepped_uv_coords(in: FullscreenVertexOutput, pixel_size: f32) -> vec2<f32> {
  return round(in.position.xy / pixel_size) * pixel_size / settings.window_size;
}

fn stepped_uv_coords_from_screenspace_origin(in: FullscreenVertexOutput, pixel_size: f32, origin: vec2<f32>) -> vec2<f32> {
  return (floor((in.position.xy - origin) / pixel_size) * pixel_size + origin) / settings.window_size;
}

fn linear_depth_at_uv(in: FullscreenVertexOutput) -> f32 {
  let near = view.projection[3][2];
  return near / textureLoad(depth_prepass_texture, vec2<i32>(in.position.xy), 0);
}

fn pixel_size_from_depth(in: FullscreenVertexOutput, template_pixel_size: f32) -> f32 {
  let linear_depth = linear_depth_at_uv(in);
  if (linear_depth < settings.artificial_near_field) {
    return template_pixel_size;
  }
  let unstepped_pixel_size = 1.0 / (settings.decay_rate * linear_depth + 1.0 - settings.artificial_near_field);
  return ceil(unstepped_pixel_size * ceil(abs(template_pixel_size)));
}

@fragment
fn fragment(in: FullscreenVertexOutput) -> @location(0) vec4<f32> {
  let screen_size = settings.window_size;
  let current_pixel_size = pixel_size_from_depth(in, settings.max_pixel_size);
  
  var result_color: vec3<f32> = vec3<f32>(0.0, 0.0, 0.0);
  
  let sample_uv = stepped_uv_coords_from_screenspace_origin(in, current_pixel_size, vec2<f32>(0.0, 0.0));
  result_color = textureSample(screen_texture, texture_sampler, sample_uv).rgb;

  return vec4<f32>(result_color, 1.0);
}