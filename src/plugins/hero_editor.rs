use std::ops::Add;

use bevy::input::mouse::MouseButton;
use bevy_egui::egui::{epaint::TextShape, DragValue, Frame, Key, ScrollArea, Sense, Shape};
use hex::encode;
use ron::ser::{to_string_pretty, PrettyConfig};

use super::*;

pub struct HeroEditorPlugin;

impl Plugin for HeroEditorPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            (Self::input, Self::ui.after(PanelsPlugin::ui))
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
        PackedTeam::spawn(Faction::Left, world);
        PackedTeam::spawn(Faction::Right, world);
        pd.hero_editor_data.apply_camera(world);
        pd.hero_editor_data.load(world);
        pd.save(world).unwrap();
    }

    fn on_exit(world: &mut World) {
        Self::save(world);
        Self::clear(world);
    }

    fn save(world: &mut World) {
        debug!("Saving.");
        let mut pd = PersistentData::load(world);
        pd.hero_editor_data.save(world);
        pd.save(world).unwrap();
    }

    fn ui(world: &mut World) {
        let mut pd = PersistentData::load(world);
        let ed = &mut pd.hero_editor_data;
        let ctx = &egui_context(world);
        let hovered = world.resource::<HoveredUnit>().0;
        let mut delete: Option<Entity> = None;
        for (unit, data) in ed.units.iter_mut() {
            let unit = *unit;
            if data.active || hovered == Some(unit) {
                entity_window(unit, vec2(0.0, 0.0), None, &format!("{unit:?}"), world)
                    .frame(Frame::none())
                    .show(ctx, |ui| {
                        if ui.button_red("DELETE").clicked() {
                            delete = Some(unit);
                        }
                        ui.add_space(5.0);
                        data.show_window(unit, ui, world);
                    });
            }
        }
        if let Some(unit) = delete {
            ed.units.remove(&unit);
            world.entity_mut(unit).despawn_recursive();
            UnitPlugin::fill_gaps_and_translate(world);
        }
        for faction in [Faction::Left, Faction::Right] {
            let offset: Vec2 = match faction {
                Faction::Left => [-1.0, 0.0],
                _ => [1.0, 0.0],
            }
            .into();
            window(&format!("spawn {faction}"))
                .fixed_pos(world_to_screen(
                    (UnitPlugin::get_slot_position(faction, 0) + offset).extend(0.0),
                    world,
                ))
                .title_bar(false)
                .stroke(false)
                .set_width(60.0)
                .show(ctx, |ui| {
                    if ui.button("SPAWN").clicked() {
                        ed.spawn(faction, world);
                    }
                });
        }
        if world
            .resource::<Input<MouseButton>>()
            .get_just_released()
            .len()
            > 0
            || world.resource::<Input<KeyCode>>().get_just_released().len() > 0
        {
            pd.hero_editor_data.save(world);
        }
        pd.save(world).unwrap();
    }

    fn input(world: &mut World) {}

    fn clear(world: &mut World) {
        let mut pd = PersistentData::load(world);
        let ed = &mut pd.hero_editor_data;
        UnitPlugin::despawn_all_teams(world);
        ed.clear();
    }
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct HeroEditorData {
    pub units: HashMap<Entity, EditorEntityData>,
    pub saved_units: (Vec<PackedUnit>, Vec<PackedUnit>),

    pub camera_pos: Vec2,
    pub camera_scale: f32,
    pub lookup: String,
    pub hovered_id: Option<String>,
}

#[derive(Serialize, Deserialize, Clone, PartialEq, Debug, Default)]
pub struct EditorEntityData {
    active: bool,
    window_center: egui::Vec2,
}

impl Default for HeroEditorData {
    fn default() -> Self {
        Self {
            units: default(),
            saved_units: default(),
            camera_pos: default(),
            camera_scale: 1.0,
            lookup: default(),
            hovered_id: default(),
        }
    }
}

