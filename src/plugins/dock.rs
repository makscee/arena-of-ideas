use egui_dock::SurfaceIndex;

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
    pub fn load_state_tree(from: GameState, to: GameState, world: &mut World) {
        info!("Load state tree for {}", to.cstr().to_colored());
        let ds = &mut rm(world).dock.state;
        let mut cs = client_state().clone();
        cs.dock_states
            .insert(from, ds.iter_surfaces().cloned().collect_vec());
        if let Some(state) = cs.dock_states.get(&to) {
            for i in (1..ds.surfaces_count()).rev() {
                ds.remove_surface(i.into());
            }
            for surface in state.into_iter() {
                match surface {
                    egui_dock::Surface::Main(tree) => *ds.main_surface_mut() = tree.clone(),
                    egui_dock::Surface::Window(..) | egui_dock::Surface::Empty => {
                        let i = ds.add_window(default());
                        *ds.get_surface_mut(i).unwrap() = surface.clone();
                    }
                }
            }
        } else {
            *ds = to.load_state();
        }
        cs.save();
    }
    pub fn push(f: impl FnOnce(&mut DockTree) + Send + Sync + 'static, world: &mut World) {
        world
            .resource_mut::<DockTabOperationsQueue>()
            .tabs
            .push(Box::new(f));
    }
    pub fn close(tab: Tab, world: &mut World) {
        let state = &mut rm(world).dock.state;
        state.retain_tabs(|t| tab != *t);

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
    pub fn set_active(tab: Tab, world: &mut World) {
        Self::push(
            move |dt| {
                if let Some(tab) = dt.state.find_tab(&tab) {
                    dt.state.set_active_tab(tab);
                } else {
                    error!("Tab not found: {tab}");
                }
            },
            world,
        );
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
        // world.resource_scope(|world, mut d: Mut<DockResource>| {
        //     let taken_world = mem::take(world);
        //     let mut taken_world = d.dock.ui(ctx, taken_world);
        //     mem::swap(&mut taken_world, world);
        // });
        Confirmation::show_current(ctx, world);
    }
}
