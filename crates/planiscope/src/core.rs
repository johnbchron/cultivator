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
  pub world_space_eval: bool,
}

pub struct Chunk(pub ChunkMetadata, pub FullMesh);

type LodTree = spatialtree::OctTree<spatialtree::OctVec, spatialtree::OctVec>;

pub struct ChunkMetadata {
  pub template: Template,
  pub coords: LodCoords,
  pub tape: fidget::eval::Tape<EvalFamily>,
}

pub fn build_tree(template: &Template) -> Vec<LodCoords> {
  let _ = &template.targets.iter().for_each(|v| {
    println!("{:?}", v.float_center_coords());
  });
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
  let mut transform: Transform = coords.clone().into();
  transform.scale *= template.chunk_mesh_bleed;
  if template.world_space_eval {
    transform.position *= template.volume_size / 4.0;
    transform.scale *= template.volume_size / 4.0;
  }
  let root = transform_context(root, &mut ctx, &transform);
  let local_tape = ctx.get_tape::<EvalFamily>(root).unwrap();
  let mut full_mesh: FullMesh =
    FullMesh::mesh_new(&local_tape, template.local_chunk_detail);
  full_mesh.prune();

  if !template.world_space_eval {
    transform.position *= template.volume_size / 4.0;
    transform.scale *= template.volume_size / 4.0;
  }
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
