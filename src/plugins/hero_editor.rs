use bevy_egui::egui::{Frame, Key, TopBottomPanel};

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
        let rep = &mut pd.hero_editor_data.rep;
        let editing_data = &mut pd.hero_editor_data.editing_data;
        let mut changed = false;
        let entity = Self::entity(world);
        let panel = TopBottomPanel::new(egui::panel::TopBottomSide::Top, "Hero Editor")
            .frame(Frame::side_top_panel(&ctx.style()).multiply_with_opacity(0.9));
        let response = panel
            .show(ctx, |ui| {
                (changed, _) = rep.show_tree(entity, editing_data, 0, ui, world);
            })
            .response;
        response.ctx.input(|reader| {
            for event in &reader.events {
                match event {
                    egui::Event::Text(s) => {
                        editing_data.lookup.push_str(s);
                        changed = true;
                    }
                    egui::Event::Key {
                        key,
                        pressed,
                        repeat: _,
                        modifiers: _,
                    } => {
                        if *pressed && key.eq(&Key::Backspace) {
                            editing_data.lookup.pop();
                            changed = true;
                        }
                    }
                    _ => {}
                }
            }
        });
        if changed {
            pd.save(world).unwrap();
            Self::respawn(world);
            debug!("Save pd");
        }
    }

    fn input(world: &mut World) {
        let input = world.resource::<Input<KeyCode>>();
        if input.just_pressed(KeyCode::Return) && input.pressed(KeyCode::ShiftLeft) {
            Self::respawn(world);
        }
    }

    fn entity(world: &mut World) -> Option<Entity> {
        world
            .query_filtered::<Entity, With<Representation>>()
            .iter(world)
            .last()
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
    pub editing_data: EditingData,
}

#[derive(Serialize, Deserialize, Default, Clone)]
pub struct EditingData {
    pub lookup: String,
    pub hovered: Option<String>,
}
