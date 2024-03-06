use std::fmt::Display;

use bevy::input::mouse::MouseButton;
use bevy_egui::egui::{DragValue, Frame, Key, ScrollArea, Sense, Shape, SidePanel};
use hex::encode;
use ron::ser::{to_string_pretty, PrettyConfig};
use serde::de::DeserializeOwned;

use super::*;

pub struct HeroEditorPlugin;

impl Plugin for HeroEditorPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            (Self::input, Self::update, Self::ui.after(PanelsPlugin::ui))
                .run_if(in_state(GameState::HeroEditor)),
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
        pd.hero_editor_data.active = None;
        pd.hero_editor_data.load(world);
        pd.save(world).unwrap();
    }

    fn on_exit(world: &mut World) {
        Self::save(world);
        UnitPlugin::despawn_all_teams(world);
    }

    fn update(world: &mut World) {
        let mut pd = PersistentData::load(world);
        let ed = &mut pd.hero_editor_data;
        let pos = if let Some((entity, _)) = ed.active {
            VarState::get(entity, world)
                .get_vec2(VarName::Position)
                .unwrap()
        } else {
            default()
        };
        ed.camera_need_pos = pos;
        ed.apply_camera(world);
        pd.save(world).unwrap();
    }

    fn input(world: &mut World) {
        let mut pd = PersistentData::load(world);
        let ed = &mut pd.hero_editor_data;
        if world.resource::<Input<KeyCode>>().just_pressed(KeyCode::Up) {
            ed.camera_scale *= 1.2;
            pd.save(world).unwrap();
        } else if world
            .resource::<Input<KeyCode>>()
            .just_pressed(KeyCode::Down)
        {
            ed.camera_scale /= 1.2;
            pd.save(world).unwrap();
        } else if world
            .resource::<Input<KeyCode>>()
            .pressed(KeyCode::SuperLeft)
        {
            if world.resource::<Input<KeyCode>>().just_pressed(KeyCode::C) {
                Self::clear(world);
            }
        }
    }

    fn save(world: &mut World) {
        debug!("Saving.");
        let mut pd = PersistentData::load(world);
        pd.hero_editor_data.save(world);
        pd.save(world).unwrap();
    }

    pub fn ui(world: &mut World) {
        let mut pd = PersistentData::load(world);
        let ed = &mut pd.hero_editor_data;
        let ctx = &egui_context(world);
        let hovered = UnitPlugin::get_hovered(world);
        let mut delete: Option<Entity> = None;
        for unit in UnitPlugin::collect_all(world) {
            let hovered = hovered == Some(unit);
            if hovered {
                entity_window(unit, vec2(0.0, 0.0), None, &format!("{unit:?}"), world)
                    .frame(Frame::none())
                    .show(ctx, |ui| {
                        let button = ui.button("EDIT");
                        if button.clicked() {
                            ed.active = Some((unit, PackedUnit::pack(unit, world)));
                        }
                        ui.add_space(5.0);
                        if ui.button_red("DELETE").clicked() {
                            delete = Some(unit);
                        }
                    });
            }
        }
        if let Some(unit) = delete {
            world.entity_mut(unit).despawn_recursive();
            UnitPlugin::fill_gaps_and_translate(world);
        }
        Self::show_edit_panel(ed, world);
        if ed.active.is_none() {
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

    fn show_edit_panel(ed: &mut HeroEditorData, world: &mut World) {
        if let Some((entity, old_unit)) = ed.active.as_ref() {
            let mut unit = old_unit.clone();
            let entity = *entity;
            let ctx = &egui_context(world);
            SidePanel::left("editor panel")
                .frame(Frame {
                    stroke: Stroke {
                        width: 1.0,
                        color: white(),
                    },
                    outer_margin: Margin::same(4.0),
                    inner_margin: Margin::same(4.0),
                    fill: black(),
                    ..default()
                })
                .default_width(ctx.screen_rect().width() * 0.7)
                .show(ctx, |ui| {
                    ui.with_layout(Layout::right_to_left(egui::Align::Min), |ui| {
                        if ui.button_red("CLOSE").clicked() {
                            ed.active = None;
                        }
                        if ui.button("PASTE").clicked() {
                            if let Some(s) = get_from_clipboard(world) {
                                match ron::from_str(&s) {
                                    Ok(u) => unit = u,
                                    Err(e) => AlertPlugin::add_error(
                                        Some("Paste Failed".to_owned()),
                                        e.to_string(),
                                        None,
                                    ),
                                }
                            }
                        }
                        if ui.button("COPY").clicked() {
                            save_to_clipboard(
                                &to_string_pretty(&unit, PrettyConfig::new()).unwrap(),
                                world,
                            );
                        }
                    });
                    ScrollArea::new([true, true])
                        .scroll_bar_visibility(egui::scroll_area::ScrollBarVisibility::AlwaysHidden)
                        .show(ui, |ui| {
                            let style = ui.style_mut();
                            style.override_text_style = Some(TextStyle::Small);
                            style.drag_value_text_style = TextStyle::Small;
                            style.visuals.widgets.inactive.bg_stroke = Stroke {
                                width: 1.0,
                                color: dark_gray(),
                            };
                            ui.horizontal(|ui| {
                                let name = &mut unit.name;
                                ui.label("name:");
                                TextEdit::singleline(name).desired_width(60.0).ui(ui);
                                let atk = &mut unit.atk;
                                ui.label("atk:");
                                DragValue::new(atk).clamp_range(0..=99).ui(ui);
                                let hp = &mut unit.hp;
                                ui.label("hp:");
                                DragValue::new(hp).clamp_range(0..=99).ui(ui);
                            });
                            ui.horizontal(|ui| {
                                let houses: HashMap<String, Color> = HashMap::from_iter(
                                    Pools::get(world)
                                        .houses
                                        .iter()
                                        .map(|(k, v)| (k.clone(), v.color.clone().into())),
                                );
                                ui.label("house:");
                                let house = &mut unit.houses;
                                ComboBox::from_id_source("house")
                                    .selected_text(house.clone())
                                    .width(140.0)
                                    .show_ui(ui, |ui| {
                                        for (h, _) in houses {
                                            ui.selectable_value(house, h.clone(), h.clone());
                                        }
                                    });
                            });
                            ui.horizontal(|ui| {
                                ui.label("desc:");
                                let description = &mut unit.description;
                                TextEdit::singleline(description)
                                    .desired_width(ui.available_width().min(200.0))
                                    .ui(ui);
                            });

                            let context = &Context::from_owner(entity, world);
                            ui.horizontal(|ui| {
                                let trigger = &mut unit.trigger;
                                match trigger {
                                    Trigger::Fire {
                                        trigger,
                                        target,
                                        effect,
                                    } => {
                                        CollapsingHeader::new("Trigger").default_open(true).show(
                                            ui,
                                            |ui| {
                                                trigger.show_editor(entity, ui);
                                                match trigger {
                                                    FireTrigger::List(list) => {
                                                        ui.vertical(|ui| {
                                                            for (i, trigger) in
                                                                list.iter_mut().enumerate()
                                                            {
                                                                ComboBox::from_id_source(
                                                                    Id::new(entity).with(i),
                                                                )
                                                                .selected_text(trigger.to_string())
                                                                .show_ui(ui, |ui| {
                                                                    for option in
                                                                        FireTrigger::iter()
                                                                    {
                                                                        let text =
                                                                            option.to_string();
                                                                        ui.selectable_value(
                                                                            trigger.as_mut(),
                                                                            option,
                                                                            text,
                                                                        );
                                                                    }
                                                                });
                                                            }
                                                            if ui.button("+").clicked() {
                                                                list.push(default());
                                                            }
                                                        });
                                                    }
                                                    _ => {}
                                                }
                                            },
                                        );
                                        CollapsingHeader::new("Target").default_open(true).show(
                                            ui,
                                            |ui| {
                                                show_tree(target, context, ui, world);
                                            },
                                        );

                                        CollapsingHeader::new("Effect").default_open(true).show(
                                            ui,
                                            |ui| {
                                                show_tree(effect, context, ui, world);
                                            },
                                        );
                                    }
                                    Trigger::Change { .. } => todo!(),
                                    Trigger::List(_) => todo!(),
                                }
                            });

                            let rep = &mut unit.representation;
                            rep.show_editor(context, "root", ui, world);
                        });
                });

            if let Some((entity, old_unit)) = ed.active.as_ref() {
                let entity = *entity;
                if !unit.eq(old_unit) {
                    let slot =
                        VarState::get(entity, world).get_int(VarName::Slot).unwrap() as usize;
                    let parent = entity.get_parent(world).unwrap();
                    world.entity_mut(entity).despawn_recursive();
                    let entity = unit.clone().unpack(parent, Some(slot), world);
                    UnitPlugin::place_into_slot(entity, world).unwrap();
                    ed.active = Some((entity, unit));
                    ed.save(world);
                }
            }
        }
    }

    fn clear(world: &mut World) {
        let mut pd = PersistentData::load(world);
        let ed = &mut pd.hero_editor_data;
        UnitPlugin::despawn_all_teams(world);
        ed.clear();
        pd.save(world).unwrap();
        Self::on_enter(world);
    }
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct HeroEditorData {
    pub active: Option<(Entity, PackedUnit)>,

    pub teams: (Vec<PackedUnit>, Vec<PackedUnit>),
    pub camera_pos: Vec2,
    pub camera_need_pos: Vec2,
    pub camera_scale: f32,
    pub lookup: String,
    pub hovered_id: Option<String>,
}

impl Default for HeroEditorData {
    fn default() -> Self {
        Self {
            camera_pos: default(),
            camera_need_pos: default(),
            camera_scale: 1.0,
            lookup: default(),
            hovered_id: default(),
            active: default(),
            teams: default(),
        }
    }
}

impl HeroEditorData {
    fn save(&mut self, world: &mut World) {
        debug!("Save hero editor data start");
        self.teams.0.clear();
        self.teams.1.clear();
        let mut units = UnitPlugin::collect_factions([Faction::Left, Faction::Right].into(), world);
        units.sort_by_key(|(e, _)| VarState::get(*e, world).get_int(VarName::Slot).unwrap());
        for (unit, faction) in units {
            let packed = PackedUnit::pack(unit, world);
            let units = match faction {
                Faction::Left => &mut self.teams.0,
                _ => &mut self.teams.1,
            };
            units.push(packed);
        }
    }

    fn load(&mut self, world: &mut World) {
        debug!("Load hero editor data start");
        let left = PackedTeam::find_entity(Faction::Left, world).unwrap();
        self.teams.0.iter().rev().for_each(|u| {
            u.clone().unpack(left, None, world);
        });
        let right = PackedTeam::find_entity(Faction::Right, world).unwrap();
        self.teams.1.iter().rev().for_each(|u| {
            u.clone().unpack(right, None, world);
        });
        UnitPlugin::fill_gaps_and_translate(world);
    }

    fn clear(&mut self) {
        self.teams.0.clear();
        self.teams.1.clear();
        self.lookup.clear();
        self.hovered_id = None;
    }

    fn apply_camera(&mut self, world: &mut World) {
        let dt = world.resource::<Time>().delta_seconds();
        if let Ok((mut transform, mut projection)) = world
            .query_filtered::<(&mut Transform, &mut OrthographicProjection), With<Camera>>()
            .get_single_mut(world)
        {
            let need_pos = if self.camera_need_pos.length() > 0.0 {
                self.camera_need_pos - vec2(projection.area.max.x - 1.2, 0.0)
            } else {
                default()
            };
            self.camera_pos += (need_pos - self.camera_pos) * dt * 10.0;
            let delta = self.camera_pos * self.camera_scale / projection.scale;
            self.camera_pos = delta;
            let z = transform.translation.z;
            transform.translation = self.camera_pos.extend(z);
            projection.scale = self.camera_scale;
        }
    }

    fn spawn(&mut self, faction: Faction, world: &mut World) {
        ron::from_str::<PackedUnit>("()").unwrap().unpack(
            PackedTeam::find_entity(faction, world).unwrap(),
            None,
            world,
        );
        UnitPlugin::fill_slot_gaps(faction, world);
        UnitPlugin::translate_to_slots(world);
    }
}

fn show_value(value: &Result<VarValue>, ui: &mut Ui) {
    match &value {
        Ok(v) => v.to_string().add_color(light_gray()),

        Err(e) => e.to_string().add_color(red()),
    }
    .set_style(ColoredStringStyle::Small)
    .as_label(ui)
    .truncate(true)
    .ui(ui);
}

pub fn show_tree(
    root: &mut impl EditorNodeGenerator,
    context: &Context,
    ui: &mut Ui,
    world: &mut World,
) {
    ui.horizontal(|ui| {
        show_node(root, default(), None, context, ui, world);
    });
}

fn show_node(
    source: &mut impl EditorNodeGenerator,
    path: String,
    connect_pos: Option<Pos2>,
    context: &Context,
    ui: &mut Ui,
    world: &mut World,
) {
    let path = format!("{path}/{source}");
    let ctx = &egui_context(world);
    let InnerResponse {
        inner: name_resp,
        response: frame_resp,
    } = Frame::none()
        .stroke(Stroke::new(1.0, dark_gray()))
        .inner_margin(6.0)
        .outer_margin(6.0)
        .rounding(0.0)
        .fill(light_black())
        .show(ui, |ui| {
            let name = source
                .to_string()
                .add_color(source.node_color())
                .as_label(ui)
                .sense(Sense::click())
                .ui(ui);
            ui.allocate_ui_at_rect(
                name.rect.translate(egui::vec2(0.0, name.rect.height())),
                |ui| {
                    source.show_extra(&path, context, world, ui);
                },
            );
            name.on_hover_text(&path)
        });

    if name_resp.clicked() {
        name_resp.request_focus();
    }
    if name_resp.has_focus() || name_resp.lost_focus() {
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
                                if source.show_replace_buttons(&lookup, submit, ui) {
                                    set_context_string(world, LOOKUP_KEY, default());
                                }
                            });
                        });
                });
            });
    }

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
                color: dark_gray(),
            },
        ));
        ui.painter().add(curve);
    }

    source.show_children(
        &path,
        Some(frame_resp.rect.right_center()),
        context,
        ui,
        world,
    );

    name_resp.context_menu(|ui| {
        if ui.button("COPY").clicked() {
            save_to_clipboard(
                &to_string_pretty(source, PrettyConfig::new()).unwrap(),
                world,
            );
            ui.close_menu();
        }
        if ui.button("PASTE").clicked() {
            let o = get_from_clipboard(world).unwrap();
            *source = ron::from_str(o.as_str()).unwrap();
            ui.close_menu();
        }
    });
}

