use bevy_egui::egui::{Context, Frame, Key, SidePanel, TopBottomPanel};
use rand::seq::IteratorRandom;
use ron::ser::{to_string_pretty, PrettyConfig};

use super::*;

pub struct HeroEditorPlugin;

impl Plugin for HeroEditorPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            (Self::input, Self::ui)
                .run_if(in_state(GameState::HeroEditor))
                .after(PanelsPlugin::ui),
        )
        .add_systems(OnEnter(GameState::HeroEditor), Self::on_enter)
        .add_systems(OnExit(GameState::HeroEditor), Self::on_exit);
    }
}

impl HeroEditorPlugin {
    fn on_enter(world: &mut World) {
        let mut pd = PersistentData::load(world);
        pd.hero_editor_data.editing_data.lookup.clear();
        pd.hero_editor_data.editing_data.hovered = None;
        pd.save(world).unwrap();
        PackedTeam::spawn(Faction::Left, world);
        PackedTeam::spawn(Faction::Right, world);
        Self::apply_camera(&mut pd, true, world);
        Self::respawn(world);
        ActionPlugin::set_timeframe(0.001, world);
    }

    fn on_exit(world: &mut World) {
        UnitPlugin::despawn_all_teams(world);
    }

    fn ui(world: &mut World) {
        let ctx = &egui_context(world);
        let mut pd = PersistentData::load(world);
        Self::side_ui(&mut pd, ctx, world);
        let hero = &mut pd.hero_editor_data.hero.clone();
        let editing_data = &mut pd.hero_editor_data.editing_data.clone();
        let entity = pd.hero_editor_data.hero_entity;
        let panel = TopBottomPanel::new(egui::panel::TopBottomSide::Top, "Hero Editor")
            .frame(Frame::side_top_panel(&ctx.style()).multiply_with_opacity(0.9));
        let response = panel
            .show(ctx, |ui| hero.show_editor(entity, editing_data, ui, world))
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
        if !pd.hero_editor_data.editing_data.eq(editing_data) {
            pd.hero_editor_data.editing_data = editing_data.to_owned();
            changed = true;
        }
        if !pd.hero_editor_data.hero.eq(hero) {
            pd.hero_editor_data.hero = hero.to_owned();
            Self::respawn_direct(&mut pd, world);
        } else if changed {
            pd.save(world).unwrap();
        }
    }

    fn get_unit_from_clipboard(world: &mut World) -> Result<PackedUnit> {
        let hero = get_from_clipboard(world);
        if let Some(hero) = hero {
            let hero = ron::from_str::<PackedUnit>(&hero);
            return hero.map_err(|e| anyhow!("{e}"));
        }
        Err(anyhow!("Clipboard is empty"))
    }

