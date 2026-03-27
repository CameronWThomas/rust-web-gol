use bevy::{
    core_pipeline::fullscreen_vertex_shader::fullscreen_shader_vertex_state,
    prelude::*,
    render::{
        extract_resource::{ExtractResource, ExtractResourcePlugin},
        render_asset::{RenderAssetUsages, RenderAssets},
        render_graph::{self, RenderGraph, RenderLabel},
        render_resource::{
            binding_types::{sampler, texture_2d, uniform_buffer},
            *,
        },
        renderer::{RenderDevice, RenderQueue},
        texture::GpuImage,
        Render, RenderApp, RenderSet,
    },
    sprite::{Material2d, Material2dPlugin},
    window::WindowResolution,
};
use bevy_egui::{egui, EguiContexts, EguiPlugin};
use std::borrow::Cow;

// ---------------------------------------------------------------------------
// Constants
// ---------------------------------------------------------------------------
const SIM_WIDTH: u32 = 512;
const SIM_HEIGHT: u32 = 512;

// ---------------------------------------------------------------------------
// Simulation parameters uniform (mirrors SimParams in WGSL)
// ---------------------------------------------------------------------------
#[derive(Clone, Copy, ShaderType)]
#[allow(dead_code)]
struct SimParams {
    resolution: Vec2,
    frame: u32,
    _pad: u32,
}

// ---------------------------------------------------------------------------
// CPU-side resource: two ping-pong texture handles + frame counter
// ---------------------------------------------------------------------------
#[derive(Resource, Clone, ExtractResource)]
struct GolTextures {
    textures: [Handle<Image>; 2],
    frame: u32,
}

impl GolTextures {
    fn read_idx(&self) -> usize {
        (self.frame % 2) as usize
    }
    fn write_idx(&self) -> usize {
        ((self.frame + 1) % 2) as usize
    }
}

// ---------------------------------------------------------------------------
// Helper: create an Rgba8Unorm render target seeded with random GOL state
// ---------------------------------------------------------------------------
fn create_render_target(width: u32, height: u32, images: &mut Assets<Image>) -> Handle<Image> {
    let pixel_count = (width * height) as usize;
    let mut data = vec![0u8; pixel_count * 4];
    for i in 0..pixel_count {
        if rand::random::<bool>() {
            data[i * 4 + 1] = 255; // green = alive
        }
        data[i * 4 + 3] = 255; // always opaque
    }

    let mut image = Image::new(
        Extent3d { width, height, depth_or_array_layers: 1 },
        TextureDimension::D2,
        data,
        TextureFormat::Rgba8Unorm,
        RenderAssetUsages::RENDER_WORLD,
    );
    image.texture_descriptor.usage =
        TextureUsages::TEXTURE_BINDING | TextureUsages::RENDER_ATTACHMENT | TextureUsages::COPY_DST;
    images.add(image)
}

// ---------------------------------------------------------------------------
// UI state — color pickers write here; advance_frame syncs to the material
// ---------------------------------------------------------------------------
#[derive(Resource)]
struct ColorSettings {
    alive: [f32; 3],
    dead: [f32; 3],
}

impl Default for ColorSettings {
    fn default() -> Self {
        Self {
            alive: [0.0, 1.0, 0.0], // green
            dead:  [0.0, 0.0, 0.0], // black
        }
    }
}

// ---------------------------------------------------------------------------
// Display material — fullscreen quad that samples the current GOL texture
// ---------------------------------------------------------------------------
#[derive(Asset, TypePath, AsBindGroup, Clone)]
struct GolDisplayMaterial {
    #[texture(0)]
    #[sampler(1)]
    texture: Handle<Image>,
    #[uniform(2)]
    alive_color: Vec4,
    #[uniform(3)]
    dead_color: Vec4,
}

impl Material2d for GolDisplayMaterial {
    fn fragment_shader() -> ShaderRef {
        "shaders/gol_display.wgsl".into()
    }
}

// ---------------------------------------------------------------------------
// GPU-side simulation pipeline
// ---------------------------------------------------------------------------
#[derive(Resource)]
struct GolSimPipeline {
    bind_group_layout: BindGroupLayout,
    pipeline: CachedRenderPipelineId,
}