pub trait EditorNodeGenerator: Display + Sized + Serialize + DeserializeOwned {
    fn node_color(&self) -> Color32;
    fn show_children(
        &mut self,
        path: &str,
        connect_pos: Option<Pos2>,
        context: &Context,
        ui: &mut Ui,
        world: &mut World,
    );
    fn show_extra(&mut self, path: &str, context: &Context, world: &mut World, ui: &mut Ui);
    fn show_replace_buttons(&mut self, lookup: &str, submit: bool, ui: &mut Ui) -> bool;
}

impl EditorNodeGenerator for Expression {
    fn node_color(&self) -> Color32 {
        self.editor_color()
    }

    fn show_extra(&mut self, path: &str, context: &Context, world: &mut World, ui: &mut Ui) {
        let value = self.get_value(context, world);
        match self {
            Expression::Value(v) => {
                show_value(&Ok(v.clone()), ui);
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
    }

    fn show_children(
        &mut self,
        path: &str,
        connect_pos: Option<Pos2>,
        context: &Context,
        ui: &mut Ui,
        world: &mut World,
    ) {
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
            | Expression::AdjacentUnits
            | Expression::Index
            | Expression::Float(_)
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
            | Expression::Vec2(_, _) => default(),
            Expression::Sin(x)
            | Expression::Cos(x)
            | Expression::Sign(x)
            | Expression::Fract(x)
            | Expression::Floor(x)
            | Expression::UnitVec(x)
            | Expression::Even(x)
            | Expression::Abs(x)
            | Expression::Vec2E(x)
            | Expression::StringInt(x)
            | Expression::StringFloat(x)
            | Expression::StringVec(x)
            | Expression::IntFloat(x)
            | Expression::SlotUnit(x)
            | Expression::FactionCount(x)
            | Expression::StatusCharges(x) => show_node(
                x.as_mut(),
                format!("{path}:x"),
                connect_pos,
                context,
                ui,
                world,
            ),

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
            | Expression::Vec2EE(a, b)
            | Expression::Or(a, b) => {
                ui.vertical(|ui| {
                    ui.horizontal(|ui| {
                        show_node(
                            a.as_mut(),
                            format!("{path}:a"),
                            connect_pos,
                            context,
                            ui,
                            world,
                        );
                    });
                    ui.horizontal(|ui| {
                        show_node(
                            b.as_mut(),
                            format!("{path}:b"),
                            connect_pos,
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
                        show_node(
                            i.as_mut(),
                            format!("{path}:i"),
                            connect_pos,
                            context,
                            ui,
                            world,
                        );
                    });
                    ui.horizontal(|ui| {
                        show_node(
                            t.as_mut(),
                            format!("{path}:t"),
                            connect_pos,
                            context,
                            ui,
                            world,
                        );
                    });
                    ui.horizontal(|ui| {
                        show_node(
                            e.as_mut(),
                            format!("{path}:e"),
                            connect_pos,
                            context,
                            ui,
                            world,
                        );
                    });
                });
            }
            Expression::WithVar(_, val, e) => {
                ui.vertical(|ui| {
                    ui.horizontal(|ui| {
                        show_node(
                            val.as_mut(),
                            format!("{path}:val"),
                            connect_pos,
                            context,
                            ui,
                            world,
                        );
                    });
                    ui.horizontal(|ui| {
                        show_node(
                            e.as_mut(),
                            format!("{path}:e"),
                            connect_pos,
                            context,
                            ui,
                            world,
                        );
                    });
                });
            }
        };
    }

    fn show_replace_buttons(&mut self, lookup: &str, submit: bool, ui: &mut Ui) -> bool {
        for e in Expression::iter() {
            if e.to_string().to_lowercase().contains(lookup) {
                let btn = e.to_string().add_color(e.node_color()).rich_text(ui);
                let btn = ui.button(btn);
                if btn.clicked() || submit {
                    btn.request_focus();
                }
                if btn.gained_focus() {
                    *self = e.set_inner(self.clone());
                    return true;
                }
            }
        }
        false
    }
}

impl EditorNodeGenerator for Effect {
    fn node_color(&self) -> Color32 {
        white()
    }

    fn show_extra(&mut self, path: &str, context: &Context, world: &mut World, ui: &mut Ui) {
        match self {
            Effect::AoeFaction(_, _)
            | Effect::WithTarget(_, _)
            | Effect::WithOwner(_, _)
            | Effect::Noop
            | Effect::Kill
            | Effect::FullCopy
            | Effect::RemoveLocalTrigger
            | Effect::Debug(_)
            | Effect::Text(_) => {}

            Effect::List(list) | Effect::ListSpread(list) => {
                if ui.button("CLEAR").clicked() {
                    list.clear()
                }
            }
            Effect::Damage(e) => {
                let mut v = e.is_some();
                if ui.checkbox(&mut v, "").changed() {
                    *e = match v {
                        true => Some(default()),
                        false => None,
                    };
                }
            }
            Effect::WithVar(x, e, _) => {
                ui.vertical(|ui| {
                    ComboBox::from_id_source(&path)
                        .selected_text(x.to_string())
                        .show_ui(ui, |ui| {
                            for option in VarName::iter() {
                                let text = option.to_string();
                                ui.selectable_value(x, option, text).changed();
                            }
                        });
                    let value = e.get_value(context, world);
                    show_value(&value, ui);
                });
            }
            Effect::UseAbility(name) => {
                ui.vertical(|ui| {
                    ComboBox::from_id_source(&path)
                        .selected_text(name.to_owned())
                        .show_ui(ui, |ui| {
                            for option in Pools::get(world).abilities.keys() {
                                let text = option.to_string();
                                ui.selectable_value(name, option.to_owned(), text).changed();
                            }
                        });
                });
            }
            Effect::AddStatus(name) => {
                ui.vertical(|ui| {
                    ComboBox::from_id_source(&path)
                        .selected_text(name.to_owned())
                        .show_ui(ui, |ui| {
                            for option in Pools::get(world).statuses.keys() {
                                let text = option.to_string();
                                ui.selectable_value(name, option.to_owned(), text).changed();
                            }
                        });
                });
            }
            Effect::Vfx(name) => {
                ui.vertical(|ui| {
                    ComboBox::from_id_source(&path)
                        .selected_text(name.to_owned())
                        .show_ui(ui, |ui| {
                            for option in Pools::get(world).vfx.keys() {
                                let text = option.to_string();
                                ui.selectable_value(name, option.to_owned(), text).changed();
                            }
                        });
                });
            }
            Effect::SendEvent(name) => {
                ui.vertical(|ui| {
                    ComboBox::from_id_source(&path)
                        .selected_text(name.to_string())
                        .show_ui(ui, |ui| {
                            for option in [Event::BattleStart, Event::TurnStart, Event::TurnEnd] {
                                let text = option.to_string();
                                ui.selectable_value(name, option, text).changed();
                            }
                        });
                });
            }
        }
    }

    fn show_replace_buttons(&mut self, lookup: &str, submit: bool, ui: &mut Ui) -> bool {
        for e in Effect::iter() {
            if e.to_string().to_lowercase().contains(lookup) {
                let btn = e.to_string().add_color(e.node_color()).rich_text(ui);
                let btn = ui.button(btn);
                if btn.clicked() || submit {
                    btn.request_focus();
                }
                if btn.gained_focus() {
                    *self = e;
                    return true;
                }
            }
        }
        false
    }

    fn show_children(
        &mut self,
        path: &str,
        connect_pos: Option<Pos2>,
        context: &Context,
        ui: &mut Ui,
        world: &mut World,
    ) {
        match self {
            Effect::Noop
            | Effect::Kill
            | Effect::FullCopy
            | Effect::UseAbility(_)
            | Effect::AddStatus(_)
            | Effect::Vfx(_)
            | Effect::SendEvent(_)
            | Effect::RemoveLocalTrigger
            | Effect::Debug(_) => {}

            Effect::Text(e) => show_node(e, format!("{path}:e"), connect_pos, context, ui, world),
            Effect::Damage(e) => {
                if let Some(e) = e {
                    show_node(e, format!("{path}:e"), connect_pos, context, ui, world);
                }
            }
            Effect::AoeFaction(e, eff) | Effect::WithTarget(e, eff) | Effect::WithOwner(e, eff) => {
                ui.vertical(|ui| {
                    ui.horizontal(|ui| {
                        show_node(e, format!("{path}:e"), connect_pos, context, ui, world);
                    });
                    ui.horizontal(|ui| {
                        show_node(
                            eff.as_mut(),
                            format!("{path}:eff"),
                            connect_pos,
                            context,
                            ui,
                            world,
                        );
                    });
                });
            }
            Effect::List(list) => {
                ui.vertical(|ui| {
                    for eff in list.iter_mut() {
                        ui.horizontal(|ui| {
                            show_node(
                                eff.as_mut(),
                                format!("{path}:eff"),
                                connect_pos,
                                context,
                                ui,
                                world,
                            );
                        });
                    }
                    if ui.button("+").clicked() {
                        list.push(default());
                    }
                });
            }
            Effect::ListSpread(_) => todo!(),
            Effect::WithVar(_, _, _) => todo!(),
        };
    }
}
