
/// A region of the world with a given LOD.
/// 
/// It contains a position, a depth, window size, and window transform.
#[derive(Debug, Clone)]
pub struct LodRegion {
	pub pos: [f32; 3],
	pub depth: u32,
	pub window_transform: [[f32; 4]; 4],
}
