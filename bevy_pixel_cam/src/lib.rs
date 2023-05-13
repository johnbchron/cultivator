use bevy::core_pipeline::prepass::DepthPrepass;
use bevy::reflect::TypeUuid;
use bevy::{
  asset::load_internal_asset,
  core_pipeline::{
    core_3d, fullscreen_vertex_shader::fullscreen_shader_vertex_state,
    prepass::ViewPrepassTextures
  },
  pbr::{MAX_CASCADES_PER_LIGHT, MAX_DIRECTIONAL_LIGHTS},
  prelude::*,
  render::{
    extract_component::{
      ComponentUniforms, ExtractComponent, ExtractComponentPlugin,
      UniformComponentPlugin,
    },
    render_graph::{
      Node, NodeRunError, RenderGraph, RenderGraphContext, SlotInfo, SlotType,
    },
    render_resource::{
      BindGroupDescriptor, BindGroupEntry, BindGroupLayout,
      BindGroupLayoutDescriptor, BindGroupLayoutEntry, BindingResource,
      BindingType, CachedRenderPipelineId, ColorTargetState, ColorWrites,
      FragmentState, MultisampleState, Operations, PipelineCache,
      PrimitiveState, RenderPassColorAttachment, RenderPassDescriptor,
      RenderPipelineDescriptor, Sampler, SamplerBindingType, SamplerDescriptor,
      Shader, ShaderDefVal, ShaderStages, ShaderType, TextureFormat, TextureSampleType,
      TextureViewDimension, BufferBindingType
    },
    renderer::{RenderContext, RenderDevice},
    texture::BevyDefault,
    view::{ExtractedView, ViewTarget, ViewUniformOffset, ViewUniforms, ViewUniform},
    RenderApp,
  },
};

pub const PIXEL_CAM_SHADER_HANDLE: HandleUntyped =
  HandleUntyped::weak_from_u64(Shader::TYPE_UUID, 10389534959802286737);

pub struct PixelCamPlugin;
impl Plugin for PixelCamPlugin {
  fn build(&self, app: &mut App) {
    app
      .add_plugin(ExtractComponentPlugin::<PixelCamSettings>::default())
      .add_plugin(UniformComponentPlugin::<PixelCamSettings>::default())
      // MSAA doesn't make sense with pixelization but also breaks the pipeline
      // because the bind group doesn't expect a multisampled texture
      .insert_resource(Msaa::Off)
      .add_system(maintain_pixel_cam_screen_resolution);

    // Load the shader and assign it its handle
    // This method is used to make sure the shader is bundled with the app
    // and not loaded at runtime.
    load_internal_asset!(
      app,
      PIXEL_CAM_SHADER_HANDLE,
      "shaders/pixel_cam_pass.wgsl",
      Shader::from_wgsl
    );

    // We need to get the render app from the main app
    let Ok(render_app) = app.get_sub_app_mut(RenderApp) else {
			return;
		};

    // Initialize the pipeline
    render_app.init_resource::<PixelCamPipeline>();

    // Bevy's renderer uses a render graph which is a collection of nodes in a directed acyclic graph.
    // It currently runs on each view/camera and executes each node in the specified order.
    // It will make sure that any node that needs a dependency from another node only runs when that dependency is done.
    //
    // Each node can execute arbitrary work, but it generally runs at least one render pass.
    // A node only has access to the render world, so if you need data from the main world
    // you need to extract it manually or with the plugin like above.

    // Create the node with the render world
    let node = PixelCamNode::new(&mut render_app.world);

    // Get the render graph for the entire app
    let mut graph = render_app.world.resource_mut::<RenderGraph>();

    // Get the render graph for 3d cameras/views
    let core_3d_graph = graph.get_sub_graph_mut(core_3d::graph::NAME).unwrap();

    // Register the post process node in the 3d render graph
    core_3d_graph.add_node(PixelCamNode::NAME, node);

    // A slot edge tells the render graph which input/output value should be passed to the node.
    // In this case, the view entity, which is the entity associated with the
    // camera on which the graph is running.
    core_3d_graph.add_slot_edge(
      core_3d_graph.input_node().id,
      core_3d::graph::input::VIEW_ENTITY,
      PixelCamNode::NAME,
      PixelCamNode::IN_VIEW,
    );

    // We now need to add an edge between our node and the nodes from bevy
    // to make sure our node is ordered correctly relative to other nodes.
    //
    // Here we want our effect to run after tonemapping and before the end of the main pass post processing
    core_3d_graph
      .add_node_edge(core_3d::graph::node::TONEMAPPING, PixelCamNode::NAME);
    core_3d_graph.add_node_edge(
      PixelCamNode::NAME,
      core_3d::graph::node::END_MAIN_PASS_POST_PROCESSING,
    );
  }
}

