use bevy::{
    prelude::{Camera2dBundle, Image},
    render::{
        render_resource::{
            Extent3d, TextureDescriptor, TextureDimension, TextureFormat, TextureUsages,
        },
        view::RenderLayers,
    },
};
use bevy_egui::EguiUserTextures;
use egui::TextureId;

use super::*;

pub struct TextureRenderPlugin;

#[derive(Resource, Default)]
struct TextureRenderResource {
    map: HashMap<u64, RenderData>,
}

struct RenderData {
    image: Handle<Image>,
    cam: Entity,
    unit: Entity,
}

fn rm(world: &mut World) -> Mut<TextureRenderResource> {
    world.resource_mut::<TextureRenderResource>()
}

impl Plugin for TextureRenderPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<TextureRenderResource>();
    }
}

impl TextureRenderPlugin {
    fn add_unit(unit: PackedUnit, len: usize, world: &mut World) -> RenderData {
        let x = len as f32 * 3.0;
        let size = Extent3d {
            width: 512,
            height: 512,
            ..default()
        };
        let mut image = Image {
            texture_descriptor: TextureDescriptor {
                label: None,
                size,
                dimension: TextureDimension::D2,
                format: TextureFormat::Bgra8UnormSrgb,
                mip_level_count: 1,
                sample_count: 1,
                usage: TextureUsages::TEXTURE_BINDING
                    | TextureUsages::COPY_DST
                    | TextureUsages::RENDER_ATTACHMENT,
                view_formats: &[],
            },
            ..default()
        };
        image.resize(size);
        let image = world.resource_mut::<Assets<Image>>().add(image);

        let layer = RenderLayers::layer(1);

        let cam = world
            .spawn((
                Camera2dBundle {
                    camera: Camera {
                        order: -1,
                        target: bevy::render::camera::RenderTarget::Image(image.clone()),
                        clear_color: Color::BLACK.into(),
                        ..default()
                    },
                    projection: bevy::prelude::OrthographicProjection {
                        scaling_mode: bevy::render::camera::ScalingMode::FixedHorizontal(3.0),
                        ..default()
                    },
                    transform: Transform::from_xyz(x, 0.0, 900.0),
                    ..default()
                },
                layer.clone(),
            ))
            .id();
        world
            .resource_mut::<EguiUserTextures>()
            .add_image(image.clone());
        let unit = unit.unpack(TeamPlugin::entity(Faction::Shop, world), None, None, world);
        VarState::get_mut(unit, world).set_vec2(VarName::Position, vec2(x, 0.0).into());
        for child in get_children_recursive(unit, world) {
            world.entity_mut(child).insert(layer.clone());
        }
        RenderData { image, cam, unit }
    }
    pub fn textures(world: &mut World) -> Vec<TextureId> {
        world
            .resource::<TextureRenderResource>()
            .map
            .iter()
            .map(|(_, d)| {
                world
                    .resource::<EguiUserTextures>()
                    .image_id(&d.image)
                    .unwrap()
            })
            .collect_vec()
    }
    pub fn texture_base_unit(unit: &TBaseUnit, world: &mut World) -> TextureId {
        let mut hasher = DefaultHasher::new();
        unit.hash(&mut hasher);
        let key = hasher.finish();

        world.resource_scope(|world, mut r: Mut<TextureRenderResource>| {
            let len = r.map.len();
            let data = r
                .map
                .entry(key)
                .or_insert_with(|| Self::add_unit(unit.clone().into(), len, world));
            world
                .resource::<EguiUserTextures>()
                .image_id(&data.image)
                .unwrap()
        })
    }
}