    fn side_ui(pd: &mut PersistentData, ctx: &Context, world: &mut World) {
        SidePanel::new(egui::panel::Side::Right, "hero editor bottom").show(ctx, |ui| {
            ui.vertical_centered_justified(|ui| {
                if ui.button("Clear").clicked() {
                    pd.hero_editor_data.clear();
                    pd.save(world).unwrap();
                    Self::respawn_direct(pd, world);
                }
                if ui.button("Save to Clipboard").clicked() {
                    save_to_clipboard(
                        &to_string_pretty(&pd.hero_editor_data.hero, PrettyConfig::new()).unwrap(),
                        world,
                    );
                }
                if ui.button("Load from Clipboard").clicked() {
                    match Self::get_unit_from_clipboard(world) {
                        Ok(hero) => {
                            let mut pd = PersistentData::load(world);
                            debug!("Loaded {hero:#?}");
                            pd.hero_editor_data.hero = hero;
                            Self::respawn_direct(&mut pd, world);
                        }
                        Err(e) => error!("Failed to get hero: {e}"),
                    }
                }
                if ui.button("Load Random").clicked() {
                    let mut pd = PersistentData::load(world);
                    pd.hero_editor_data.hero = Pools::get(world)
                        .heroes
                        .values()
                        .choose(&mut thread_rng())
                        .unwrap()
                        .clone();
                    Self::respawn_direct(&mut pd, world);
                }
                let spawn_ally = ui.button("Paste Ally").clicked();
                let spawn_enemy = ui.button("Paste Enemy").clicked();
                if spawn_ally || spawn_enemy {
                    let faction = match spawn_ally {
                        true => Faction::Left,
                        false => Faction::Right,
                    };
                    match Self::get_unit_from_clipboard(world) {
                        Ok(hero) => {
                            hero.unpack(faction.team_entity(world), None, world);
                            UnitPlugin::fill_slot_gaps(faction, world);
                            UnitPlugin::translate_to_slots(world);
                        }
                        Err(e) => error!("Failed to get hero: {e}"),
                    }
                }
                let mut changed = false;
                let pos = &mut pd.hero_editor_data.editing_data.camera_pos;
                changed |= ui
                    .add(
                        Slider::new(&mut pos.x, 10.0..=-10.0)
                            .text("cam x")
                            .clamp_to_range(false),
                    )
                    .changed();
                changed |= ui
                    .add(
                        Slider::new(&mut pos.y, 10.0..=-10.0)
                            .text("cam y")
                            .clamp_to_range(false),
                    )
                    .changed();
                let scale = &mut pd.hero_editor_data.editing_data.camera_scale;

                changed |= ui
                    .add(
                        Slider::new(scale, 2.0..=0.00001)
                            .text("cam scale")
                            .clamp_to_range(false),
                    )
                    .changed();
                if changed {
                    Self::apply_camera(pd, false, world);
                }
                CollapsingHeader::new("Triggers")
                    .default_open(false)
                    .show(ui, |ui| {
                        ui.vertical_centered_justified(|ui| {
                            if ui.button("Send Battle Start").clicked() {
                                Event::BattleStart.send(world);
                            }
                            if ui.button("Send Turn Start").clicked() {
                                Event::TurnStart.send(world);
                            }
                            if ui.button("Send Turn End").clicked() {
                                Event::TurnEnd.send(world);
                            }
                        });
                    });
                if ui.button("Spawn enemy").clicked() {
                    PackedUnit {
                        hp: 5,
                        atk: 1,
                        house: "Enemy".to_owned(),
                        ..default()
                    }
                    .unpack(Faction::Right.team_entity(world), None, world);
                    UnitPlugin::fill_slot_gaps(Faction::Right, world);
                    UnitPlugin::translate_to_slots(world);
                }
                if ui.button("Run Strike").clicked() {
                    if let Some((left, right)) = BattlePlugin::get_strikers(world) {
                        BattlePlugin::run_strike(left, right, world);
                    }
                }
                if ui.button("Reset").clicked() {
                    Self::respawn(world);
                }
            })
        });
    }

    fn input(world: &mut World) {
        let input = world.resource::<Input<KeyCode>>();
        if input.just_pressed(KeyCode::Return) && input.pressed(KeyCode::ShiftLeft) {
            Self::respawn(world);
        }
    }

    fn respawn_direct(pd: &mut PersistentData, world: &mut World) {
        UnitPlugin::despawn_all_units(world);
        let unit =
            pd.hero_editor_data
                .hero
                .clone()
                .unpack(Faction::Left.team_entity(world), None, world);
        pd.hero_editor_data.hero_entity = Some(unit);
        UnitPlugin::fill_slot_gaps(Faction::Left, world);
        UnitPlugin::place_into_slot(unit, world).unwrap();
        pd.save(world).unwrap();
    }

    fn respawn(world: &mut World) {
        Self::respawn_direct(&mut PersistentData::load(world), world);
    }

    fn apply_camera(data: &mut PersistentData, initial: bool, world: &mut World) {
        if let Ok((mut transform, mut projection)) = world
            .query_filtered::<(&mut Transform, &mut OrthographicProjection), With<Camera>>()
            .get_single_mut(world)
        {
            let ed = &mut data.hero_editor_data.editing_data;
            let delta = match initial {
                true => ed.camera_pos,
                false => ed.camera_pos * ed.camera_scale / projection.scale,
            };
            let z = transform.translation.z;
            transform.translation = delta.extend(z);
            ed.camera_pos = delta;
            projection.scale = ed.camera_scale;
            data.save(world).unwrap();
        }
    }
}

#[derive(Serialize, Deserialize, Default, Clone, Debug)]
pub struct HeroEditorData {
    pub hero: PackedUnit,
    pub hero_entity: Option<Entity>,
    pub editing_data: EditingData,
}

#[derive(Serialize, Deserialize, Clone, PartialEq, Debug)]
pub struct EditingData {
    pub lookup: String,
    pub hovered: Option<String>,
    pub camera_pos: Vec2,
    pub camera_scale: f32,
}

impl Default for EditingData {
    fn default() -> Self {
        Self {
            lookup: default(),
            hovered: default(),
            camera_pos: default(),
            camera_scale: 1.0,
        }
    }
}

impl HeroEditorData {
    fn clear(&mut self) {
        self.hero = default();
        self.editing_data.hovered = None;
        self.editing_data.lookup.clear();
    }
}
