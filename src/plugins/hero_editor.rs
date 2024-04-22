use bevy_egui::egui::{Frame, Key, ScrollArea, SelectableLabel, Sense, SidePanel};
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
    pub fn load_unit(unit: PackedUnit, world: &mut World) {
        let mut pd = PersistentData::load(world);
        let ed = &mut pd.hero_editor_data;
        ed.saved_teams = ed.teams.clone();
        ed.clear();
        ed.teams.0 = [unit.clone()].into();
        ed.active = Some((Faction::Left, 1, unit));
        pd.save(world).unwrap();
    }

    fn on_enter(world: &mut World) {
        let pd = PersistentData::load(world);
        pd.hero_editor_data.load(world);
        world.insert_resource(HeroEditorHistory {
            frames: vec![(0.0, pd.hero_editor_data.clone())],
            ind: default(),
        });
        pd.save(world).unwrap();
        Pools::get_mut(world).only_local_cache = true;
    }

    fn on_exit(world: &mut World) {
        Self::save(world);
        UnitPlugin::despawn_all_teams(world);
    }

    fn update(world: &mut World) {
        let mut pd = PersistentData::load(world);
        let ed = &mut pd.hero_editor_data;
        let pos = if let Some((faction, slot, _)) = ed.active {
            UnitPlugin::get_slot_position(faction, slot)
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
        let input = world.resource::<ButtonInput<KeyCode>>();
        if input.just_pressed(KeyCode::ArrowUp) {
            ed.camera_scale *= 1.2;
            pd.save(world).unwrap();
        } else if input.just_pressed(KeyCode::ArrowDown) {
            ed.camera_scale /= 1.2;
            pd.save(world).unwrap();
        } else if input.pressed(KeyCode::SuperLeft) && input.pressed(KeyCode::ControlLeft) {
            if input.just_pressed(KeyCode::KeyZ) {
                if input.pressed(KeyCode::ShiftLeft) {
                    HeroEditorHistory::redo(world);
                } else {
                    HeroEditorHistory::undo(world);
                }
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
        let ctx = &if let Some(context) = egui_context(world) {
            context
        } else {
            return;
        };
        let mut pd = PersistentData::load(world);
        let ed = &mut pd.hero_editor_data.clone();

        let hovered = UnitPlugin::get_hovered(world);
        let mut delete: Option<Entity> = None;
        for unit in UnitPlugin::collect_all(world) {
            let hovered = hovered == Some(unit);
            if hovered {
                entity_window(unit, vec2(0.0, 0.0), None, &format!("{unit:?}"), world)
                    .frame(Frame::none())
                    .show(ctx, |ui| {
                        let button = ui.button("Edit");
                        if button.clicked() {
                            let state = VarState::get(unit, world);
                            let faction = state
                                .parent(world)
                                .unwrap()
                                .get_faction(VarName::Faction)
                                .unwrap();
                            let slot = state.get_int(VarName::Slot).unwrap();
                            ed.active =
                                Some((faction, slot as usize, PackedUnit::pack(unit, world)));
                        }
                        ui.add_space(5.0);
                        if ui.button_red("Delete").clicked() {
                            delete = Some(unit);
                        }
                    });
            }
        }
        if let Some(unit) = delete {
            world.entity_mut(unit).despawn_recursive();
            UnitPlugin::fill_gaps_and_place(world);
            ed.save(world);
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
                        if ui.button("Spawn").clicked() {
                            ed.spawn(faction, world);
                        }
                    });
            }
        }
        Self::draw_top_buttons(ed, ctx, world);
        TopBottomPanel::new(egui::panel::TopBottomSide::Bottom, "bot btns").show(ctx, |ui| {
            ui.with_layout(Layout::right_to_left(egui::Align::Center), |ui| {
                if ui.button("Table").clicked() {
                    GameState::HeroTable.change(world);
                }
            });
        });
        if !pd.hero_editor_data.eq(ed) {
            ed.save(world);
            mem::swap(&mut pd.hero_editor_data, ed);
            pd.save(world).unwrap();
        }
    }

    fn draw_top_buttons(ed: &mut HeroEditorData, ctx: &egui::Context, world: &mut World) {
        if ed.active.is_some() {
            return;
        }
        TopBottomPanel::top("battle btns").show(ctx, |ui| {
            ui.horizontal(|ui| {
                if ui.button("Turn End").clicked() {
                    Event::TurnEnd.send(world);
                }
                if ui.button("Battle Start").clicked() {
                    Event::BattleStart.send(world);
                }
                if ui.button("Strike").clicked() {
                    if let Some((left, right)) = BattlePlugin::get_strikers(world) {
                        match BattlePlugin::run_strike(left, right, world) {
                            Ok(_) => {}
                            Err(e) => error!("{e}"),
                        }
                    }
                }

                ui.add_space(10.0);
                if ui.button_color("Save", yellow()).clicked() {
                    ed.saved_teams = ed.teams.clone();
                }
                if ui.button_color("Load", yellow()).clicked() {
                    ed.teams = ed.saved_teams.clone();
                    ed.load(world);
                }

                ui.add_space(10.0);
                if ui.button("Fuse").clicked() {
                    let fused =
                        PackedUnit::fuse(ed.teams.0[0].clone(), ed.teams.0[1].clone(), world)
                            .remove(0);
                    ed.teams.0.remove(0);
                    ed.teams.0[0] = fused;
                    ed.load(world);
                }

                ui.add_space(10.0);
                if ui.button_red("Clear Statuses").clicked() {
                    for unit in ed.teams.0.iter_mut().chain(ed.teams.1.iter_mut()) {
                        unit.statuses.clear();
                    }
                    ed.load(world);
                }
                if ui.button_red("Reset").clicked() {
                    UnitPlugin::despawn_all_teams(world);
                    ed.load(world);
                }
                if ui.button_red("Clear").clicked() {
                    Self::clear(world);
                }
            });
        });
    }

    fn show_edit_panel(ed: &mut HeroEditorData, world: &mut World) {
        if let Some((faction, slot, old_unit)) = ed.active.as_ref() {
            let ctx = &if let Some(context) = egui_context(world) {
                context
            } else {
                return;
            };
            let mut unit = old_unit.clone();
            let entity = UnitPlugin::find_unit(*faction, *slot, world).unwrap();

            SidePanel::left("edit panel")
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
                        if ui.button_red("Close").clicked() {
                            ed.active = None;
                        }
                        if ui.button("Paste").clicked() {
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
                        if ui.button("Copy").clicked() {
                            let mut unit = unit.clone();
                            unit.state = default();
                            save_to_clipboard(
                                &to_string_pretty(&unit, PrettyConfig::new()).unwrap(),
                                world,
                            );
                        }
                        ui.add_space(10.0);
                        const SELECTED_STATUS_KEY: &str = "selected_status";
                        let mut status = get_context_string(world, SELECTED_STATUS_KEY);

                        if let Some(option) =
                            Status::show_selector(&mut status, "apply status selector", ui, world)
                        {
                            set_context_string(world, SELECTED_STATUS_KEY, option);
                        }
                        ui.add_enabled_ui(!status.is_empty(), |ui| {
                            if ui.button("Add Status").clicked() {
                                if let Some((i, _)) =
                                    unit.statuses.iter().find_position(|(s, _)| status.eq(s))
                                {
                                    unit.statuses[i].1 += 1;
                                } else {
                                    unit.statuses.push((status, 1));
                                }
                            }
                        });

                        ui.add_space(10.0);
                        const LOAD_HERO_KEY: &str = "load_hero";
                        let mut hero = get_context_string(world, LOAD_HERO_KEY);
                        let heroes = Pools::get(world)
                            .heroes
                            .iter()
                            .sorted_by_key(|(_, h)| &h.houses)
                            .map(|(k, _)| k.clone())
                            .collect_vec();

                        if ui.button("Next").clicked() {
                            let p = heroes.iter().position(|h| hero.eq(h)).unwrap_or_default();
                            hero = heroes.get((p + 1) % heroes.len()).unwrap().clone();
                            set_context_string(world, LOAD_HERO_KEY, hero.clone());
                            unit = Pools::get(world).heroes.get(&hero).unwrap().clone();
                        }
                        ComboBox::from_id_source(LOAD_HERO_KEY)
                            .selected_text(hero.clone())
                            .show_ui(ui, |ui| {
                                for option in heroes {
                                    let text = option.to_string();

                                    if SelectableLabel::new(
                                        hero.eq(&option),
                                        text.add_color(
                                            Pools::get_color_by_name(&option, world).unwrap().c32(),
                                        )
                                        .rich_text(ui),
                                    )
                                    .ui(ui)
                                    .clicked()
                                    {
                                        unit =
                                            Pools::get(world).heroes.get(&option).unwrap().clone();
                                        set_context_string(world, LOAD_HERO_KEY, option);
                                    }
                                }
                            });
                        if ui.button("Load").clicked() {
                            unit = Pools::get(world).heroes.get(&hero).unwrap().clone();
                        }

                        ui.add_space(10.0);
                        let mut sd = SettingsData::get(world).clone();
                        let card = &mut sd.always_show_card;
                        if ui.checkbox(card, "").changed() {
                            sd.save(world).unwrap();
                        }
                        ui.label("card:");

                        ui.add_space(10.0);
                        ui.add_enabled_ui(HeroEditorHistory::get_mut(world).can_redo(), |ui| {
                            if ui.button("->").clicked() {
                                HeroEditorHistory::redo(world);
                            }
                        });
                        ui.add_enabled_ui(HeroEditorHistory::get_mut(world).can_undo(), |ui| {
                            if ui.button("<-").clicked() {
                                HeroEditorHistory::undo(world);
                            }
                        })
                    });
                    ScrollArea::new([true, true])
                        .scroll_bar_visibility(egui::scroll_area::ScrollBarVisibility::AlwaysHidden)
                        .show(ui, |ui| {
                            unit.show_editor(entity, ui, world);
                        });
                });

            if let Some((faction, slot, old_unit)) = ed.active.as_ref() {
                if !unit.eq(old_unit) {
                    let entity = UnitPlugin::find_unit(*faction, *slot, world).unwrap();
                    let parent = entity.get_parent(world).unwrap();
                    world.entity_mut(entity).despawn_recursive();
                    let entity = unit.clone().unpack(parent, Some(*slot), world);
                    UnitPlugin::place_into_slot(entity, world).unwrap();
                    ed.active = Some((*faction, *slot, unit));
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

#[derive(Serialize, Deserialize, Resource, Default)]
struct HeroEditorHistory {
    frames: Vec<(f32, HeroEditorData)>,
    ind: usize,
}

impl HeroEditorHistory {
    fn get_mut(world: &mut World) -> Mut<Self> {
        world.resource_mut::<HeroEditorHistory>()
    }
    fn push(ed: HeroEditorData, world: &mut World) {
        const CD: f32 = 0.5;
        const LIMIT: usize = 100;
        let ts = world.resource::<Time>().elapsed_seconds();
        let mut heh = Self::get_mut(world);
        if heh.frames.last().is_some_and(|(t, _)| ts - *t < CD) {
            return;
        }
        debug!("Push frame, total frames: {}", heh.frames.len());
        heh.frames.push((ts, ed));
        heh.ind = 0;
        if heh.frames.len() > LIMIT {
            heh.frames.remove(0);
        }
    }
    fn can_undo(&self) -> bool {
        self.frames.len() > self.ind + 1
    }
    fn can_redo(&self) -> bool {
        self.ind > 0
    }
    fn undo(world: &mut World) {
        let mut pd = PersistentData::load(world);
        let mut heh = Self::get_mut(world);
        if !heh.can_undo() {
            return;
        }
        heh.ind += 1;
        debug!("Undo {}, total frames: {}", heh.ind, heh.frames.len());
        if let Some((_, data)) = heh.frames.get(heh.frames.len() - heh.ind - 1) {
            let data = data.clone();
            data.load(world);
            pd.hero_editor_data = data;
            pd.save(world).unwrap();
        }
    }
    fn redo(world: &mut World) {
        let mut pd = PersistentData::load(world);
        let mut heh = Self::get_mut(world);
        if !heh.can_redo() {
            return;
        }
        heh.ind -= 1;
        debug!("Redo {}, total frames: {}", heh.ind, heh.frames.len());
        if let Some((_, data)) = heh.frames.get(heh.frames.len() - heh.ind - 1) {
            let data = data.clone();
            data.load(world);
            pd.hero_editor_data = data;
            pd.save(world).unwrap();
        }
    }
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct HeroEditorData {
    pub active: Option<(Faction, usize, PackedUnit)>,

    pub teams: (Vec<PackedUnit>, Vec<PackedUnit>),
    pub camera_pos: Vec2,
    pub camera_need_pos: Vec2,
    pub camera_scale: f32,

    pub saved_teams: (Vec<PackedUnit>, Vec<PackedUnit>),
}

impl Default for HeroEditorData {
    fn default() -> Self {
        Self {
            camera_pos: default(),
            camera_need_pos: default(),
            camera_scale: 1.0,
            active: default(),
            teams: default(),
            saved_teams: default(),
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
        HeroEditorHistory::push(self.clone(), world);
    }

    fn load(&self, world: &mut World) {
        debug!("Load hero editor data start");
        UnitPlugin::despawn_all_teams(world);
        let left = TeamPlugin::spawn(Faction::Left, world);
        let right = TeamPlugin::spawn(Faction::Right, world);
        self.teams.0.iter().for_each(|u| {
            u.clone().unpack(left, None, world);
        });
        self.teams.1.iter().for_each(|u| {
            u.clone().unpack(right, None, world);
        });
        UnitPlugin::fill_gaps_and_place(world);
    }

    fn clear(&mut self) {
        self.active = None;
        self.teams.0.clear();
        self.teams.1.clear();
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
            TeamPlugin::find_entity(faction, world).unwrap(),
            None,
            world,
        );
        UnitPlugin::fill_slot_gaps(faction, world);
        UnitPlugin::translate_to_slots(world);
        self.save(world);
    }
}

pub fn show_value(value: &Result<VarValue>, ui: &mut Ui) {
    match &value {
        Ok(v) => v.to_string().add_color(light_gray()),

        Err(e) => e.to_string().add_color(red()),
    }
    .set_style_ref(ColoredStringStyle::Small)
    .as_label(ui)
    .truncate(true)
    .ui(ui);
}

pub fn show_trees_desc(
    label: &str,
    roots: &mut Vec<(impl EditorNodeGenerator, Option<String>)>,
    context: &Context,
    ui: &mut Ui,
    world: &mut World,
) {
    ui.vertical(|ui| {
        let mut delete = None;
        for (i, (node, d)) in roots.iter_mut().enumerate() {
            ui.horizontal(|ui| {
                if ui.button_red("-").clicked() {
                    delete = Some(i);
                }
                let mut c = d.is_some();
                if ui.checkbox(&mut c, "").changed() {
                    if c {
                        *d = Some(default());
                    } else {
                        *d = None;
                    }
                }
                if let Some(d) = d {
                    ui.add_sized([100.0, 20.0], TextEdit::singleline(d));
                }
            });
            show_tree(&i.to_string(), node, context, ui, world);
        }
        if let Some(delete) = delete {
            roots.remove(delete);
        }
        if ui.button("+").clicked() {
            roots.push((default(), None));
        }
    });
}

pub fn show_tree(
    label: &str,
    root: &mut impl EditorNodeGenerator,
    context: &Context,
    ui: &mut Ui,
    world: &mut World,
) {
    show_trees([(label, root)].into(), context, ui, world);
}

pub fn show_trees(
    data: Vec<(&str, &mut impl EditorNodeGenerator)>,
    context: &Context,
    ui: &mut Ui,
    world: &mut World,
) {
    ui.with_layout(Layout::left_to_right(egui::Align::Min), |ui| {
        for (label, root) in data {
            ui.label(label);
            show_node(root, label.to_owned(), None, context, ui, world);
        }
    });
}

pub fn show_node(
    source: &mut impl EditorNodeGenerator,
    path: String,
    connect_pos: Option<Pos2>,
    context: &Context,
    ui: &mut Ui,
    world: &mut World,
) {
    let ctx = &if let Some(context) = egui_context(world) {
        context
    } else {
        return;
    };
    let path = format!("{path}/{}", source.as_ref());
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
                .as_ref()
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

    {
        let mut left_line = frame_resp.rect.translate(egui::vec2(3.0, 0.0));
        left_line.set_width(2.0);
        left_line = left_line.shrink2(egui::vec2(0.0, 14.0));
        let mut ui = ui.child_ui(left_line, Layout::left_to_right(egui::Align::Center));
        let response = ui.allocate_rect(left_line, Sense::click());
        let color = if response.hovered() {
            yellow()
        } else {
            dark_gray()
        };
        ui.painter_at(left_line)
            .rect_filled(left_line, Rounding::ZERO, color);
        if response.clicked() {
            source.wrap();
        }
    }

    const LOOKUP_KEY: &str = "lookup";
    const OPEN_KEY: &str = "replace_window";
    if get_context_string(world, OPEN_KEY).eq(&path) {
        if name_resp.clicked_elsewhere() {
            set_context_string(world, OPEN_KEY, default());
        }
        window("replace")
            .order(egui::Order::Foreground)
            .title_bar(false)
            .fixed_pos(frame_resp.rect.right_center().to_bvec2())
            .show(ctx, |ui| {
                Frame::none().inner_margin(8.0).show(ui, |ui| {
                    let mut lookup = get_context_string(world, LOOKUP_KEY);
                    let mut submit = false;
                    let mut close = false;
                    ctx.input(|i| {
                        for e in &i.events {
                            match e {
                                egui::Event::Text(t) => lookup += t,
                                egui::Event::Key { key, pressed, .. } => {
                                    if *pressed {
                                        if key.eq(&Key::Backspace) && !lookup.is_empty() {
                                            lookup.pop();
                                        } else if matches!(key, Key::Enter | Key::Tab) {
                                            submit = true;
                                        } else if key.eq(&Key::Escape) {
                                            close = true;
                                        }
                                    }
                                }
                                _ => {}
                            }
                        }
                    });
                    if close {
                        set_context_string(world, OPEN_KEY, default());
                    }
                    ui.label(&lookup);
                    set_context_string(world, LOOKUP_KEY, lookup.clone());
                    ScrollArea::new([false, true])
                        .max_height(300.0)
                        .show(ui, |ui| {
                            let lookup = lookup.to_lowercase();
                            frame(ui, |ui| {
                                source.show_replace_buttons(&lookup, submit, ui);
                            });
                        });
                });
            });
    }
    if name_resp.clicked() {
        set_context_string(world, LOOKUP_KEY, default());
        set_context_string(world, OPEN_KEY, path.clone());
    }

    if let Some(pos) = connect_pos {
        let end = frame_resp.rect.left_center();
        let mut mid1 = pos;
        mid1.x += 5.0;
        let mut mid2 = end;
        mid2.x -= 5.0;
        draw_curve(pos, mid1, mid2, end, 1.0, dark_gray(), false, ui);
    }

    source.show_children(
        &path,
        Some(frame_resp.rect.right_center()),
        context,
        ui,
        world,
    );

    name_resp.context_menu(|ui| {
        if ui.button("Copy").clicked() {
            save_to_clipboard(
                &to_string_pretty(source, PrettyConfig::new()).unwrap(),
                world,
            );
            ui.close_menu();
        }
        if ui.button("Paste").clicked() {
            let o = get_from_clipboard(world).unwrap();
            if let Ok(o) = ron::from_str(o.as_str()) {
                *source = o;
            }
            ui.close_menu();
        }
        source.show_context_menu(ui);
    });
}

pub trait EditorNodeGenerator: AsRef<str> + Sized + Serialize + DeserializeOwned + Default {
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
    fn show_context_menu(&mut self, ui: &mut Ui);
    fn wrap(&mut self);
}