impl EditorEntityData {
    fn show_window(&mut self, unit: Entity, ui: &mut Ui, world: &mut World) {
        let button = ui.button_or_primary("EDIT", self.active);
        if button.clicked() {
            self.active = !self.active;
        }
        if !self.active {
            return;
        }
        let pos = button.rect.center();
        let center = self.window_center;
        let ctx = &egui_context(world);
        CentralPanel::default()
            .frame(Frame::none())
            .show(ctx, |ui| {
                ui.painter().line_segment(
                    [pos, center.to_pos2()],
                    Stroke {
                        width: 2.0,
                        color: white(),
                    },
                );
            });
        let window_pos = window(&format!("edit {unit:?}"))
            .default_pos(pos.add(egui::vec2(0.0, 150.0)))
            .title_bar(false)
            .show(&egui_context(world), |ui| {
                frame(ui, |ui| {
                    let houses: HashMap<String, Color> = HashMap::from_iter(
                        Pools::get(world)
                            .houses
                            .iter()
                            .map(|(k, v)| (k.clone(), v.color.clone().into())),
                    );
                    let mut state = VarState::get_mut(unit, world);
                    ui.horizontal(|ui| {
                        let name = &mut state.get_string(VarName::Name).unwrap();
                        ui.label("name:");
                        if TextEdit::singleline(name)
                            .desired_width(60.0)
                            .ui(ui)
                            .changed()
                        {
                            state.init(VarName::Name, VarValue::String(name.to_owned()));
                        }
                        let hp = &mut state.get_int(VarName::Hp).unwrap();
                        ui.label("hp:");
                        if DragValue::new(hp).clamp_range(0..=99).ui(ui).changed() {
                            state.init(VarName::Hp, VarValue::Int(*hp));
                        }
                        let atk = &mut state.get_int(VarName::Atk).unwrap();
                        ui.label("atk:");
                        if DragValue::new(atk).clamp_range(0..=99).ui(ui).changed() {
                            state.init(VarName::Atk, VarValue::Int(*atk));
                        }
                    });
                    ui.horizontal(|ui| {
                        ui.label("house:");
                        let house = &mut state.get_string(VarName::Houses).unwrap();
                        ComboBox::from_id_source("house")
                            .selected_text(house.clone())
                            .width(140.0)
                            .show_ui(ui, |ui| {
                                for (h, c) in houses {
                                    if ui.selectable_value(house, h.clone(), h.clone()).changed() {
                                        state.init(VarName::Houses, VarValue::String(h));
                                        state.init(VarName::HouseColor1, VarValue::Color(c));
                                    }
                                }
                            });
                    });
                    ui.horizontal(|ui| {
                        ui.label("desc:");
                        let description = &mut state.get_string(VarName::Description).unwrap();
                        if TextEdit::singleline(description)
                            .desired_width(ui.available_width().min(200.0))
                            .ui(ui)
                            .changed()
                        {
                            state.init(
                                VarName::Description,
                                VarValue::String(description.to_owned()),
                            );
                        }
                    });
                    ui.horizontal(|ui| {
                        let trigger = &mut default();
                        mem::swap(
                            trigger,
                            &mut Status::find_unit_status(unit, LOCAL_TRIGGER, world)
                                .unwrap()
                                .trigger,
                        );
                        match trigger {
                            Trigger::Fire {
                                trigger,
                                target,
                                effect,
                            } => {
                                CollapsingHeader::new("TARGET").default_open(false).show(
                                    ui,
                                    |ui| {
                                        ui.horizontal(|ui| {
                                            target.show_node(
                                                default(),
                                                None,
                                                &Context::from_owner(unit, world),
                                                ui,
                                                world,
                                            );
                                        });
                                    },
                                );
                            }
                            Trigger::Change { trigger, expr } => todo!(),
                            Trigger::List(_) => todo!(),
                        }

                        mem::swap(
                            trigger,
                            &mut Status::find_unit_status(unit, LOCAL_TRIGGER, world)
                                .unwrap()
                                .trigger,
                        );
                    });
                });
            })
            .response
            .rect
            .center();

        self.window_center = window_pos.to_vec2();
    }
}

impl HeroEditorData {
    fn save(&mut self, world: &mut World) {
        debug!("Save hero editor data start");
        self.saved_units.0.clear();
        self.saved_units.1.clear();
        let mut units = UnitPlugin::collect_factions([Faction::Left, Faction::Right].into(), world);
        units.sort_by_key(|(e, _)| VarState::get(*e, world).get_int(VarName::Slot).unwrap());
        for (unit, faction) in units {
            let packed = PackedUnit::pack(unit, world);
            let units = match faction {
                Faction::Left => &mut self.saved_units.0,
                _ => &mut self.saved_units.1,
            };
            units.push(packed);
        }
    }

