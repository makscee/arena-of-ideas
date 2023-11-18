use super::*;

pub struct HeroEditorPlugin;

impl Plugin for HeroEditorPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            (Self::ui, Self::input).run_if(in_state(GameState::HeroEditor)),
        )
        .add_systems(OnEnter(GameState::HeroEditor), Self::on_enter);
    }
}

impl HeroEditorPlugin {
    fn on_enter(world: &mut World) {
        PersistentData::save_last_state(GameState::HeroEditor, world);
    }

    fn ui(world: &mut World) {
        let ctx = &egui_context(world);
        let mut pd = PersistentData::load(world);
        let mut data = pd.hero_editor_data;
        let rep = &mut data.rep;
        let lookup = &mut data.lookup;
        Window::new("Hero Editor")
            .anchor(Align2::LEFT_TOP, [0.0, 0.0])
            .scroll2([true, true])
            .show(ctx, |ui| {
                rep.show_tree(lookup, 0, ui);
            });
        pd.hero_editor_data = data;
        pd.save(world).unwrap();
    }

    fn input(world: &mut World) {
        let input = world.resource::<Input<KeyCode>>();
        if input.just_pressed(KeyCode::Return) && input.pressed(KeyCode::ShiftLeft) {
            Self::respawn(world);
        }
    }

    fn respawn(world: &mut World) {
        Representation::despawn_all(world);
        PersistentData::load(world)
            .hero_editor_data
            .rep
            .unpack(None, None, world);
    }
}

#[derive(Serialize, Deserialize, Default, Clone)]
pub struct HeroEditorData {
    pub rep: Representation,
    pub lookup: String,
}
