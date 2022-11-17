use std::mem::swap;

use geng::ui::*;
use geng::Draw2d;
use usvg::Point;
pub mod drag_controller;
pub mod render;

use self::drag_controller::DragController;
use self::drag_controller::DragSource;
use self::drag_controller::DragTarget;
use self::drag_controller::Touchable;

use super::*;

use geng::{Camera2d, MouseButton};

const MAX_PARTY: usize = 6;
const UNIT_COST: Money = 3;
const UNIT_SELL_COST: Money = 1;
const REROLL_COST: Money = 1;
const TIER_UP_COST: [Money; 5] = [5, 7, 8, 9, 10];
const TIER_UNITS: [usize; 6] = [3, 4, 4, 5, 5, 6];
const CLAN_BONUS_ACTIVATION_SIZE: [usize; 3] = [2, 4, 5];

pub type Money = u32;

#[derive(Clone)]
pub struct Shop {
    pub tier: Tier,
    pub money: Money,
    pub reroll: bool,
    pub stock: Vec<(UnitType, UnitTemplate)>,
    pub case: Vec<Unit>,
    pub drag_controller: DragController<Unit>,
    pub dirty: bool,
    pub toggle_clans_info: bool,
}

impl Shop {
    pub fn new(tier: usize, templates: &UnitTemplates) -> Self {
        let stock = templates
            .iter()
            .filter(|unit| unit.1.tier > 0 && unit.1.tier <= tier)
            .map(|(name, unit)| (name.clone(), unit.clone()))
            .collect_vec();

        Self {
            tier,
            money: 10,
            stock,
            drag_controller: DragController::new(),
            case: vec![],
            reroll: true,
            dirty: true,
            toggle_clans_info: false,
        }
    }

    pub fn refresh(&mut self, next_id: &mut Id, statuses: &Statuses) {
        if !self.reroll {
            return;
        }
        self.reroll = false;
        let mut case = vec![];
        for i in 0..TIER_UNITS[self.tier - 1 as usize] {
            let unit = &self
                .stock
                .choose(&mut global_rng())
                .expect("No units for shop")
                .1;
            let mut unit = Unit::new(
                unit,
                *next_id,
                Position {
                    side: Faction::Enemy,
                    x: i as i64,
                },
                statuses,
            );
            debug!("Shop refresh {}#{}", unit.unit_type, unit.id);
            unit.render.render_position = unit.position.to_world();

            case.push(unit);
            *next_id += 1;
        }
        self.dirty = true;
        self.case = case;
    }

    pub fn draw(&mut self, render: &Render, framebuffer: &mut ugli::Framebuffer, game_time: f32) {
        if let Some(drag) = &self.drag_controller.drag_target {
            render.draw_unit(&drag, game_time, framebuffer);
        };
    }

