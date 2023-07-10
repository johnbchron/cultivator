use bevy::{
  prelude::*,
  tasks::{AsyncComputeTaskPool, Task},
};
use bevy_egui::{egui, EguiContexts, EguiPlugin};
use futures_lite::future;
use planiscope::{
  comp::{CompilationSettings, Composition},
  mesh::FullMesh,
  rhai::eval,
  shape::Shape,
};

fn main() {
  App::new()
    .add_plugins(DefaultPlugins)
    .add_plugin(EguiPlugin)
    .init_resource::<ModelMaterialHandle>()
    .init_resource::<UiSettings>()
    .init_resource::<UiCode>()
    .add_startup_system(configure_visuals_system)
    .add_startup_system(configure_ui_state_system)
    .add_startup_system(setup_3d_env)
    .add_system(ui_system)
    .add_system(spawn_compute_mesh_jobs)
    .add_system(handle_tasks)
    .run();
}

#[derive(Resource, Clone)]
struct UiSettings {
  name:           String,
  parsing_error:  Option<String>,
  min_voxel_size: f32,
  translate:      [f32; 3],
  scale:          [f32; 3],
}

impl Default for UiSettings {
  fn default() -> Self {
    Self {
      name:           "shape_name".to_string(),
      parsing_error:  None,
      min_voxel_size: 0.1,
      translate:      [0.0, 0.0, 0.0],
      scale:          [5.0, 5.0, 5.0],
    }
  }
}

#[derive(Default, Resource)]
struct UiCode(pub String);

#[derive(Component)]
struct ComputeMeshJob(Task<Mesh>);

#[derive(Component)]
struct CurrentModel;

#[derive(Resource, Deref)]
struct ModelMaterialHandle(Handle<StandardMaterial>);

impl FromWorld for ModelMaterialHandle {
  fn from_world(world: &mut World) -> Self {
    let mut materials = world
      .get_resource_mut::<Assets<StandardMaterial>>()
      .unwrap();
    ModelMaterialHandle(
      materials.add(StandardMaterial::from(Color::rgb(1.0, 1.0, 1.0))),
    )
  }
}

fn configure_visuals_system(mut contexts: EguiContexts) {
  contexts.ctx_mut().set_visuals(egui::Visuals {
    window_rounding: 0.0.into(),
    ..Default::default()
  });
}

fn configure_ui_state_system(
  mut ui_settings: ResMut<UiSettings>,
  mut ui_code: ResMut<UiCode>,
) {
  ui_settings.name = "shape_name".to_string();
  ui_code.0 = "".to_string();
}

fn ui_system(
  mut contexts: EguiContexts,
  mut ui_settings: ResMut<UiSettings>,
  mut ui_code: ResMut<UiCode>,
) {
  let ctx = contexts.ctx_mut();

  egui::SidePanel::left("side_panel")
    .default_width(400.0)
    .show(ctx, |ui| {
      ui.heading("Planiscope Editor");
      ui.separator();

      ui.horizontal(|ui| {
        ui.label("Shape Name: ");
        ui.text_edit_singleline(&mut ui_settings.name);
        ui.label(".pls");
      });

      ui.vertical(|ui| {
        ui.label("Shape Code: ");
        ui.text_edit_multiline(&mut ui_code.0);
      });

      // show parsing error
      if let Some(error) = &ui_settings.parsing_error {
        ui.label(error);
      }
    });
}

fn setup_3d_env(mut commands: Commands) {
  // lights
  commands.spawn(PointLightBundle {
    transform: Transform::from_xyz(4.0, 12.0, 15.0),
    ..default()
  });

  // camera
  commands.spawn(Camera3dBundle {
    transform: Transform::from_xyz(0.0, 5.0, 10.0)
      .looking_at(Vec3::new(0.0, 0.0, 0.0), Vec3::Y),
    ..default()
  });
}

fn compute_mesh(settings: UiSettings, shapes: Vec<(Shape, [f32; 3])>) -> Mesh {
  let mut composition = Composition::new();
  shapes.into_iter().for_each(|(shape, pos)| {
    composition.add_shape(shape, pos);
  });

  let mut ctx = fidget::Context::new();
  let comp_settings = CompilationSettings {
    min_voxel_size: settings.min_voxel_size,
  };

  let solid_root_node = composition.compile_solid(&mut ctx, &comp_settings);
  let color_root_node = composition.compile_color(&mut ctx, &comp_settings);

  let solid_root_node = planiscope::csg::csg_normalize_region(
    solid_root_node,
    settings.translate,
    settings.scale,
    &mut ctx,
  );
  let color_root_node = planiscope::csg::csg_normalize_region(
    color_root_node,
    settings.translate,
    settings.scale,
    &mut ctx,
  );

  let solid_tape: fidget::eval::Tape<fidget::vm::Eval> =
    ctx.get_tape(solid_root_node).unwrap();
  let color_tape: fidget::eval::Tape<fidget::vm::Eval> =
    ctx.get_tape(color_root_node).unwrap();

  let mut full_mesh = FullMesh::mesh_new(&solid_tape, &color_tape, 7);
  full_mesh.prune();
  full_mesh.denormalize(settings.translate.into(), settings.scale.into());

  full_mesh.into()
}

fn spawn_compute_mesh_jobs(
  mut commands: Commands,
  mut ui_settings: ResMut<UiSettings>,
  ui_code: Res<UiCode>,
  mut previous_code: Local<String>,
  previous_jobs: Query<Entity, With<ComputeMeshJob>>,
) {
  let pool = AsyncComputeTaskPool::get();

  if ui_code.0 != *previous_code {
    let shape_code = ui_code.0.clone();

    for job in previous_jobs.iter() {
      commands.entity(job).despawn_recursive();
    }

    match eval(&shape_code) {
      Ok(shapes) => {
        ui_settings.parsing_error = None;
        let ui_settings = ui_settings.clone();
        let task = pool.spawn(async move { compute_mesh(ui_settings, shapes) });

        commands.spawn(ComputeMeshJob(task));
      }
      Err(error) => {
        ui_settings.parsing_error = Some(error.to_string());
      }
    }
  }

  *previous_code = ui_code.0.clone();
}

fn handle_tasks(
  mut commands: Commands,
  mut compute_mesh_jobs: Query<(Entity, &mut ComputeMeshJob)>,
  current_model: Query<Entity, With<CurrentModel>>,
  mut meshes: ResMut<Assets<Mesh>>,
  material: Res<ModelMaterialHandle>,
) {
  for (entity, mut task) in &mut compute_mesh_jobs {
    if let Some(mesh) = future::block_on(future::poll_once(&mut task.0)) {
      // Despawn the previous model
      for entity in current_model.iter() {
        commands.entity(entity).despawn_recursive();
      }

      // Add our new PbrBundle of components to our tagged entity
      commands.entity(entity).insert((
        PbrBundle {
          mesh: meshes.add(mesh),
          material: material.clone(),
          ..default()
        },
        CurrentModel,
      ));

      // Task is complete, so remove task component from entity
      commands.entity(entity).remove::<ComputeMeshJob>();
    }
  }
}