/// The post process node used for the render graph
struct PixelCamNode {
  // The node needs a query to gather data from the ECS in order to do its rendering,
  // but it's not a normal system so we need to define it manually.
  query: QueryState<
    (
      &'static ViewUniformOffset,
      &'static ViewTarget,
      &'static ViewPrepassTextures,
    ),
    With<ExtractedView>
  >,
}

impl PixelCamNode {
  pub const IN_VIEW: &str = "view";
  pub const NAME: &str = "pixel_cam";

  fn new(world: &mut World) -> Self {
    Self {
      query: QueryState::new(world),
    }
  }
}

impl Node for PixelCamNode {
  // This defines the input slot of the node and tells the render graph what
  // we will need when running the node.
  fn input(&self) -> Vec<SlotInfo> {
    // In this case we tell the graph that our node will use the view entity.
    // Currently, every node in bevy uses this pattern, so it's safe to just copy it.
    vec![SlotInfo::new(PixelCamNode::IN_VIEW, SlotType::Entity)]
  }

  // This will run every frame before the run() method
  // The important difference is that `self` is `mut` here
  fn update(&mut self, world: &mut World) {
    // Since this is not a system we need to update the query manually.
    // This is mostly boilerplate. There are plans to remove this in the future.
    // For now, you can just copy it.
    self.query.update_archetypes(world);
  }

