use super::Shape;

pub trait ShapeDef: Shape {
  fn semantic_id(&self) -> u128;
}
