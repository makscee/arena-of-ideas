use super::*;

pub struct TilePlugin;

#[derive(Resource, Default)]
struct TileData {
    tree: TileTree,
    save_requested: Option<GameState>,
}

impl Plugin for TilePlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<TileData>();
    }
}

fn rm(world: &mut World) -> Mut<TileData> {
    world.resource_mut::<TileData>()
}

impl TilePlugin {
    pub fn request_tree_save(state: GameState) {
        OperationsPlugin::add(move |world| {
            rm(world).save_requested = Some(state);
        });
    }
    fn save_tree(state: GameState, tree: Tree<Pane>) {
        let mut cs = client_state().clone();
        cs.tile_states.insert(state, tree);
        cs.save();
    }
    pub fn load_state_tree(state: GameState, world: &mut World) {
        info!("Load state tree for {}", state.cstr().to_colored());
        let tree = &mut rm(world).tree.tree;
        let cs = client_state();
        if let Some(state) = cs.tile_states.get(&state) {
            *tree = state.clone();
        } else {
            *tree = state.load_tree();
        }
    }
    pub fn set_active(pane: Pane) {
        OperationsPlugin::add(move |world| {
            rm(world)
                .tree
                .tree
                .make_active(|_, tile| matches!(tile, Tile::Pane(p) if *p == pane));
        });
    }
    pub fn ui(ctx: &egui::Context, world: &mut World) {
        world.resource_scope(|world, mut d: Mut<TileData>| {
            d.tree.behavior.world = Some(mem::take(world));
            d.tree.show(ctx);
            mem::swap(&mut d.tree.behavior.world.take().unwrap(), world);
            if let Some(state) = d.save_requested {
                if !left_mouse_pressed(world) {
                    d.save_requested = None;
                    Self::save_tree(state, d.tree.tree.clone());
                }
            }
        });
    }
}
