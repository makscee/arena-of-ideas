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
    next_x: f32,
}

struct RenderData {
    image: Handle<Image>,
    cam: Entity,
    entity: Entity,
    ts: f32,
}

fn rm(world: &mut World) -> Mut<TextureRenderResource> {
    world.resource_mut::<TextureRenderResource>()
}

impl Plugin for TextureRenderPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<TextureRenderResource>()
            .add_systems(Update, Self::update);
    }
}

impl TextureRenderPlugin {
    fn layer() -> RenderLayers {
        RenderLayers::layer(1)
    }
    fn update(world: &mut World) {
        let mut r = rm(world);
        let ph = gt().play_head();
        let mut despawn: Vec<Entity> = default();
        r.map.retain(|_, v| {
            if ph - v.ts > 0.05 {
                despawn.push(v.cam);
                despawn.push(v.entity);
                false
            } else {
                true
            }
        });
        for entity in despawn {
            if let Some(e) = world.get_entity_mut(entity) {
                e.despawn_recursive();
            }
        }
    }
    fn spawn_camera(x: f32, entity: Entity, world: &mut World) -> RenderData {
        let size = Extent3d {
            width: 1024,
            height: 1024,
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
        let cam = world
            .spawn((
                Camera2dBundle {
                    camera: Camera {
                        order: -1,
                        target: bevy::render::camera::RenderTarget::Image(image.clone()),
                        clear_color: TRANSPARENT.to_color().into(),
                        ..default()
                    },
                    projection: bevy::prelude::OrthographicProjection {
                        scaling_mode: bevy::render::camera::ScalingMode::FixedHorizontal(3.0),
                        ..default()
                    },
                    transform: Transform::from_xyz(x, 0.0, 900.0),
                    ..default()
                },
                Self::layer(),
            ))
            .id();
        world
            .resource_mut::<EguiUserTextures>()
            .add_image(image.clone());
        RenderData {
            image,
            cam,
            entity,
            ts: gt().play_head(),
        }
    }
    fn spawn_unit(unit: PackedUnit, x: f32, world: &mut World) -> Entity {
        let entity = unit.unpack(TeamPlugin::entity(Faction::Shop, world), None, None, world);
        VarState::get_mut(entity, world).set_vec2(VarName::Position, vec2(x, 0.0).into());
        for child in get_children_recursive(entity, world) {
            world.entity_mut(child).insert(Self::layer());
        }
        entity
    }
    fn spawn_representation(rep: Representation, x: f32, world: &mut World) -> Entity {
        let entity = world.spawn_empty().id();
        rep.unpack(entity, world);
        VarState::new_with(VarName::Position, vec2(x, 0.0).into()).attach(entity, 0, world);
        for child in get_children_recursive(entity, world) {
            world.entity_mut(child).insert(Self::layer());
        }
        entity
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
    fn cached_texture(
        h: impl Hash,
        spawn: impl FnOnce(f32, &mut World) -> Entity,
        world: &mut World,
    ) -> TextureId {
        let mut hasher = DefaultHasher::new();
        h.hash(&mut hasher);
        let key = hasher.finish();

        world.resource_scope(|world, mut r: Mut<TextureRenderResource>| {
            let x = r.next_x;
            let mut increase_x = false;
            let data = r.map.entry(key).or_insert_with(|| {
                increase_x = true;
                let entity = spawn(x, world);
                Self::spawn_camera(x, entity, world)
            });
            data.ts = gt().play_head();
            let image = world
                .resource::<EguiUserTextures>()
                .image_id(&data.image)
                .unwrap();
            if increase_x {
                r.next_x += 100.0;
            }
            image
        })
    }
    pub fn texture_base_unit(unit: &TBaseUnit, world: &mut World) -> TextureId {
        Self::cached_texture(
            unit,
            |x, world| Self::spawn_unit(unit.clone().into(), x, world),
            world,
        )
    }
    pub fn texture_representation(rep: &Representation, world: &mut World) -> TextureId {
        Self::cached_texture(
            rep,
            |x, world| Self::spawn_representation(rep.clone(), x, world),
            world,
        )
    }
}
