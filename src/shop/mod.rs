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
    pub units: Vec<Unit>,
    pub team: Vec<Unit>,
    pub drag_controller: DragController<Unit>,
    pub camera: Camera2d,
    pub framebuffer_size: Vec2<f32>,
    pub lives: i32,
    pub enabled: bool,
    pub updated: bool,
    pub frozen: bool,
    pub unit_hovered: bool,
    pub clan_configs: HashMap<Clan, ClanConfig>,
}

impl Shop {
    pub fn new(assets: &Rc<Assets>, camera: Camera2d) -> Self {
        let units = assets
            .units
            .iter()
            .filter(|unit| unit.1.tier > 0)
            .map(|(name, unit)| (name.clone(), unit.clone()))
            .collect();

        Self {
            statuses: assets.statuses.clone(),
            round: 0,
            tier: 1,
            tier_rounds: 0,
            money: 10,
            units: vec![],
            team: vec![],
            available: units,
            lives: MAX_LIVES,
            enabled: true,
            framebuffer_size: Vec2::ZERO,
            camera,
            drag_controller: DragController::new(),
            updated: false,
            frozen: false,
            unit_hovered: false,
            clan_configs: assets.options.clan_configs.clone(),
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
        camera: &Camera2d,
    ) {
        if !self.enabled {
            return;
        };
        let unit_render = UnitRender::new(&geng, assets);
        self.framebuffer_size = framebuffer.size().map(|x| x as f32);
        let mouse_world_pos = camera.screen_to_world(
            self.framebuffer_size,
            geng.window().mouse_pos().map(|x| x as f32),
        );

        if let Some(drag) = &self.drag_controller.drag_target {
            unit_render.draw_unit(&drag, None, game_time, &camera, framebuffer);
        };

        let mut hovered_unit = None;

        for (index, unit) in self.units.iter_mut().enumerate() {
            unit_render.draw_unit(&unit, None, game_time, &camera, framebuffer);
            unit_render.draw_unit_stats(&unit, &camera, framebuffer);
            let tier = assets
                .units
                .get(&unit.unit_type)
                .unwrap_or_else(|| panic!("Failed to find unit {}", unit.unit_type))
                .tier;
            let radius = unit.render.radius.as_f32();
            let unit_aabb =
                AABB::point(unit.render.render_position.map(|x| x.as_f32())).extend_uniform(radius);
            let size = radius * 0.5;
            let damage = AABB::point(unit_aabb.top_left())
                .extend_right(size)
                .extend_up(size)
                .translate(vec2(-0.1, 0.1));
            draw_2d::Quad::new(
                damage.extend_uniform(0.03),
                Rgba::try_from("#6e6e6e").unwrap(),
            )
            .draw_2d(&geng, framebuffer, camera);
            draw_2d::Text::unit(
                geng.default_font().clone(),
                format!("T{}", tier),
                Rgba::WHITE,
            )
            .fit_into(damage)
            .draw_2d(&geng, framebuffer, camera);

            // On unit hover
            if (mouse_world_pos - unit.render.render_position.map(|x| x.as_f32())).len()
                < unit.render.radius.as_f32()
            {
                // Draw extra ui: statuses descriptions, damage/heal descriptions
                hovered_unit = Some(unit.clone());
            }
        }

        // Draw slots
        let factions = vec![Faction::Player, Faction::Enemy];
        let shader_program = &assets.custom_renders.slot;
        for faction in factions {
            for i in 0..SIDE_SLOTS {
                let quad = shader_program.get_vertices(geng);
                let framebuffer_size = framebuffer.size();
                let position = Position {
                    x: i as i64,
                    side: faction,
                }
                .to_world_f32();
                let unit = self
                    .units
                    .iter()
                    .chain(self.team.iter())
                    .find(|unit| unit.position.x == i as i64 && unit.faction == faction);

                let health = match unit {
                    Some(unit) => 1.0,
                    None => 0.0,
                };

                ugli::draw(
                    framebuffer,
                    &shader_program.program,
                    ugli::DrawMode::TriangleStrip,
                    &quad,
                    (
                        ugli::uniforms! {
                            u_time: game_time,
                            u_unit_position: position,
                            u_parent_faction: 1.0,
                            u_health: health,
                        },
                        geng::camera2d_uniforms(&self.camera, framebuffer_size.map(|x| x as f32)),
                        &shader_program.parameters,
                    ),
                    ugli::DrawParameters {
                        blend_mode: Some(ugli::BlendMode::default()),
                        ..default()
                    },
                );
            }
        }
        self.unit_hovered = self.unit_hovered || hovered_unit.is_some();

        if let Some(unit) = hovered_unit {
            unit_render.draw_hover(&unit, camera, framebuffer);
        }
    }

    pub fn ui<'b>(&mut self, cx: &'b geng::ui::Controller) -> Option<impl Widget + 'b> {
        if !self.enabled {
            return None;
        };
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
            self.enabled = false;
        }
        let text_color = Rgba::BLACK;
        let button_color = Rgba::try_from("#aabbff").unwrap();
        let text = format!("Tier {}", self.tier);
        let tier = geng::ui::Text::new(text, cx.geng().default_font(), 120.0, text_color);

        let text = if self.money == 1 { "coin" } else { "coins" };
        let text = format!("{} {}", self.money, text);
        let coins = geng::ui::Text::new(text, cx.geng().default_font(), 60.0, text_color);

        let mut clans_info = geng::ui::column![];
        let clan_members = calc_clan_members(&self.team);
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
        if !self.enabled {
            return;
        };
        match event {
            geng::Event::MouseDown {
                position,
                button: MouseButton::Left,
            } => {
                let position = position.map(|x| x as f32);
                let mouse_world_pos = self
                    .camera
                    .screen_to_world(self.framebuffer_size, position.map(|x| x as f32));

                let mut drag_index = -1 as i32;
                if self.money >= UNIT_COST {
                    for (index, unit) in self.units.iter().enumerate() {
                        if unit.is_touched(mouse_world_pos) {
                            drag_index = index as i32;
                            break;
                        }
                    }
                }
                if drag_index >= 0 {
                    self.drag_controller.drag_target = Some(self.units.remove(drag_index as usize));
                    self.drag_controller.source = DragSource::Shop;
                } else {
                    for (index, unit) in self.team.iter().enumerate() {
                        if unit.is_touched(mouse_world_pos) {
                            drag_index = index as i32;
                            break;
                        }
                    }
                    if drag_index >= 0 {
                        self.drag_controller.drag_target =
                            Some(self.team.remove(drag_index as usize));
                        self.drag_controller.source = DragSource::Team;
                        self.updated = true;
                    }
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
                            if let Some(unit_in_slot) = self
                                .team
                                .iter_mut()
                                .find(|unit| unit.position.x == i as i64)
                            {
                                dropped = unit_in_slot.merge(drag.clone());
                            } else {
                                drag.position = Position {
                                    side: Faction::Player,
                                    x: i as i64,
                                };
                                drag.faction = Faction::Player;
                                drag.drag(drag.position.to_world());
                                self.team.push(drag.clone());
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
                        match self.drag_controller.source {
                            DragSource::Team => self.team.push(drag),
                            DragSource::Shop => self.units.push(drag),
                        }
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
            .screen_to_world(self.framebuffer_size, position.map(|x| x as f32))
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
            if let Some(units) = tier_units_number(self.tier) {
                let mut rng = global_rng();
                let options = self
                    .available
                    .iter()
                    .filter(|(_, unit)| unit.tier <= self.tier)
                    .map(|(name, template)| {
                        Unit::new(
                            &template.clone(),
                            0,
                            Position::zero(Faction::Player),
                            &Statuses { map: hashmap! {} },
                        )
                    })
                    .collect::<Vec<_>>();
                if options.is_empty() {
                    error!("No units are available to roll");
                    return;
                }
                self.units = (0..units)
                    .map(|_| options.choose(&mut rng).unwrap().clone())
                    .collect();
                for (index, unit) in self.units.iter_mut().enumerate() {
                    let position = Position {
                        side: Faction::Enemy,
                        x: index.try_into().unwrap(),
                    };
                    unit.render.render_position = position.to_world();
                    unit.faction = position.side;
                    unit.position = position.clone();
                }
            }
            self.frozen = false;
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
