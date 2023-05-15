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

const N_COLORS: f32 = 8.0;

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

fn happy_art_curve(x: f32) -> f32 {
  return log(0.9 * x + 0.1) + 1.0;
}

fn rgb_to_hsv(c: vec3<f32>) -> vec3<f32> {
  // https://github.com/patriciogonzalezvivo/lygia/blob/cce7260a220bf453bb3d703b81a2623678131835/color/space/rgb2hsv.wgsl#L2
  let K = vec4<f32>(0.0, -0.33333333333333333333, 0.6666666666666666666, -1.0);
  let p = mix(vec4<f32>(c.bg, K.wz), vec4<f32>(c.gb, K.xy), step(c.b, c.g));
  let q = mix(vec4<f32>(p.xyw, c.r), vec4<f32>(c.r, p.yzx), step(p.x, c.r));
  let d = q.x - min(q.w, q.y);
  let e = 1.0e-10;
  return vec3<f32>(abs(q.z + (q.w - q.y) / (6.0 * d + e)), d / (q.x + e), q.x);
}

fn hsv_to_rgb(hsb : vec3<f32>) -> vec3<f32> {
  // https://github.com/patriciogonzalezvivo/lygia/blob/cce7260a220bf453bb3d703b81a2623678131835/color/space/hsv2rgb.wgsl
  var rgb = saturate(abs(((hsb.x * 6.0 + vec3<f32>(0.0, 4.0, 2.0)) % 6.0) - 3.0) - 1.0);
  return hsb.z * mix(vec3(1.), rgb, hsb.y);
}

fn coerce_color(in: vec3<f32>) -> vec3<f32> {
  var hsv: vec3<f32> = rgb_to_hsv(in);
  // snap the saturation to the nearest N_COLORS, rounding up
  hsv.y = round(hsv.y * N_COLORS) / N_COLORS;
  // make sure that the value is below log(0.9x+0.1)+1
  // https://www.youtube.com/watch?v=rxNDmfFw2zU
  hsv.z = min(happy_art_curve(1.0 - hsv.y), hsv.z);
  // // snap the value to the nearest N_COLORS
  // hsv.z = round(hsv.z * N_COLORS) / N_COLORS;
  return hsv_to_rgb(hsv);
}

@fragment
fn fragment(in: FullscreenVertexOutput) -> @location(0) vec4<f32> {
  let screen_size = settings.window_size;
  let current_pixel_size = pixel_size_from_depth(in, settings.max_pixel_size);
  
  var result_color: vec3<f32> = vec3<f32>(0.0, 0.0, 0.0);
  
  let sample_uv = stepped_uv_coords_from_screenspace_origin(in, current_pixel_size, vec2<f32>(0.0, 0.0));
  result_color = textureSample(screen_texture, texture_sampler, sample_uv).rgb;
  
  result_color = coerce_color(result_color);

  return vec4<f32>(result_color, 1.0);
}