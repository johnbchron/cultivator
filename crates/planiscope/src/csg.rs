
use fidget::{context::Node, Context};

pub fn csg_union(a: Node, b: Node, ctx: &mut Context) -> Node {
  ctx.max(a, b).unwrap()
}

pub fn csg_difference(a: Node, b: Node, ctx: &mut Context) -> Node {
  let b = ctx.neg(b).unwrap();
  ctx.max(a, b).unwrap()
}

pub fn csg_intersection(a: Node, b: Node, ctx: &mut Context) -> Node {
  ctx.min(a, b).unwrap()
}

pub fn csg_replacement(a: Node, b: Node, ctx: &mut Context) -> Node {
  let neg_a = ctx.neg(a).unwrap();
  let b = ctx.min(b, neg_a).unwrap();
  ctx.min(a, b).unwrap()
}

pub fn csg_translate(shape: Node, pos: [f32; 3], ctx: &mut Context) -> Node {
  let x = ctx.x();
  let y = ctx.y();
  let z = ctx.z();
  let pos_x = ctx.constant(pos[0].into());
  let pos_y = ctx.constant(pos[1].into());
  let pos_z = ctx.constant(pos[2].into());
  let new_x = ctx.sub(x, pos_x).unwrap();
  let new_y = ctx.sub(y, pos_y).unwrap();
  let new_z = ctx.sub(z, pos_z).unwrap();
  ctx.remap_xyz(shape, [new_x, new_y, new_z]).unwrap()
}

pub fn csg_scale(shape: Node, scale: [f32; 3], ctx: &mut Context) -> Node {
  let x = ctx.x();
  let y = ctx.y();
  let z = ctx.z();
  let scale_x = ctx.constant(scale[0].into());
  let scale_y = ctx.constant(scale[1].into());
  let scale_z = ctx.constant(scale[2].into());
  let new_x = ctx.mul(x, scale_x).unwrap();
  let new_y = ctx.mul(y, scale_y).unwrap();
  let new_z = ctx.mul(z, scale_z).unwrap();
  ctx.remap_xyz(shape, [new_x, new_y, new_z]).unwrap()
}

pub fn csg_clamp(shape: Node, ctx: &mut Context) -> Node {
  let steep_slope = ctx.constant(1000.0);
  let steep_shape = ctx.mul(shape, steep_slope).unwrap();
  let one = ctx.constant(1.0);
  let neg_one = ctx.constant(-1.0);
  let outside_bounded = ctx.min(steep_shape, one).unwrap();
  ctx.max(outside_bounded, neg_one).unwrap()
}

pub fn csg_color(shape: Node, rgb: [u8; 3], ctx: &mut Context) -> Node {
	// bitshift the rgb array into a 24bit integer
	let rgb = (rgb[0] as u32) << 16 | (rgb[1] as u32) << 8 | (rgb[2] as u32);
	// divide by 2^24 to get a float between 0 and 1
	let rgb = rgb as f64 / 2_f64.powi(24);
	// convert to a node
	let rgb = ctx.constant(rgb);
	
	// convert from -1 inside and 1 outside to 1 inside and 0 outside
	let neg_point_five = ctx.constant(-0.5);
	let one = ctx.constant(1.0);
	let shape = ctx.mul(shape, neg_point_five).unwrap();
	let shape = ctx.add(shape, one).unwrap();
	
	// multiply by rgb
	let shape = ctx.mul(shape, rgb).unwrap();
	
	// clamp to 0-1
	let zero = ctx.constant(0.0);
	let shape = ctx.max(shape, zero).unwrap();
	let one = ctx.constant(1.0);
  ctx.min(shape, one).unwrap()
}