  // Runs the node logic
  // This is where you encode draw commands.
  //
  // This will run on every view on which the graph is running. If you don't want your effect to run on every camera,
  // you'll need to make sure you have a marker component to identify which camera(s) should run the effect.
  fn run(
    &self,
    graph_context: &mut RenderGraphContext,
    render_context: &mut RenderContext,
    world: &World,
  ) -> Result<(), NodeRunError> {
    // Get the entity of the view for the render graph where this node is running
    let view_entity = graph_context.get_input_entity(PixelCamNode::IN_VIEW)?;
    let view_uniforms = world.resource::<ViewUniforms>();
    let view_uniforms = view_uniforms.uniforms.binding().unwrap();

    // We get the data we need from the world based on the view entity passed to the node.
    // The data is the query that was defined earlier in the [`PostProcessNode`]
    let Ok((view_uniform_offset, view_target, prepass_textures)) = self.query.get_manual(world, view_entity) else {
			return Ok(());
		};

    // Get the pipeline resource that contains the global data we need to create the render pipeline
    let pixel_cam_pipeline = world.resource::<PixelCamPipeline>();

    // The pipeline cache is a cache of all previously created pipelines.
    // It is required to avoid creating a new pipeline each frame, which is expensive due to shader compilation.
    let pipeline_cache = world.resource::<PipelineCache>();

    // Get the pipeline from the cache
    let Some(pipeline) = pipeline_cache.get_render_pipeline(pixel_cam_pipeline.pipeline_id) else {
			return Ok(());
		};

    // Get the settings uniform binding
    let settings_uniforms =
      world.resource::<ComponentUniforms<PixelCamSettings>>();
    let Some(settings_binding) = settings_uniforms.uniforms().binding() else {
			return Ok(());
		};

    // This will start a new "post process write", obtaining two texture
    // views from the view target - a `source` and a `destination`.
    // `source` is the "current" main texture and you _must_ write into
    // `destination` because calling `post_process_write()` on the
    // [`ViewTarget`] will internally flip the [`ViewTarget`]'s main
    // texture to the `destination` texture. Failing to do so will cause
    // the current main texture information to be lost.
    let post_process = view_target.post_process_write();

    // The bind_group gets created each frame.
    //
    // Normally, you would create a bind_group in the Queue stage, but this doesn't work with the post_process_write().
    // The reason it doesn't work is because each post_process_write will alternate the source/destination.
    // The only way to have the correct source/destination for the bind_group is to make sure you get it during the node execution.
    let bind_group = render_context
      .render_device()
      .create_bind_group(&BindGroupDescriptor {
        label: Some("pixel_cam_bind_group"),
        layout: &pixel_cam_pipeline.layout,
        // It's important for this to match the BindGroupLayout defined in the PixelCamPipeline
        entries: &[
          BindGroupEntry {
            binding: 0,
            // Use the view uniform buffer
            resource: view_uniforms.clone(),
          },
          BindGroupEntry {
            binding: 1,
            // Make sure to use the source view
            resource: BindingResource::TextureView(post_process.source),
          },
          BindGroupEntry {
            binding: 2,
            // Use the sampler created for the pipeline
            resource: BindingResource::Sampler(&pixel_cam_pipeline.sampler),
          },
          BindGroupEntry {
            binding: 3,
            // Set the settings binding
            resource: settings_binding.clone(),
          },
          BindGroupEntry {
            binding: 4,
            // Use the depth texture view from the prepass
            resource: BindingResource::TextureView(
              &prepass_textures.depth.clone().unwrap().default_view,
            ),
          },
          // BindGroupEntry {
          //   binding: 5,
          //   // Use the normal texture view from the prepass
          //   resource: BindingResource::TextureView(
          //     &prepass_textures.normal.clone().unwrap().default_view,
          //   ),
          // },
        ],
      });

    // Begin the render pass
    let mut render_pass =
      render_context.begin_tracked_render_pass(RenderPassDescriptor {
        label: Some("post_process_pass"),
        color_attachments: &[Some(RenderPassColorAttachment {
          // We need to specify the post process destination view here
          // to make sure we write to the appropriate texture.
          view: post_process.destination,
          resolve_target: None,
          ops: Operations::default(),
        })],
        depth_stencil_attachment: None,
      });

    // This is mostly just wgpu boilerplate for drawing a fullscreen triangle,
    // using the pipeline/bind_group created above
    render_pass.set_render_pipeline(pipeline);
    render_pass.set_bind_group(0, &bind_group, &[view_uniform_offset.offset]);
    render_pass.draw(0..3, 0..1);

    Ok(())
  }
}

// This contains global data used by the render pipeline. This will be created once on startup.
#[derive(Resource)]
struct PixelCamPipeline {
  layout: BindGroupLayout,
  sampler: Sampler,
  pipeline_id: CachedRenderPipelineId,
}

