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
        let mut pd = PersistentData::load(world).set_last_state(GameState::HeroEditor);
        pd.hero_editor_data.editing_data.lookup.clear();
        pd.hero_editor_data.editing_data.hovered = None;
        pd.save(world).unwrap();
        Self::respawn(world);
    }

    fn ui(world: &mut World) {
        let ctx = &egui_context(world);
        let mut pd = PersistentData::load(world);
        let rep = &mut pd.hero_editor_data.rep.clone();
        let editing_data = &mut pd.hero_editor_data.editing_data.clone();
        let entity = Self::entity(world);
        let panel = TopBottomPanel::new(egui::panel::TopBottomSide::Top, "Hero Editor")
            .frame(Frame::side_top_panel(&ctx.style()).multiply_with_opacity(0.9));
        let response = panel
            .show(ctx, |ui| {
                rep.show_editor(entity, editing_data, 0, ui, world)
            })
            .response;
        response.ctx.input(|reader| {
            for event in &reader.events {
                match event {
                    egui::Event::Text(s) => {
                        editing_data.lookup.push_str(s);
                    }
                    egui::Event::Key {
                        key,
                        pressed,
                        repeat: _,
                        modifiers: _,
                    } => {
                        if *pressed && key.eq(&Key::Backspace) {
                            editing_data.lookup.pop();
                        }
                    }
                    _ => {}
                }
            }
        });
        let mut changed = false;
        if !pd.hero_editor_data.editing_data.eq(&editing_data) {
            pd.hero_editor_data.editing_data = editing_data.to_owned();
            changed = true;
        }
        if !pd.hero_editor_data.rep.eq(&rep) {
            pd.hero_editor_data.rep = rep.to_owned();
            pd.save(world).unwrap();
            Self::respawn_direct(pd.hero_editor_data.rep, world);
        } else if changed {
            pd.save(world).unwrap();
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

    fn respawn_direct(rep: Representation, world: &mut World) {
        Representation::despawn_all(world);
        rep.unpack(None, None, world);
    }

    fn respawn(world: &mut World) {
        Self::respawn_direct(PersistentData::load(world).hero_editor_data.rep, world);
    }
}

#[derive(Serialize, Deserialize, Default, Clone)]
pub struct HeroEditorData {
    pub rep: Representation,
    pub editing_data: EditingData,
}

#[derive(Serialize, Deserialize, Default, Clone, PartialEq)]
pub struct EditingData {
    pub lookup: String,
    pub hovered: Option<String>,
}