    fn load(&mut self, world: &mut World) {
        debug!("Load hero editor data start");
        self.units.clear();
        let left = PackedTeam::find_entity(Faction::Left, world).unwrap();
        self.saved_units.0.iter().rev().for_each(|u| {
            let e = u.clone().unpack(left, None, world);
            self.units.insert(e, default());
        });
        let right = PackedTeam::find_entity(Faction::Right, world).unwrap();
        self.saved_units.1.iter().rev().for_each(|u| {
            let e = u.clone().unpack(right, None, world);
            self.units.insert(e, default());
        });
        UnitPlugin::fill_gaps_and_translate(world);
    }

    fn clear(&mut self) {
        self.units.clear();
        self.lookup.clear();
        self.hovered_id = None;
    }

    fn apply_camera(&mut self, world: &mut World) {
        if let Ok((mut transform, mut projection)) = world
            .query_filtered::<(&mut Transform, &mut OrthographicProjection), With<Camera>>()
            .get_single_mut(world)
        {
            let delta = self.camera_pos * self.camera_scale / projection.scale;
            self.camera_pos = delta;
            let z = transform.translation.z;
            transform.translation = delta.extend(z);
            projection.scale = self.camera_scale;
        }
    }

    fn spawn(&mut self, faction: Faction, world: &mut World) {
        let unit = ron::from_str::<PackedUnit>("()").unwrap();
        let unit = unit.unpack(
            PackedTeam::find_entity(faction, world).unwrap(),
            None,
            world,
        );
        UnitPlugin::fill_slot_gaps(faction, world);
        UnitPlugin::translate_to_slots(world);
        self.units.insert(unit, default());
    }
}

trait EditorNodeGenerator {
    fn show_node(
        &mut self,
        path: String,
        connect_pos: Option<Pos2>,
        context: &Context,
        ui: &mut Ui,
        world: &mut World,
    );
}

fn show_value(value: &Result<VarValue>, ui: &mut Ui) {
    match &value {
        Ok(v) => {
            v.to_string()
                .add_color(light_gray())
                .set_style(ColoredStringStyle::Small)
                .label(ui);
        }
        Err(e) => {
            e.to_string()
                .add_color(red())
                .set_style(ColoredStringStyle::Small)
                .as_label(ui)
                .truncate(true)
                .ui(ui);
        }
    }
}

