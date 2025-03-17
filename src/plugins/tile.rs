use egui_tiles::Tile::Container;

use super::*;

pub struct TilePlugin;

#[derive(Resource, Default)]
struct TileData {
    tree: TileTree,
}

impl Plugin for TilePlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<TileData>()
            .add_systems(Update, Self::ui)
            .add_systems(OnEnter(GameState::Login), |world: &mut World| {
                Self::add_pane(Pane::Admin, world);
            });
    }
}

fn rm(world: &mut World) -> Mut<TileData> {
    world.resource_mut::<TileData>()
}

impl TilePlugin {
    pub fn load_state_tree(from: GameState, to: GameState, world: &mut World) {
        info!("Load state tree for {}", to.cstr().to_colored());
        let tree = &mut rm(world).tree.tree;
        let mut cs = client_state().clone();
        cs.tile_states.insert(from, tree.clone());
        if let Some(state) = cs.tile_states.get(&to) {
            *tree = state.clone();
        } else {
            *tree = to.load_tree();
        }
        cs.save();
    }
    pub fn add_pane(pane: Pane, world: &mut World) {
        let mut td = world.resource_mut::<TileData>();
        let tree = &mut td.tree.tree;
        let tile = dbg!(&mut tree.tiles).insert_pane(pane);
        if let Some(Container(root)) = tree.root().and_then(|id| tree.tiles.get_mut(id)) {
            root.add_child(tile);
        }
    }
    fn ui(world: &mut World) {
        let Some(ctx) = &egui_context(world) else {
            return;
        };
        world.resource_scope(|world, mut d: Mut<TileData>| {
            d.tree.behavior.world = Some(mem::take(world));
            d.tree.show(ctx);
            mem::swap(&mut d.tree.behavior.world.take().unwrap(), world);
        });
    }
}
