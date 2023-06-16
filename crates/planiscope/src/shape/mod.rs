
/// All the field types that can be calculated for a `Shape`.
pub enum FieldType {
  /// The implicit surface of the shape. The shape is defined as the set of
  /// points in space where this field is zero. The field is positive outside
  /// the shape and negative inside.
  Solid,
  /// The color field of the shape. The color field two components: hue and
  /// saturation.
  Color(ColorComponent),
  /// The identity field of the shape. This field describes the ID of the shape,
  /// for use in operational collisions. The field is one inside the shape and
  /// zero outside.
  Identity,
  /// The semantic field of the shape. This field describes the semantic ID of
  /// the shape, for use in semantic collisions.
  Semantic,
}

pub enum ColorComponent {
	Hue,
	Saturation,
}

pub trait Shape {
	fn build();
}