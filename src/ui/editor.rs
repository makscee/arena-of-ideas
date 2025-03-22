use super::*;

#[derive(Resource)]
pub struct Editor<T: ShowEditor> {
    data: T,
    on_save: Box<dyn Fn(T, &mut World) + 'static + Sync + Send>,
}

pub trait ShowEditor: 'static + Clone + Send + Sync {
    fn pane_editor_ui(&mut self, pane: Pane, ui: &mut Ui, world: &mut World);
    fn add_editor_panes(&self, world: &mut World);

    fn open_editor(
        self,
        world: &mut World,
        on_save: impl Fn(Self, &mut World) + 'static + Sync + Send,
    ) {
        world.insert_resource(Editor {
            data: self,
            on_save: Box::new(on_save),
        });
    }
    fn show_editor(pane: Pane, ui: &mut Ui, world: &mut World) {
        if let Some(mut data) = world.remove_resource::<Editor<Self>>() {
            data.data.pane_editor_ui(pane, ui, world);
            world.insert_resource(data);
        } else {
            "No editor data loaded".cstr().label(ui);
        }
    }
}

impl ShowEditor for Team {
    fn pane_editor_ui(&mut self, pane: Pane, ui: &mut Ui, world: &mut World) {}
    fn add_editor_panes(&self, world: &mut World) {
        TilePlugin::op(|tree| {
            let id = tree.tiles.insert_pane(Pane::EditorTeam);
            tree.add_to_root(id).log();
        });
    }
}