impl FromWorld for GolSimPipeline {
    fn from_world(world: &mut World) -> Self {
        let render_device = world.resource::<RenderDevice>();

        let bind_group_layout = render_device.create_bind_group_layout(
            "gol_sim_bind_group_layout",
            &BindGroupLayoutEntries::sequential(
                ShaderStages::FRAGMENT,
                (
                    texture_2d(TextureSampleType::Float { filterable: true }),
                    sampler(SamplerBindingType::Filtering),
                    uniform_buffer::<SimParams>(false),
                ),
            ),
        );

        let shader = world
            .resource::<AssetServer>()
            .load("shaders/gol_simulation.wgsl");

        let pipeline = world
            .resource::<PipelineCache>()
            .queue_render_pipeline(RenderPipelineDescriptor {
                label: Some("gol_simulation_pipeline".into()),
                layout: vec![bind_group_layout.clone()],
                vertex: fullscreen_shader_vertex_state(),
                fragment: Some(FragmentState {
                    shader,
                    shader_defs: vec![],
                    entry_point: Cow::from("fs_main"),
                    targets: vec![Some(ColorTargetState {
                        format: TextureFormat::Rgba8Unorm,
                        blend: None,
                        write_mask: ColorWrites::ALL,
                    })],
                }),
                primitive: PrimitiveState::default(),
                depth_stencil: None,
                multisample: MultisampleState::default(),
                push_constant_ranges: vec![],
                zero_initialize_workgroup_memory: false,
            });

        GolSimPipeline { bind_group_layout, pipeline }
    }
}

// ---------------------------------------------------------------------------
// Bind group rebuilt each frame (ping-pong swap)
// ---------------------------------------------------------------------------
#[derive(Resource, Default)]
struct GolSimBindGroup(Option<BindGroup>);

fn prepare_sim_bind_group(
    mut bind_group: ResMut<GolSimBindGroup>,
    pipeline: Res<GolSimPipeline>,
    gol_textures: Res<GolTextures>,
    gpu_images: Res<RenderAssets<GpuImage>>,
    render_device: Res<RenderDevice>,
    render_queue: Res<RenderQueue>,
) {
    let Some(gpu_image) = gpu_images.get(&gol_textures.textures[gol_textures.read_idx()]) else {
        return;
    };

    let params = SimParams {
        resolution: Vec2::new(SIM_WIDTH as f32, SIM_HEIGHT as f32),
        frame: gol_textures.frame,
        _pad: 0,
    };
    let mut uniform_buf = UniformBuffer::from(params);
    uniform_buf.write_buffer(&render_device, &render_queue);

    bind_group.0 = Some(render_device.create_bind_group(
        "gol_sim_bind_group",
        &pipeline.bind_group_layout,
        &BindGroupEntries::sequential((
            &gpu_image.texture_view,
            &gpu_image.sampler,
            uniform_buf.binding().unwrap(),
        )),
    ));
}

// ---------------------------------------------------------------------------
// Render graph node — runs the simulation render pass
// ---------------------------------------------------------------------------
#[derive(Debug, Hash, PartialEq, Eq, Clone, RenderLabel)]
struct GolSimLabel;

struct GolSimNode;

impl render_graph::Node for GolSimNode {
    fn run(
        &self,
        _graph: &mut render_graph::RenderGraphContext,
        render_context: &mut bevy::render::renderer::RenderContext,
        world: &World,
    ) -> Result<(), render_graph::NodeRunError> {
        let bind_group = world.resource::<GolSimBindGroup>();
        let Some(bind_group) = &bind_group.0 else { return Ok(()); };

        let pipeline_cache = world.resource::<PipelineCache>();
        let sim_pipeline = world.resource::<GolSimPipeline>();
        let Some(pipeline) = pipeline_cache.get_render_pipeline(sim_pipeline.pipeline) else {
            return Ok(());
        };

        let gol_textures = world.resource::<GolTextures>();
        let gpu_images = world.resource::<RenderAssets<GpuImage>>();
        let Some(write_image) = gpu_images.get(&gol_textures.textures[gol_textures.write_idx()])
        else {
            return Ok(());
        };

        let mut pass =
            render_context
                .command_encoder()
                .begin_render_pass(&RenderPassDescriptor {
                    label: Some("gol_sim_pass"),
                    color_attachments: &[Some(RenderPassColorAttachment {
                        view: &write_image.texture_view,
                        resolve_target: None,
                        ops: Operations {
                            load: LoadOp::Clear(LinearRgba::BLACK.into()),
                            store: StoreOp::Store,
                        },
                    })],
                    depth_stencil_attachment: None,
                    timestamp_writes: None,
                    occlusion_query_set: None,
                });

        pass.set_pipeline(pipeline);
        pass.set_bind_group(0, bind_group, &[]);
        pass.draw(0..3, 0..1);

        Ok(())
    }
}

