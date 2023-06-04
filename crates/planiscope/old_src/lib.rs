mod mesh;
mod normals;
pub mod transform;

use fidget::eval::Tape;
use fidget::mesh::{Octree, Settings};
use fidget::rhai::eval;

use crate::mesh::{prune_mesh, FullMesh};
use crate::normals::implicit_normals;
use crate::transform::Transform;

pub type LodCoord = spatialtree::coords::OctVec;
pub type LodTree = spatialtree::OctTree<Chunk, LodCoord>;
type Evaluator = fidget::vm::Eval;

/// A container for the settings used to build a Coordination of Chunks.
#[derive(Debug, Clone)]
pub struct Template {
  /// The expression for the implicit surface passed in by the user.
  pub volume_expr: String,
  /// The size of the final volume. It will always be a cube, so this is a side length in world units.
  pub volume_size: f32,
  /// The octree depth used within each Chunk.
  pub local_detail: u8,
  /// The number of neighbors each target should be surrounded with at the same level of detail.
  pub neighbors: u8,
  /// The amount each Chunk should overlap with its neighbor. 1.0 means no overlap, 1.5 means 50% overlap.
  pub bleed: f32,
  /// The LodCoords around which to center the Coordination octree.
  pub targets: Vec<LodCoord>,
}

/// A finished chunk. It contains its mesh data and the information used to generate it.
pub struct Chunk {
  /// The template used to build this chunk.
  pub template: Template,
  /// The location and (global) detail of this chunk.
  pub coords: LodCoord,
  /// The tape after it has been optimized for this specific location.
  pub local_tape: Tape<Evaluator>,
  /// The completed mesh data for this chunk.
  pub full_mesh: FullMesh,
}

pub fn build_from_template(template: Template) -> LodTree {
  let mut tree = LodTree::with_capacity(32, 64);

  let chunk_creator = |coords: LodCoord| {
    let map_transform = Transform::from_lodcoords_to_map(coords, &template);
    let (root, mut ctx) = eval(&template.volume_expr).unwrap();
    let root = map_transform.transform_context(root, &mut ctx);
    let local_tape = ctx.get_tape::<Evaluator>(root).unwrap();

    let full_mesh = mesh_from_surface(&local_tape, template.local_detail);
    let world_transform = Transform::from_lodcoords_to_world(coords, &template);
    let full_mesh = full_mesh.transform(&world_transform);
    
    Chunk {
      template: template.clone(),
      coords,
      local_tape,
      full_mesh,
    }
  };

  tree.lod_update(&template.targets, template.neighbors as u32, chunk_creator, |_, _| {});
  tree
}

fn mesh_from_surface(tape: &Tape<Evaluator>, depth: u8) -> FullMesh {
  let settings = Settings {
    threads: 8,
    min_depth: depth,
    max_depth: 0,
  };
  let octree = Octree::build::<Evaluator>(tape, settings);
  let mesh = prune_mesh(octree.walk_dual(settings));

  let normals = implicit_normals(&mesh, tape);

  FullMesh { mesh, normals }
}
