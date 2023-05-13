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
const PIXEL_NEAR_FIELD: f32 = 25.0;
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

fn stepped_uv_coords(in: FullscreenVertexOutput, pixel_size: f32) -> vec2<f32> {
  return round(in.position.xy / pixel_size) * pixel_size / settings.window_size;
}

fn stepped_uv_coords_from_screenspace_origin(in: FullscreenVertexOutput, pixel_size: f32, origin: vec2<f32>) -> vec2<f32> {
  return (floor((in.position.xy - origin) / pixel_size) * pixel_size + origin) / settings.window_size;
}


fn pixel_size_from_depth(in: FullscreenVertexOutput, template_pixel_size: f32) -> f32 {
  let near = view.projection[3][2];
  let stepped_position = vec4<f32>(stepped_uv_coords(in, template_pixel_size) * settings.window_size, in.position.ba);
  let linear_depth = (near / prepass_depth(stepped_position, 0u));
  return 1.0 / (linear_depth);
}

@fragment
fn fragment(in: FullscreenVertexOutput) -> @location(0) vec4<f32> {
  let screen_size = settings.window_size;
  let depth_pixel_size = settings.new_pixel_size;
  let fragment_pixel_size = ceil(pixel_size_from_depth(in, depth_pixel_size) * depth_pixel_size);
  
  var result_color: vec3<f32> = vec3<f32>(0.0, 0.0, 0.0);

  let depth_cell_uv_coords = stepped_uv_coords_from_screenspace_origin(in, depth_pixel_size, settings.window_size / 2.0);
  let depth_cell_screen_coords = depth_cell_uv_coords * screen_size;
  let unclamped_fragment_cell_uv_coords = stepped_uv_coords(in, fragment_pixel_size);
  let fragment_cell_uv_coords = clamp(unclamped_fragment_cell_uv_coords, depth_cell_uv_coords - (depth_pixel_size / settings.window_size), depth_cell_uv_coords + (depth_pixel_size / settings.window_size));
  result_color += textureSample(screen_texture, texture_sampler, fragment_cell_uv_coords).rgb;

  return vec4<f32>(result_color, 1.0);
}