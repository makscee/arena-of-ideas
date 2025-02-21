use super::*;

pub struct DockPlugin;

#[derive(Resource)]
struct DockResource {
    dock: DockTree,
}

#[derive(Resource, Default)]
struct DockQueuedTabs {
    tabs: Vec<TabContent>,
}

impl Plugin for DockPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(DockResource {
            dock: DockTree::new(TabContent::new("Main Menu", |ui, world| {
                if "Add 1".cstr().button(ui).clicked() {
                    Self::add_tab(
                        "New Tab 1",
                        |ui, _| {
                            "hey".cstr().label(ui);
                        },
                        world,
                    );
                }
                if "Add 2".cstr().button(ui).clicked() {
                    Self::add_tab(
                        "New Tab 2",
                        |ui, _| {
                            "hey".cstr().label(ui);
                        },
                        world,
                    );
                }
            })),
        })
        .init_resource::<DockQueuedTabs>()
        .add_systems(Update, Self::ui);
    }
}

impl DockPlugin {
    pub fn add_tab(
        name: impl ToString,
        content: impl FnMut(&mut Ui, &mut World) + Send + Sync + 'static,
        world: &mut World,
    ) {
        world
            .resource_mut::<DockQueuedTabs>()
            .tabs
            .push(TabContent::new(name, content));
    }
    fn ui(world: &mut World) {
        let Some(ctx) = &egui_context(world).clone() else {
            return;
        };
        world.resource_scope(|world, mut d: Mut<DockResource>| {
            let taken_world = mem::take(world);
            let mut taken_world = d.dock.ui(ctx, taken_world);
            mem::swap(&mut taken_world, world);
        });
        let tabs = mem::take(&mut world.resource_mut::<DockQueuedTabs>().tabs);
        let dock_tree = &mut world.resource_mut::<DockResource>().dock;
        for tab in tabs {
            dock_tree.add_tab(tab);
        }
    }
}
