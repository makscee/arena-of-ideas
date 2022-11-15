use geng::ui::*;
use geng::Draw2d;
use usvg::Point;
pub mod drag_controller;
pub mod render;

use self::drag_controller::DragController;
use self::drag_controller::DragSource;
use self::drag_controller::DragTarget;
use self::drag_controller::Touchable;
use crate::render::UnitRender;

use super::*;

use geng::{Camera2d, MouseButton};

const MAX_PARTY: usize = 6;
const MAX_INVENTORY: usize = 7;
const UNIT_COST: Money = 3;
const UNIT_SELL_COST: Money = 1;
const REROLL_COST: Money = 1;
const TIER_UP_COST: [Money; 5] = [5, 7, 8, 9, 10];
const TIER_UNITS: [usize; 6] = [3, 4, 4, 5, 5, 6];
const CLAN_BONUS_ACTIVATION_SIZE: [usize; 3] = [2, 4, 5];

pub type Money = u32;

pub struct Shop {
    pub statuses: Statuses,
    pub round: usize,
    pub tier: Tier,
    /// The number of rounds that the shop has not been upgraded to the next tier.
    /// Once the shop is tiered up, that number is reset to 0.
    pub tier_rounds: usize,
    pub money: Money,
    pub available: Vec<(UnitType, UnitTemplate)>,
    pub drag_controller: DragController<Unit>,
    pub camera: Camera2d,
    pub framebuffer_size: Vec2<f32>,
    pub lives: i32,
    pub updated: bool,
    pub frozen: bool,
    pub unit_hovered: bool,
    pub clan_configs: HashMap<Clan, ClanConfig>,
    pub model: Model,
}

impl Shop {
    pub fn new(assets: &Rc<Assets>, camera: Camera2d, model: Model, tier: u32) -> Self {
        let units = assets
            .units
            .iter()
            .filter(|unit| unit.1.tier > 0)
            .map(|(name, unit)| (name.clone(), unit.clone()))
            .collect();

        Self {
            statuses: assets.statuses.clone(),
            round: 0,
            tier,
            tier_rounds: 0,
            money: 10,
            available: units,
            lives: MAX_LIVES,
            framebuffer_size: Vec2::ZERO,
            camera,
            drag_controller: DragController::new(),
            updated: true,
            frozen: false,
            unit_hovered: false,
            clan_configs: assets.options.clan_configs.clone(),
            model,
        }
    }

    pub fn tier_up(&mut self) {
        if let Some(cost) = tier_up_cost(self.tier, self.tier_rounds) {
            if self.money >= cost {
                self.tier += 1;
                self.tier_rounds = 0;
                self.money -= cost;
            }
        }
    }

    pub fn freeze(&mut self) {
        self.frozen = !self.frozen;
    }

    pub fn draw(
        &mut self,
        geng: &Geng,
        assets: &Rc<Assets>,
        game_time: f32,
        framebuffer: &mut ugli::Framebuffer,
    ) {
        let unit_render = UnitRender::new(&geng, assets);
        self.framebuffer_size = framebuffer.size().map(|x| x as f32);
        if let Some(drag) = &self.drag_controller.drag_target {
            unit_render.draw_unit(&drag, None, game_time, &self.camera, framebuffer);
        };
    }

