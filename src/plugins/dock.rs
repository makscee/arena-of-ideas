use egui_dock::{NodeIndex, SurfaceIndex, Tree};

use super::*;

pub struct DockPlugin;

#[derive(Resource, Default)]
struct DockResource {
    dock: DockTree,
}

#[derive(Resource, Default)]
struct DockTabOperationsQueue {
    tabs: Vec<Box<dyn FnOnce(&mut DockTree) + 'static + Sync + Send>>,
}

impl Plugin for DockPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<DockResource>()
            .init_resource::<DockTabOperationsQueue>()
            .add_systems(Update, Self::ui);
    }
}

fn rm(world: &mut World) -> Mut<DockResource> {
    world.resource_mut::<DockResource>()
}

impl DockPlugin {
    pub fn load_state_tree(state: GameState, world: &mut World) {
        info!("Load state tree for {}", state.cstr().to_colored());
        let ds = &mut rm(world).dock.state;
        *ds.main_surface_mut() = default();
        for i in 1..ds.surfaces_count() {
            ds.remove_surface(i.into());
        }
        match state {
            GameState::Connect => {
                let tree = Tree::new(Tab::new_vec("Connect", |ui, _| ConnectPlugin::ui(ui)));
                *ds.main_surface_mut() = tree;
            }
            GameState::Login => {
                let tree = Tree::new(Tab::new_vec("Login", LoginPlugin::login_ui));
                *ds.main_surface_mut() = tree;
            }
            GameState::Title => {
                let tree = Tree::new(Tab::new_vec("Main Menu", |ui, world| {
                    ui.vertical_centered_justified(|ui| {
                        ui.add_space(ui.available_height() * 0.3);
                        ui.set_width(350.0.at_most(ui.available_width()));
                        if "Start Match"
                            .cstr_cs(VISIBLE_LIGHT, CstrStyle::Bold)
                            .button(ui)
                            .clicked()
                        {
                            GameState::Match.set_next(world);
                        }
                    });
                }));
                *ds.main_surface_mut() = tree;
            }
            _ => {}
        }
    }
    pub fn add_tab(
        name: impl ToString,
        content: impl FnMut(&mut Ui, &mut World) + Send + Sync + 'static,
        world: &mut World,
    ) {
        let name = name.to_string();
        Self::push(
            |dt| {
                dt.state
                    .push_to_focused_leaf(Tab::new(name, Box::new(content)))
            },
            world,
        );
    }
    pub fn push(f: impl FnOnce(&mut DockTree) + Send + Sync + 'static, world: &mut World) {
        world
            .resource_mut::<DockTabOperationsQueue>()
            .tabs
            .push(Box::new(f));
    }
    pub fn close_by_name(name: impl ToString, world: &mut World) {
        let name = name.to_string();
        let state = &mut rm(world).dock.state;
        state.retain_tabs(|tab| tab.name != name);

        // hack because of broken retain_tabs that leaves an empty surface, leads to panic on closing last tab of a window
        let mut to_remove: Vec<SurfaceIndex> = default();
        for (i, s) in state.iter_surfaces().enumerate() {
            if i == 0 {
                continue;
            }
            if s.iter_all_tabs().next().is_none() {
                to_remove.push(i.into());
            }
        }
        for i in to_remove {
            state.remove_surface(i);
        }
    }
    fn ui(world: &mut World) {
        let Some(ctx) = &egui_context(world).clone() else {
            return;
        };
        let operations = mem::take(&mut world.resource_mut::<DockTabOperationsQueue>().tabs);
        let dock_tree = &mut world.resource_mut::<DockResource>().dock;
        for operation in operations {
            (operation)(dock_tree);
        }
        world.resource_scope(|world, mut d: Mut<DockResource>| {
            let taken_world = mem::take(world);
            let mut taken_world = d.dock.ui(ctx, taken_world);
            mem::swap(&mut taken_world, world);
        });
    }
}