impl EditorNodeGenerator for Expression {
    fn show_node(
        &mut self,
        path: String,
        connect_pos: Option<Pos2>,
        context: &Context,
        ui: &mut Ui,
        world: &mut World,
    ) {
        let path = format!("{path}/{self}");
        let value = self.get_value(context, world);
        let ctx = &egui_context(world);
        let InnerResponse {
            inner: node,
            response: frame_resp,
        } = frame_horizontal(ui, |ui| {
            ui.set_min_width(50.0);
            let (pos, galley, resp) = self
                .to_string()
                .add_color(self.editor_color())
                .as_label(ui)
                .sense(Sense::click())
                .layout_in_ui(ui);
            ui.painter().add(TextShape::new(pos, galley.galley));
            ui.allocate_ui_at_rect(
                resp.rect.translate(egui::vec2(0.0, resp.rect.height())),
                |ui| {
                    let style = ui.style_mut();
                    style.override_text_style = Some(TextStyle::Small);
                    style.drag_value_text_style = TextStyle::Small;
                    style.visuals.widgets.inactive.bg_stroke = Stroke {
                        width: 1.0,
                        color: dark_gray(),
                    };
                    match self {
                        Expression::Value(v) => {
                            ui.label(format!("{v:?}"));
                        }
                        Expression::Float(x) => {
                            ui.add(DragValue::new(x).speed(0.1));
                        }
                        Expression::Int(x) => {
                            ui.add(DragValue::new(x));
                        }
                        Expression::Bool(x) => {
                            ui.checkbox(x, "");
                        }
                        Expression::String(x) => {
                            ui.text_edit_singleline(x);
                        }
                        Expression::Hex(x) => {
                            let c = Color::hex(&x).unwrap_or_default().as_rgba_u8();
                            let mut c = Color32::from_rgb(c[0], c[1], c[2]);
                            if ui.color_edit_button_srgba(&mut c).changed() {
                                *x = encode(c.to_array());
                            }
                        }
                        Expression::Faction(x) => {
                            ComboBox::from_id_source(&path)
                                .selected_text(x.to_string())
                                .show_ui(ui, |ui| {
                                    for option in Faction::iter() {
                                        let text = option.to_string();
                                        ui.selectable_value(x, option, text).changed();
                                    }
                                });
                        }
                        Expression::State(x)
                        | Expression::TargetState(x)
                        | Expression::Context(x)
                        | Expression::StateLast(x) => {
                            ComboBox::from_id_source(&path)
                                .selected_text(x.to_string())
                                .show_ui(ui, |ui| {
                                    for option in VarName::iter() {
                                        if context.get_var(option, world).is_some() {
                                            let text = option.to_string();
                                            ui.selectable_value(x, option, text).changed();
                                        }
                                    }
                                });
                        }
                        Expression::WithVar(x, ..) => {
                            ui.vertical(|ui| {
                                ComboBox::from_id_source(&path)
                                    .selected_text(x.to_string())
                                    .show_ui(ui, |ui| {
                                        for option in VarName::iter() {
                                            let text = option.to_string();
                                            ui.selectable_value(x, option, text).changed();
                                        }
                                    });
                                show_value(&value, ui);
                            });
                        }
                        Expression::Vec2(x, y) => {
                            ui.add(DragValue::new(x).speed(0.1));
                            ui.add(DragValue::new(y).speed(0.1));
                        }
                        _ => show_value(&value, ui),
                    };
                },
            );
            resp
        });
        if let Some(pos) = connect_pos {
            let end = frame_resp.rect.left_center();
            let mut mid1 = pos;
            mid1.x += 5.0;
            let mut mid2 = end;
            mid2.x -= 5.0;
            let points = [pos, mid1, mid2, end];
            let curve = Shape::CubicBezier(egui::epaint::CubicBezierShape::from_points_stroke(
                points,
                false,
                Color32::TRANSPARENT,
                Stroke {
                    width: 1.0,
                    color: match value {
                        Ok(_) => light_gray(),
                        Err(_) => red(),
                    },
                },
            ));
            ui.painter().add(curve);
        }
        if node.clicked() {
            node.request_focus();
        }
        if node.has_focus() || node.lost_focus() {
            const LOOKUP_KEY: &str = "lookup";

            window("replace")
                .order(egui::Order::Foreground)
                .title_bar(false)
                .fixed_pos(frame_resp.rect.right_center().to_bvec2())
                .show(ctx, |ui| {
                    Frame::none().inner_margin(8.0).show(ui, |ui| {
                        let mut lookup = get_context_string(world, LOOKUP_KEY);
                        let mut submit = false;
                        ctx.input(|i| {
                            for e in &i.events {
                                match e {
                                    egui::Event::Text(t) => lookup += t,
                                    egui::Event::Key { key, pressed, .. } => {
                                        if *pressed {
                                            if key.eq(&Key::Backspace) && !lookup.is_empty() {
                                                lookup.remove(lookup.len() - 1);
                                            } else if matches!(key, Key::Enter | Key::Tab) {
                                                submit = true;
                                            }
                                        }
                                    }
                                    _ => {}
                                }
                            }
                        });
                        ui.label(&lookup);
                        set_context_string(world, LOOKUP_KEY, lookup.clone());
                        ScrollArea::new([false, true])
                            .max_height(300.0)
                            .show(ui, |ui| {
                                let lookup = lookup.to_lowercase();
                                frame(ui, |ui| {
                                    for e in Expression::iter() {
                                        if e.to_string().to_lowercase().contains(&lookup) {
                                            let btn = e
                                                .to_string()
                                                .add_color(e.editor_color())
                                                .rich_text(ui);
                                            let btn = ui.button(btn);
                                            if btn.clicked() || submit {
                                                btn.request_focus();
                                            }
                                            if btn.gained_focus() {
                                                *self = e.set_inner(self.clone());
                                                set_context_string(world, LOOKUP_KEY, default());
                                                break;
                                            }
                                        }
                                    }
                                });
                            });
                    });
                });
            // let mut rect = response.rect;
            // rect.set_width(220.0);
            // *rect.bottom_mut() += 300.0;
            // let ui = &mut ui.child_ui(rect, Layout::top_down_justified(egui::Align::Center));
        }
        match self {
            Expression::Zero
            | Expression::GameTime
            | Expression::RandomFloat
            | Expression::PI
            | Expression::Age
            | Expression::SlotPosition
            | Expression::OwnerFaction
            | Expression::OppositeFaction
            | Expression::Beat
            | Expression::Owner
            | Expression::Caster
            | Expression::Target
            | Expression::RandomUnit
            | Expression::RandomAdjacentUnit
            | Expression::RandomAlly
            | Expression::RandomEnemy
            | Expression::AllyUnits
            | Expression::EnemyUnits
            | Expression::AllUnits
            | Expression::AdjacentUnits => {}
            Expression::Float(_)
            | Expression::Int(_)
            | Expression::Bool(_)
            | Expression::String(_)
            | Expression::Hex(_)
            | Expression::Faction(_)
            | Expression::State(_)
            | Expression::TargetState(_)
            | Expression::StateLast(_)
            | Expression::Context(_)
            | Expression::Value(_)
            | Expression::Vec2(_, _) => {}
            Expression::Vec2E(e)
            | Expression::StringInt(e)
            | Expression::StringFloat(e)
            | Expression::StringVec(e)
            | Expression::IntFloat(e)
            | Expression::Sin(e)
            | Expression::Cos(e)
            | Expression::Sign(e)
            | Expression::Fract(e)
            | Expression::Floor(e)
            | Expression::UnitVec(e)
            | Expression::Even(e)
            | Expression::Abs(e)
            | Expression::SlotUnit(e)
            | Expression::FactionCount(e)
            | Expression::StatusCharges(e) => e.show_node(
                format!("{path}:e"),
                Some(frame_resp.rect.right_center()),
                context,
                ui,
                world,
            ),
            Expression::Vec2EE(x, y) => {
                ui.vertical(|ui| {
                    ui.horizontal(|ui| {
                        x.show_node(
                            format!("{path}:x"),
                            Some(frame_resp.rect.right_center()),
                            context,
                            ui,
                            world,
                        );
                    });
                    ui.horizontal(|ui| {
                        y.show_node(
                            format!("{path}:y"),
                            Some(frame_resp.rect.right_center()),
                            context,
                            ui,
                            world,
                        );
                    });
                });
            }
            Expression::Sum(a, b)
            | Expression::Sub(a, b)
            | Expression::Mul(a, b)
            | Expression::Div(a, b)
            | Expression::GreaterThen(a, b)
            | Expression::LessThen(a, b)
            | Expression::Min(a, b)
            | Expression::Max(a, b)
            | Expression::Equals(a, b)
            | Expression::And(a, b)
            | Expression::Or(a, b)
            | Expression::WithVar(_, a, b) => {
                ui.vertical(|ui| {
                    ui.horizontal(|ui| {
                        a.show_node(
                            format!("{path}:a"),
                            Some(frame_resp.rect.right_center()),
                            context,
                            ui,
                            world,
                        );
                    });
                    ui.horizontal(|ui| {
                        b.show_node(
                            format!("{path}:b"),
                            Some(frame_resp.rect.right_center()),
                            context,
                            ui,
                            world,
                        );
                    });
                });
            }
            Expression::If(i, t, e) => {
                ui.vertical(|ui| {
                    ui.horizontal(|ui| {
                        i.show_node(
                            format!("{path}:i"),
                            Some(frame_resp.rect.right_center()),
                            context,
                            ui,
                            world,
                        );
                    });
                    ui.horizontal(|ui| {
                        t.show_node(
                            format!("{path}:t"),
                            Some(frame_resp.rect.right_center()),
                            context,
                            ui,
                            world,
                        );
                    });
                    ui.horizontal(|ui| {
                        e.show_node(
                            format!("{path}:e"),
                            Some(frame_resp.rect.right_center()),
                            context,
                            ui,
                            world,
                        );
                    });
                });
            }
        };

        node.context_menu(|ui| {
            if ui.button("COPY").clicked() {
                save_to_clipboard(&to_string_pretty(self, PrettyConfig::new()).unwrap(), world);
                ui.close_menu();
            }
            if ui.button("PASTE").clicked() {
                *self = ron::from_str(&get_from_clipboard(world).unwrap()).unwrap();
                ui.close_menu();
            }
            if ui.button("WRAP").clicked() {
                *self = Expression::Abs(Box::new(self.clone()));
                ui.close_menu();
            }
        });
    }
}