// ---------------------------------------------------------------------------
// System: advance frame counter + point display material at the new texture
// ---------------------------------------------------------------------------
#[derive(Resource)]
struct DisplayMaterialHandle(Handle<GolDisplayMaterial>);

fn advance_frame(
    mut gol: ResMut<GolTextures>,
    mut materials: ResMut<Assets<GolDisplayMaterial>>,
    display_material: Res<DisplayMaterialHandle>,
    color_settings: Res<ColorSettings>,
) {
    gol.frame += 1;
    if let Some(mat) = materials.get_mut(&display_material.0) {
        mat.texture = gol.textures[gol.write_idx()].clone();
        mat.alive_color = Vec4::new(
            color_settings.alive[0],
            color_settings.alive[1],
            color_settings.alive[2],
            1.0,
        );
        mat.dead_color = Vec4::new(
            color_settings.dead[0],
            color_settings.dead[1],
            color_settings.dead[2],
            1.0,
        );
    }
}

fn color_ui(mut contexts: EguiContexts, mut color_settings: ResMut<ColorSettings>) {
    egui::Window::new("GOL Settings").show(contexts.ctx_mut(), |ui| {
        ui.label("Alive color");
        ui.color_edit_button_rgb(&mut color_settings.alive);
        ui.label("Dead color");
        ui.color_edit_button_rgb(&mut color_settings.dead);
    });
}

// ---------------------------------------------------------------------------
// Plugin
// ---------------------------------------------------------------------------
struct GolPlugin;

impl Plugin for GolPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins((
            ExtractResourcePlugin::<GolTextures>::default(),
            Material2dPlugin::<GolDisplayMaterial>::default(),
        ));
        app.add_systems(Update, advance_frame);

        let render_app = app.sub_app_mut(RenderApp);
        render_app
            .init_resource::<GolSimBindGroup>()
            .add_systems(
                Render,
                prepare_sim_bind_group.in_set(RenderSet::PrepareBindGroups),
            );

        let mut graph = render_app.world_mut().resource_mut::<RenderGraph>();
        graph.add_node(GolSimLabel, GolSimNode);
        graph.add_node_edge(GolSimLabel, bevy::render::graph::CameraDriverLabel);
    }

    fn finish(&self, app: &mut App) {
        app.sub_app_mut(RenderApp).init_resource::<GolSimPipeline>();
    }
}

// ---------------------------------------------------------------------------
// main + setup
// ---------------------------------------------------------------------------
fn main() {
    App::new()
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                resolution: WindowResolution::new(SIM_WIDTH as f32, SIM_HEIGHT as f32),
                title: "Game of Life".into(),
                ..default()
            }),
            ..default()
        }))
        .add_plugins(EguiPlugin)
        .add_plugins(GolPlugin)
        .init_resource::<ColorSettings>()
        .add_systems(Startup, setup)
        .add_systems(Update, color_ui)
        .run();
}

fn setup(
    mut commands: Commands,
    mut images: ResMut<Assets<Image>>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut display_materials: ResMut<Assets<GolDisplayMaterial>>,
) {
    let tex_a = create_render_target(SIM_WIDTH, SIM_HEIGHT, &mut images);
    let tex_b = create_render_target(SIM_WIDTH, SIM_HEIGHT, &mut images);

    let mat_handle = display_materials.add(GolDisplayMaterial {
        texture: tex_a.clone(),
        alive_color: Vec4::new(0.0, 1.0, 0.0, 1.0),
        dead_color: Vec4::new(0.0, 0.0, 0.0, 1.0),
    });

    commands.insert_resource(GolTextures { textures: [tex_a, tex_b], frame: 0 });
    commands.insert_resource(DisplayMaterialHandle(mat_handle.clone()));

    commands.spawn(Camera2d::default());
    commands.spawn((
        Mesh2d(meshes.add(Rectangle::new(SIM_WIDTH as f32, SIM_HEIGHT as f32))),
        MeshMaterial2d(mat_handle),
        Transform::default(),
    ));
}