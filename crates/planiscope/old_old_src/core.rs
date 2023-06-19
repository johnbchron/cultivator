use crate::coords::{LodCoords, transform_context, Transform};
use crate::mesh::FullMesh;

use fidget::rhai::eval;

type EvalFamily = fidget::vm::Eval;

#[derive(Clone)]
pub struct Template {
  pub source: String,
  pub volume_size: f32,
  pub local_chunk_detail: u8,
  pub neighbor_count: u8,
  pub chunk_mesh_bleed: f32,
  pub targets: Vec<LodCoords>,
}

impl Template {
  fn world_transform(&self, coords: &LodCoords) -> Transform {
    Transform {
      position: coords.float_center_coords() * self.volume_size / 4.0,
      scale: coords.float_size() * self.volume_size / 4.0 * self.chunk_mesh_bleed,
    }
  }
  fn map_transform(&self, coords: &LodCoords) -> Transform {
    self.world_transform(coords)
  }
}

pub struct Chunk(pub ChunkMetadata, pub FullMesh);

type LodTree = spatialtree::OctTree<spatialtree::OctVec, spatialtree::OctVec>;

pub struct ChunkMetadata {
  pub template: Template,
  pub coords: LodCoords,
  pub tape: fidget::eval::Tape<EvalFamily>,
}

pub fn build_tree(template: &Template) -> Vec<LodCoords> {
  let mut tree = LodTree::with_capacity(32, 64);
  tree.lod_update(
    &template
      .targets
      .clone()
      .into_iter()
      .map(|v| v.into())
      .collect::<Vec<spatialtree::OctVec>>(),
    template.neighbor_count.into(),
    |v| v,
    |_, _| {},
  );
  tree.iter_chunks().map(|(_, v)| v.chunk.into()).collect()
}

pub fn build_chunk(template: &Template, coords: LodCoords) -> Chunk {
  let (root, mut ctx) = eval(&template.source).unwrap();
  let transform = template.map_transform(&coords);
  
  let root = transform_context(root, &mut ctx, &transform);
  let local_tape = ctx.get_tape::<EvalFamily>(root).unwrap();
  let mut full_mesh: FullMesh =
    FullMesh::mesh_new(&local_tape, template.local_chunk_detail);
  full_mesh.prune();

  let transform = template.map_transform(&coords);
  full_mesh.transform(&transform);

  Chunk(
    ChunkMetadata {
      template: template.clone(),
      coords,
      tape: local_tape,
    },
    full_mesh,
  )
}