    pub fn ui<'b>(&mut self, cx: &'b geng::ui::Controller) -> Option<impl Widget + 'b> {
        let mut col = geng::ui::column![];
        let mut shop_info = geng::ui::column![];
        let mut left = geng::ui::column![];
        let mut right = geng::ui::column![];
        let mut row = geng::ui::row![];

        let reroll = geng::ui::Button::new(cx, "reroll");
        if reroll.was_clicked() {
            self.reroll(false);
        }
        let freeze = geng::ui::Button::new(cx, "freeze");
        if freeze.was_clicked() {
            self.freeze();
        }
        let go = geng::ui::Button::new(cx, "Go");
        if go.was_clicked() {
            self.model.transition = true;
            self.updated = true;
        }
        let text_color = Rgba::BLACK;
        let button_color = Rgba::try_from("#aabbff").unwrap();
        let text = format!("Tier {}", self.tier);
        let tier = geng::ui::Text::new(text, cx.geng().default_font(), 120.0, text_color);

        let text = if self.money == 1 { "coin" } else { "coins" };
        let text = format!("{} {}", self.money, text);
        let coins = geng::ui::Text::new(text, cx.geng().default_font(), 60.0, text_color);

        let mut clans_info = geng::ui::column![];
        let team: Vec<&Unit> = self
            .model
            .units
            .iter()
            .filter(|unit| unit.faction == Faction::Player)
            .collect();
        let clan_members = calc_clan_members(team);
        for (clan, config) in self.clan_configs.iter() {
            if config.description.len() < 3 {
                continue;
            }
            let members = *clan_members.get(clan).unwrap_or(&0);
            let mut descriptions = geng::ui::column![];
            descriptions.push(
                Text::new(
                    config.ability.clone(),
                    cx.geng().default_font(),
                    40.0,
                    config.color,
                )
                .boxed(),
            );
            for (i, text) in config.description.iter().enumerate() {
                let activation_size = CLAN_BONUS_ACTIVATION_SIZE[i];
                let color: Rgba<f32> = match activation_size <= members {
                    true => config.color,
                    false => Rgba::try_from("#818181").unwrap(),
                };
                descriptions.push(
                    Text::new(
                        format!("({}) {}", activation_size, text),
                        cx.geng().default_font(),
                        30.0,
                        color,
                    )
                    .boxed(),
                )
            }
            clans_info.push(
                (
                    Text::new(
                        clan.to_string(),
                        cx.geng().default_font(),
                        40.0,
                        Rgba::WHITE,
                    )
                    .padding_horizontal(16.0)
                    .center()
                    .background_color(config.color)
                    .fixed_size(vec2(200.0, 0.0)),
                    descriptions.padding_left(16.0),
                )
                    .row()
                    .boxed(),
            )
        }

        left.push(
            reroll
                .uniform_padding(16.0)
                .background_color(button_color)
                .uniform_padding(16.0)
                .boxed(),
        );
        left.push(
            freeze
                .uniform_padding(16.0)
                .background_color(button_color)
                .uniform_padding(16.0)
                .boxed(),
        );
        right.push(
            go.fixed_size(vec2(128.0, 128.0))
                .uniform_padding(16.0)
                .background_color(button_color)
                .boxed(),
        );
        row.push(left.boxed());
        row.push(
            right
                .flex_align(
                    Vec2 {
                        x: Some(1.0),
                        y: Some(0.0),
                    },
                    Vec2 { x: 1.0, y: 0.0 },
                )
                .boxed(),
        );

        if !self.unit_hovered {
            shop_info.push(
                clans_info
                    .flex_align(vec2(Some(1.0), Some(1.0)), vec2(0.0, 1.0))
                    .padding_top(100.0)
                    .padding_left(32.0)
                    .boxed(),
            );
        }
        shop_info.push(tier.boxed());
        shop_info.push(coins.boxed());
        col.push(shop_info.boxed());
        col.push(row.boxed());
        Some(col.uniform_padding(30.0))
    }

    pub fn handle_event(&mut self, event: geng::Event) {
        match event {
            geng::Event::MouseDown {
                position,
                button: MouseButton::Left,
            } => {
                let position = position.map(|x| x as f32);
                let mouse_world_pos = self.camera.screen_to_world(
                    self.framebuffer_size.map(|x| x as f32),
                    position.map(|x| x as f32),
                );
                let mut id = -1;
                for unit in &self.model.units {
                    if unit.is_touched(mouse_world_pos) {
                        id = unit.id;
                        self.drag_controller.source = if unit.faction == Faction::Player {
                            DragSource::Team
                        } else {
                            DragSource::Shop
                        };
                        break;
                    }
                }
                if id >= 0 {
                    if self.drag_controller.source == DragSource::Shop {
                        if self.money >= UNIT_COST {
                            self.drag_controller.drag_target =
                                self.model.units.remove(&(id as i64));
                        }
                    } else {
                        self.drag_controller.drag_target = self.model.units.remove(&(id as i64));
                    }
                    self.updated = true;
                }
            }
            geng::Event::MouseUp {
                position,
                button: MouseButton::Left,
            } => {
                if let Some(mut drag) = self.drag_controller.drag_target.take() {
                    let mut dropped = false;
                    if self.world_position(position).x > r32(0.0)
                        && self.drag_controller.source == DragSource::Team
                    {
                        self.money += UNIT_SELL_COST;
                        return;
                    }
                    for i in 0..SIDE_SLOTS {
                        let pos = -(i as f32);
                        let slot_box = AABB::point(Vec2 {
                            x: pos - 1.0,
                            y: 0.0,
                        })
                        .extend_uniform(drag.render.radius.as_f32() * 2.0);
                        if slot_box.contains(drag.position()) {
                            dropped = true;
                            let unit_in_slot = self.model.units.iter_mut().find(|unit| {
                                unit.faction == Faction::Player && unit.position.x == i as i64
                            });
                            match unit_in_slot {
                                Some(unit_in_slot) => {
                                    dropped = unit_in_slot.merge(drag.clone());
                                }
                                None => {
                                    drag.position = Position {
                                        side: Faction::Player,
                                        x: i as i64,
                                    };
                                    drag.faction = Faction::Player;
                                    drag.drag(drag.position.to_world());
                                    self.model.units.insert(drag.clone());
                                }
                            }
                            if dropped {
                                if self.drag_controller.source == DragSource::Shop {
                                    self.money -= UNIT_COST;
                                }
                            }
                            self.updated = true;
                            break;
                        }
                    }
                    if !dropped {
                        drag.restore();
                        self.model.units.insert(drag);
                        self.updated = true;
                    }
                }
            }
            geng::Event::MouseMove { position, .. } => {
                if let Some(mut drag) = self.drag_controller.drag_target.take() {
                    drag.drag(self.world_position(position));
                    self.drag_controller.drag_target = Some(drag);
                }
            }
            _ => {}
        }
    }

    pub fn world_position(&self, position: Vec2<f64>) -> Vec2<R32> {
        let position = position.map(|x| x as f32);
        self.camera
            .screen_to_world(
                self.framebuffer_size.map(|x| x as f32),
                position.map(|x| x as f32),
            )
            .map(|x| r32(x))
    }

    /// Rerolls the shop units. If `force` is true, then the cost is not paid.
    pub fn reroll(&mut self, force: bool) {
        if self.frozen && force {
            self.frozen = false;
            return;
        }
        if self.money >= REROLL_COST || force {
            if !force {
                self.money -= REROLL_COST;
            }
            self.model
                .units
                .retain(|unit| unit.faction == Faction::Player);
            if let Some(count) = tier_units_number(self.tier) {
                let mut rng = global_rng();
                let options = self
                    .available
                    .iter()
                    .filter(|(_, unit)| unit.tier <= self.tier)
                    .map(|(name, template)| {
                        Unit::new(
                            &template.clone(),
                            0,
                            Position::zero(Faction::Enemy),
                            &Statuses { map: hashmap! {} },
                        )
                    })
                    .collect::<Vec<_>>();
                if options.is_empty() {
                    error!("No units are available to roll");
                    return;
                }
                let units: Vec<Unit> = (0..count)
                    .map(|_| options.choose(&mut rng).unwrap().clone())
                    .collect();
                for (index, unit) in units.iter().enumerate() {
                    let mut cloned = unit.clone();
                    let position = Position {
                        side: Faction::Enemy,
                        x: index.try_into().unwrap(),
                    };
                    cloned.render.render_position = position.to_world();
                    cloned.faction = position.side;
                    cloned.position = position.clone();
                    cloned.id = self.model.next_id;
                    self.model.units.insert(cloned);
                    self.model.next_id += 1;
                }
            }
            self.frozen = false;
            self.updated = true;
        }
    }
}

pub fn calc_clan_members<'a>(units: impl IntoIterator<Item = &'a Unit>) -> HashMap<Clan, usize> {
    let unique_units = units
        .into_iter()
        .map(|unit| (&unit.unit_type, &unit.clans))
        .collect::<HashMap<_, _>>();
    let mut clans = HashMap::new();
    for clan in unique_units.into_values().flatten() {
        *clans.entry(*clan).or_insert(0) += 1;
    }
    clans
}

fn earn_money(round: usize) -> Money {
    (4 + round).min(10) as _
}

fn tier_up_cost(current_tier: Tier, tier_rounds: usize) -> Option<Money> {
    TIER_UP_COST
        .get(current_tier as usize - 1)
        .map(|&cost| cost.saturating_sub(tier_rounds as Money))
}

fn tier_units_number(current_tier: Tier) -> Option<usize> {
    TIER_UNITS.get(current_tier as usize - 1).copied()
}
