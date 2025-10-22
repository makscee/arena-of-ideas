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

fn rm<'a>(world: &'a mut World) -> Mut<'a, TileData> {
    world.resource_mut::<TileData>()
}

impl TilePlugin {
    pub fn op(f: impl Fn(&mut Tree<Pane>) + 'static + Sync + Send) {
        OperationsPlugin::add(move |world| f(&mut rm(world).tree.tree));
    }
    pub fn add_to_current(f: impl Fn(&mut Tree<Pane>) -> TileId + 'static + Sync + Send) {
        let id = cur_tile_id();
        Self::op(move |tree| {
            let new = f(tree);
            tree.add_tab(id, new).notify_op();
        })
    }
    pub fn close_current() {
        let id = cur_tile_id();
        Self::op(move |tree| {
            tree.tiles.remove(id);
        })
    }
    pub fn close_match(predicate: fn(&Pane) -> bool) {
        Self::op(move |tree| {
            let ids = tree
                .tiles
                .iter()
                .filter_map(|tile| {
                    if let Tile::Pane(pane) = tile.1 {
                        if predicate(pane) { Some(*tile.0) } else { None }
                    } else {
                        None
                    }
                })
                .collect_vec();
            for id in ids {
                tree.tiles.remove(id);
            }
        })
    }
    pub fn request_tree_save(state: GameState) {
        OperationsPlugin::add(move |world| {
            rm(world).save_requested = Some(state);
        });
    }
    fn save_tree(state: GameState, tree: Tree<Pane>, names: HashMap<TileId, String>) {
        pd_mut(|data| {
            data.client_state.tile_states.insert(state, (tree, names));
        });
    }
    pub fn load_state_tree(state: GameState, world: &mut World) {
        info!("Load state tree for {}", state.cstr().to_colored());
        let tile_tree = &mut rm(world).tree;
        if let Some((tree, names)) = pd().client_state.tile_states.get(&state).cloned() {
            tile_tree.tree = tree;
            tile_tree.behavior.tile_names = names;
        } else {
            tile_tree.behavior.tile_names.clear();
            state.load_tree(tile_tree);
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
                    Self::save_tree(
                        state,
                        d.tree.tree.clone(),
                        d.tree.behavior.tile_names.clone(),
                    );
                }
            }
        });
    }
}
