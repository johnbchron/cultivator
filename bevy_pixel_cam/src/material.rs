use bevy::{
  asset::load_internal_asset,
  prelude::*,
  reflect::TypeUuid,
  render::render_resource::{AsBindGroup, ShaderRef},
};

pub const MATERIAL_FRAGMENT_SHADER_HANDLE: HandleUntyped =
  HandleUntyped::weak_from_u64(Shader::TYPE_UUID, 11127992064966948873);

// This is the struct that will be passed to your shader
#[derive(AsBindGroup, Clone, TypeUuid, Default)]
#[uuid = "4ee9c363-1124-4113-890e-199d81b00281"]
pub struct PixelMaterial {
  #[uniform(0)]
  pub color: Color,
  #[texture(1)]
  #[sampler(2)]
  pub color_texture: Option<Handle<Image>>,
  pub alpha_mode: AlphaMode,
}

pub struct PixelMaterialPlugin;
impl Plugin for PixelMaterialPlugin {
  fn build(&self, app: &mut App) {
    load_internal_asset!(
      app,
      MATERIAL_FRAGMENT_SHADER_HANDLE,
      "shaders/material_fragment.wgsl",
      Shader::from_wgsl
    );
  }
}

/// The Material trait is very configurable, but comes with sensible defaults for all methods.
/// You only need to implement functions for features that need non-default behavior. See the Material api docs for details!
impl Material for PixelMaterial {
  // fn vertex_shader() -> ShaderRef {
  // 	"shaders/custom_material.vert".into()
  // }

  fn fragment_shader() -> ShaderRef {
    ShaderRef::Handle(MATERIAL_FRAGMENT_SHADER_HANDLE.typed())
  }

  fn alpha_mode(&self) -> AlphaMode {
    self.alpha_mode
  }
}