impl FromWorld for PixelCamPipeline {
  fn from_world(world: &mut World) -> Self {
    let render_device = world.resource::<RenderDevice>();

    // We need to define the bind group layout used for our pipeline
    let layout =
      render_device.create_bind_group_layout(&BindGroupLayoutDescriptor {
        label: Some("pixel_cam_bind_group_layout"),
        entries: &[
          BindGroupLayoutEntry {
            binding: 0,
            visibility: ShaderStages::VERTEX
              | ShaderStages::FRAGMENT
              | ShaderStages::COMPUTE,
            ty: BindingType::Buffer {
              ty: BufferBindingType::Uniform,
              has_dynamic_offset: true,
              min_binding_size: Some(ViewUniform::min_size()),
            },
            count: None,
          },
          // The screen texture
          BindGroupLayoutEntry {
            binding: 1,
            visibility: ShaderStages::FRAGMENT,
            ty: BindingType::Texture {
              sample_type: TextureSampleType::Float { filterable: true },
              view_dimension: TextureViewDimension::D2,
              multisampled: false,
            },
            count: None,
          },
          // The sampler that will be used to sample the screen texture
          BindGroupLayoutEntry {
            binding: 2,
            visibility: ShaderStages::FRAGMENT,
            ty: BindingType::Sampler(SamplerBindingType::Filtering),
            count: None,
          },
          // The settings uniform that will control the effect
          BindGroupLayoutEntry {
            binding: 3,
            visibility: ShaderStages::FRAGMENT,
            ty: BindingType::Buffer {
              ty: bevy::render::render_resource::BufferBindingType::Uniform,
              has_dynamic_offset: false,
              min_binding_size: None,
            },
            count: None,
          },
          BindGroupLayoutEntry {
            binding: 4,
            visibility: ShaderStages::FRAGMENT,
            ty: BindingType::Texture {
              multisampled: false,
              sample_type: TextureSampleType::Depth,
              view_dimension: TextureViewDimension::D2,
            },
            count: None,
          },
          // // Normal texture
          // BindGroupLayoutEntry {
          //   binding: 5,
          //   visibility: ShaderStages::FRAGMENT,
          //   ty: BindingType::Texture {
          //     multisampled: false,
          //     sample_type: TextureSampleType::Float { filterable: true },
          //     view_dimension: TextureViewDimension::D2,
          //   },
          //   count: None,
          // },
        ],
      });

    // We can create the sampler here since it won't change at runtime and doesn't depend on the view
    let sampler = render_device.create_sampler(&SamplerDescriptor::default());

    let mut shader_defs = Vec::new();
    shader_defs.push(ShaderDefVal::UInt(
      "MAX_DIRECTIONAL_LIGHTS".to_string(),
      MAX_DIRECTIONAL_LIGHTS as u32,
    ));
    shader_defs.push(ShaderDefVal::UInt(
      "MAX_CASCADES_PER_LIGHT".to_string(),
      MAX_CASCADES_PER_LIGHT as u32,
    ));

    // // Get the shader handle
    // let shader = world
    //   .resource::<AssetServer>()
    //   .load("shaders/pixel_cam_pass.wgsl");

    let pipeline_id = world
      .resource_mut::<PipelineCache>()
      // This will add the pipeline to the cache and queue it's creation
      .queue_render_pipeline(RenderPipelineDescriptor {
        label: Some("pixel_cam_pipeline".into()),
        layout: vec![layout.clone()],
        // This will setup a fullscreen triangle for the vertex state
        vertex: fullscreen_shader_vertex_state(),
        fragment: Some(FragmentState {
          shader: PIXEL_CAM_SHADER_HANDLE.typed(),
          shader_defs: shader_defs,
          // Make sure this matches the entry point of your shader.
          // It can be anything as long as it matches here and in the shader.
          entry_point: "fragment".into(),
          targets: vec![Some(ColorTargetState {
            format: TextureFormat::bevy_default(),
            blend: None,
            write_mask: ColorWrites::ALL,
          })],
        }),
        // All of the following property are not important for this effect so just use the default values.
        // This struct doesn't have the Default trai implemented because not all field can have a default value.
        primitive: PrimitiveState::default(),
        depth_stencil: None,
        multisample: MultisampleState::default(),
        push_constant_ranges: vec![],
      });

    Self {
      layout,
      sampler,
      pipeline_id,
    }
  }
}

// This is the component that will get passed to the shader
#[derive(Component, Clone, Copy, ExtractComponent, ShaderType)]
pub struct PixelCamSettings {
  window_size: Vec2,
  pub max_pixel_size: f32,
  pub artificial_near_field: f32,
  pub decay_rate: f32,
}

impl PixelCamSettings {
  pub fn new(max_pixel_size: f32, artificial_near_field: f32, decay_rate: f32) -> Self {
    Self {
      window_size: Vec2::new(0.0, 0.0),
      max_pixel_size: max_pixel_size,
      artificial_near_field,
      decay_rate,
    }
  }
}

impl Default for PixelCamSettings {
  fn default() -> Self {
    Self::new(40.0, 2.0, 0.5)
  }
}

#[derive(Bundle, Default)]
pub struct PixelCamBundle {
  pub settings: PixelCamSettings,
  depth_prepass: DepthPrepass,
}

fn maintain_pixel_cam_screen_resolution(
  mut pixel_cam_settings: Query<&mut PixelCamSettings>,
  windows: Query<&Window, Changed<Window>>,
) {
  let Some(window) = windows.iter().next() else {
    return;
  };
  for mut pixel_cam_settings in pixel_cam_settings.iter_mut() {
    pixel_cam_settings.window_size = Vec2::new(window.width(), window.height());
  }
}