    pub fn ui<'b>(
        &mut self,
        cx: &'b geng::ui::Controller,
        sound_controller: &'b SoundController,
        transition: &mut bool,
        clan_configs: &HashMap<Clan, ClanConfig>,
        clan_members: &HashMap<Clan, usize>,
    ) -> Option<impl Widget + 'b> {
        let mut col = geng::ui::column![];
        let mut shop_info = geng::ui::column![];
        let mut left = geng::ui::column![];
        let mut right = geng::ui::column![];
        let mut row = geng::ui::row![];

        let reroll = geng::ui::Button::new(cx, "reroll");
        if reroll.was_clicked() {
            sound_controller.click();
            self.reroll(false);
        }
        let go = geng::ui::Button::new(cx, "Go");
        if go.was_clicked() {
            sound_controller.click();
            *transition = true;
        }
        let text_color = Rgba::BLACK;
        let button_color = Rgba::try_from("#aabbff").unwrap();
        let text = format!("Tier {}", self.tier);
        let tier = geng::ui::Text::new(text, cx.geng().default_font(), 120.0, text_color);

        let text = if self.money == 1 { "coin" } else { "coins" };
        let text = format!("{} {}", self.money, text);
        let coins = geng::ui::Text::new(text, cx.geng().default_font(), 60.0, text_color);

        let mut clans_info = geng::ui::column![];
        let clans_info_button = Button::new(cx, "Clans Info");
        if clans_info_button.was_clicked() {
            self.toggle_clans_info = !self.toggle_clans_info;
        }
        if self.toggle_clans_info {
            for (clan, config) in clan_configs.iter() {
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
        }

        left.push(
            reroll
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

        shop_info.push(
            clans_info_button
                .background_color(button_color)
                .flex_align(vec2(None, None), vec2(0.0, 1.0))
                .uniform_padding(16.0)
                .uniform_padding(16.0)
                .boxed(),
        );
        shop_info.push(
            clans_info
                .flex_align(vec2(Some(1.0), Some(1.0)), vec2(0.0, 1.0))
                .padding_top(32.0)
                .padding_left(32.0)
                .boxed(),
        );
        shop_info.push(tier.boxed());
        shop_info.push(coins.boxed());
        col.push(shop_info.boxed());
        col.push(row.boxed());
        Some(col.uniform_padding(30.0))
    }

    pub fn handle_event(
        &mut self,
        sound_controller: &SoundController,
        render: &Render,
        event: geng::Event,
        team: &mut Vec<Unit>,
    ) {
        match event {
            geng::Event::MouseDown {
                position,
                button: MouseButton::Left,
            } => {
                let position = position.map(|x| x as f32);
                let mouse_world_pos = render
                    .camera
                    .screen_to_world(render.framebuffer_size, position.map(|x| x as f32));
                let mut id = -1;
                for unit in self.case.iter().chain(team.iter()) {
                    if unit.is_touched(mouse_world_pos) {
                        id = unit.id;
                        break;
                    }
                }
                if id >= 0 {
                    if let Some((ind, _)) = team.iter().find_position(|u| u.id == id) {
                        self.drag_controller.source = DragSource::Team;
                        let unit = team.remove(ind);
                        self.drag_controller.drag_target = Some(unit);
                        sound_controller.click();
                    } else if self.money >= UNIT_COST {
                        if let Some((ind, _)) = self.case.iter().find_position(|u| u.id == id) {
                            self.drag_controller.source = DragSource::Shop;
                            let unit = self.case.remove(ind);
                            self.drag_controller.drag_target = Some(unit);
                            sound_controller.click();
                        }
                    }
                }
                self.dirty = true;
            }
            geng::Event::MouseUp {
                position,
                button: MouseButton::Left,
            } => {
                if let Some(mut drag) = self.drag_controller.drag_target.take() {
                    let mut dropped = false;
                    if self.world_position(render, position).x > r32(0.0)
                        && self.drag_controller.source == DragSource::Team
                    {
                        sound_controller.sell();
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
                            let mut unit_in_slot =
                                team.iter_mut().find(|unit| unit.position.x == i as i64);
                            match unit_in_slot {
                                Some(unit_in_slot) => {
                                    if unit_in_slot.merge(drag.clone()) {
                                        dropped = true;
                                        sound_controller.merge();
                                    } else if self.drag_controller.source == DragSource::Team {
                                        let pos = drag.position.clone();
                                        drag.position = unit_in_slot.position.clone();
                                        unit_in_slot.position = pos.clone();
                                        drag.drag(drag.position.to_world());
                                        unit_in_slot.drag(unit_in_slot.position.to_world());
                                    }
                                }
                                None => {
                                    drag.position = Position {
                                        side: Faction::Player,
                                        x: i as i64,
                                    };
                                    drag.faction = Faction::Player;
                                    drag.drag(drag.position.to_world());
                                    team.push(drag.clone());
                                    dropped = true;
                                }
                            }
                            if dropped {
                                if self.drag_controller.source == DragSource::Shop {
                                    sound_controller.buy();
                                    self.money -= UNIT_COST;
                                }
                            }
                            break;
                        }
                    }
                    if !dropped {
                        drag.restore();
                        match self.drag_controller.source {
                            DragSource::Team => team.push(drag),
                            DragSource::Shop => self.case.push(drag),
                        }
                    }
                }
                self.dirty = true;
            }
            geng::Event::MouseMove { position, .. } => {
                if let Some(mut drag) = self.drag_controller.drag_target.take() {
                    drag.drag(self.world_position(render, position));
                    self.drag_controller.drag_target = Some(drag);
                }
            }
            _ => {}
        }
    }

    pub fn world_position(&self, render: &Render, position: Vec2<f64>) -> Vec2<R32> {
        let position = position.map(|x| x as f32);
        render
            .camera
            .screen_to_world(render.framebuffer_size, position.map(|x| x as f32))
            .map(|x| r32(x))
    }

    /// Rerolls the shop units. If `force` is true, then the cost is not paid.
    pub fn reroll(&mut self, force: bool) {
        if self.money >= REROLL_COST || force {
            self.money -= REROLL_COST;
            self.reroll = true;
        }
    }
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
