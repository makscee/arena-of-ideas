use super::*;

#[derive(Clone, Serialize, Deserialize, Default, Debug)]
pub struct UnitCard {
    pub name: Cstr,
    houses: Vec<String>,
    house_colors: Vec<Color32>,
    rarity_colors: Vec<Color32>,
    lvl: i32,
    xp: i32,
    hp: i32,
    pwr: i32,
    deafness: f32,
    hp_mutation: i32,
    pwr_mutation: i32,
    triggers: Vec<Cstr>,
    targets: Vec<Cstr>,
    effects: Vec<Cstr>,
    active_statuses: HashMap<String, i32>,
    used_definitions: HashMap<String, Cstr>,
}

impl UnitCard {
    pub fn from_packed(unit: PackedUnit, world: &mut World) -> Result<Self> {
        let unit = unit.unpack(TeamPlugin::entity(Faction::Team, world), None, None, world);
        let context = Context::new(unit).detach(world).take();
        UnitPlugin::despawn(unit, world);
        UnitCard::new(&context, world)
    }
    pub fn from_fused(unit: FusedUnit, world: &mut World) -> Result<Self> {
        Self::from_packed(unit.into(), world)
    }
    pub fn from_base(unit: TBaseUnit, world: &mut World) -> Result<Self> {
        Self::from_packed(unit.into(), world)
    }
    pub fn new(context: &Context, world: &World) -> Result<Self> {
        debug!("new card");
        let mut effects = context
            .get_value(VarName::EffectsDescription, world)?
            .get_cstr_list()?;
        for c in effects.iter_mut() {
            c.inject_context(context, world);
        }
        Ok(Self {
            name: entity_name(context.owner())
                .style(CstrStyle::Heading)
                .take(),
            houses: context
                .get_value(VarName::Houses, world)?
                .get_string_list()?,
            house_colors: context
                .get_value(VarName::HouseColors, world)?
                .get_color32_list()?,
            rarity_colors: context
                .get_value(VarName::RarityColors, world)?
                .get_color32_list()?,
            lvl: context.get_int(VarName::Lvl, world)?,
            xp: context.get_int(VarName::Xp, world)?,
            hp: context.get_int(VarName::Hp, world)?,
            pwr: context.get_int(VarName::Pwr, world)?,
            deafness: context
                .get_float(VarName::Deafness, world)
                .unwrap_or_default(),
            hp_mutation: context.get_int(VarName::HpMutation, world)?,
            pwr_mutation: context.get_int(VarName::PwrMutation, world)?,
            triggers: context
                .get_value(VarName::TriggersDescription, world)?
                .get_cstr_list()?,
            targets: context
                .get_value(VarName::TargetsDescription, world)?
                .get_cstr_list()?,
            effects,
            active_statuses: context.all_active_statuses(world),
            used_definitions: HashMap::from_iter(
                context
                    .get_value(VarName::UsedDefinitions, world)?
                    .get_string_list()?
                    .into_iter()
                    .map(|name| {
                        (
                            name.clone(),
                            definition(&name)
                                .inject_ability_state(
                                    &name,
                                    context.clone().set_ability_state(&name, world).unwrap(),
                                )
                                .take(),
                        )
                    }),
            ),
        })
    }
    pub fn ui(&self, ui: &mut Ui) {
        let fusible_lvl = self.houses.len() as i32 + 1;
        let fusible_str = if fusible_lvl > self.lvl {
            "Fusible from lvl "
                .cstr()
                .push(fusible_lvl.to_string().cstr_cs(PURPLE, CstrStyle::Bold))
                .take()
        } else {
            "Fusible".cstr_cs(YELLOW, CstrStyle::Bold)
        };

        let rect = Frame {
            inner_margin: Margin::same(8.0),
            outer_margin: Margin::ZERO,
            rounding: Rounding::ZERO,
            shadow: Shadow::NONE,
            fill: EMPTINESS,
            stroke: Stroke::NONE,
        }
        .show(ui, |ui| {
            ui.set_min_width(ui.available_width());
            self.name.label(ui);
            fn mutation_cstr(value: i32) -> Cstr {
                match value.signum() {
                    1 => format!(" (+{value})").cstr_c(GREEN),
                    -1 => format!(" ({value})").to_string().cstr_c(RED),
                    _ => format!(" (+{value})").cstr(),
                }
            }
            ui.horizontal_wrapped(|ui| {
                let var = VarName::Pwr;
                let color = YELLOW;
                var.cstr_c(color)
                    .push(": ".cstr_c(color))
                    .push(self.pwr.to_string().cstr_c(VISIBLE_BRIGHT))
                    .style(CstrStyle::Bold)
                    .push(mutation_cstr(self.pwr_mutation))
                    .label(ui);
                ui.add_space(2.0);
                let var = VarName::Hp;
                let color = RED;
                var.cstr_c(color)
                    .push(": ".cstr_c(color))
                    .push(self.hp.to_string().cstr_c(VISIBLE_BRIGHT))
                    .style(CstrStyle::Bold)
                    .push(mutation_cstr(self.hp_mutation))
                    .label(ui);
                ui.add_space(2.0);
                let var = VarName::Lvl;
                let color = PURPLE;
                var.cstr_c(color)
                    .push(": ".cstr_c(color))
                    .push(self.lvl.to_string().cstr_c(VISIBLE_BRIGHT))
                    .style(CstrStyle::Bold)
                    .label(ui);
                ui.add_space(2.0);
                let var = VarName::Xp;
                let color = LIGHT_PURPLE;
                var.cstr_c(color)
                    .push(": ".cstr_c(color))
                    .push(self.xp.to_string().cstr_c(VISIBLE_BRIGHT))
                    .push("/".cstr())
                    .push((self.lvl).to_string().cstr_c(VISIBLE_BRIGHT))
                    .style(CstrStyle::Bold)
                    .label(ui);
                if self.deafness > 0.01 {
                    ui.add_space(2.0);
                    let var = VarName::Deafness;
                    let color = RED;
                    var.cstr_c(color)
                        .push(": ".cstr_c(color))
                        .push(format!("{}%", (self.deafness * 100.0) as i32).cstr_c(RED))
                        .style(CstrStyle::Bold)
                        .label(ui);
                }
                ui.add_space(2.0);
            });

            fusible_str.label(ui);

            let mut houses_cstr = Cstr::default();
            for (i, house) in self.houses.iter().enumerate() {
                houses_cstr.push(house.cstr_c(self.house_colors[i]));
            }
            houses_cstr.join(&" + ".cstr()).as_label(ui).wrap().ui(ui);
            ui.add_space(2.0);
        })
        .response
        .rect;

        if self.house_colors.len() > 1 {
            let len = self.house_colors.len() as f32;
            let t = gt().play_head() * 0.1;
            for (i, color) in self.house_colors.iter().copied().enumerate() {
                let from = (i as f32 / len + t).fract();
                let to = ((i + 1) as f32 / len + t).fract();
                lines_around_rect((from, to), &rect, color, ui);
            }
        } else {
            lines_around_rect((0.0, 1.0), &rect, self.house_colors[0], ui);
        }

        ui.add_space(-ui.style().spacing.item_spacing.y + 0.5);
        Frame {
            inner_margin: Margin::same(8.0),
            outer_margin: Margin::ZERO,
            rounding: Rounding {
                nw: 0.0,
                ne: 0.0,
                sw: 13.0,
                se: 13.0,
            },
            shadow: Shadow::NONE,
            fill: BG_TRANSPARENT,
            stroke: Stroke::NONE,
        }
        .show(ui, |ui| {
            ui.set_min_width(ui.available_width());
            show_trigger_part("trg:", &self.triggers, EVENT_COLOR, ui);
            show_trigger_part("tar:", &self.targets, TARGET_COLOR, ui);
            show_trigger_part("eff:", &self.effects, EFFECT_COLOR, ui);

            br(ui);
            if !self.active_statuses.is_empty() {
                ui.horizontal_wrapped(|ui| {
                    for (name, charges) in &self.active_statuses {
                        format!("{name} ({charges})")
                            .cstr_c(name_color(&name))
                            .label(ui);
                    }
                });
                br(ui);
            }
            ui.vertical_centered_justified(|ui| {
                for (name, text) in &self.used_definitions {
                    ui.horizontal_wrapped(|ui| {
                        name.cstr_cs(name_color(&name), CstrStyle::Bold).label(ui);
                        text.as_label(ui).wrap().ui(ui);
                    });
                }
            });
        });
        const OFFSET: egui::Vec2 = egui::vec2(33.0, 0.0);
        let from = rect.center_bottom() - (self.rarity_colors.len() as f32 - 1.0) * 0.5 * OFFSET;
        for (i, color) in self.rarity_colors.iter().enumerate() {
            let pos = from + OFFSET * i as f32;
            ui.painter().circle_filled(pos, 13.0, BG_LIGHT);
            ui.painter().circle_filled(pos, 10.0, *color);
        }
    }
}

fn show_trigger_part(title: &str, content: &Vec<Cstr>, color: Color32, ui: &mut Ui) {
    ui.horizontal(|ui| {
        title.cstr_c(VISIBLE_DARK).label(ui);
        let rect = Frame::none()
            .inner_margin(Margin::same(4.0))
            .show(ui, |ui| {
                ui.vertical(|ui| {
                    for c in content {
                        c.as_label(ui).wrap().ui(ui);
                    }
                })
            })
            .response
            .rect;
        ui.painter().line_segment(
            [rect.left_top(), rect.left_bottom()],
            Stroke { width: 1.0, color },
        );
    });
}

fn lines_around_rect(range: (f32, f32), rect: &Rect, color: Color32, ui: &mut Ui) {
    let mut path = vec![point_on_rect(range.0, rect)];
    let w_part = rect.width() / (rect.width() + rect.height()) * 0.5;
    let points = [
        (0.0, rect.left_top()),
        (w_part, rect.right_top()),
        (0.5, rect.right_bottom()),
        (0.5 + w_part, rect.left_bottom()),
        (1.0, rect.left_top()),
    ];
    let mut start = 0;
    let mut end = 0;
    for i in 0..(points.len() - 1) {
        if range.0 >= points[i].0 && range.0 <= points[i + 1].0 {
            start = i + 1;
        }
        if range.1 >= points[i].0 && range.1 <= points[i + 1].0 {
            end = i + 1;
        }
    }
    if start > end {
        end += points.len();
    }
    for i in start..end {
        path.push(points[i % points.len()].1);
    }
    path.push(point_on_rect(range.1, rect));
    ui.painter()
        .add(egui::Shape::line(path, Stroke { width: 1.0, color }));
}

fn point_on_rect(t: f32, rect: &Rect) -> egui::Pos2 {
    let w_part = rect.width() / (rect.width() + rect.height());
    if t < 0.5 {
        let t = t * 2.0;
        if t < w_part {
            let t = t / w_part;
            rect.left_top() + (rect.right_top() - rect.left_top()) * t
        } else {
            let t = (t - w_part) / (1.0 - w_part);
            rect.right_top() + (rect.right_bottom() - rect.right_top()) * t
        }
    } else {
        let t = (t - 0.5) * 2.0;
        if t < w_part {
            let t = t / w_part;
            rect.right_bottom() + (rect.left_bottom() - rect.right_bottom()) * t
        } else {
            let t = (t - w_part) / (1.0 - w_part);
            rect.left_bottom() + (rect.left_top() - rect.left_bottom()) * t
        }
    }
}

static UNIT_CARD_CACHE: OnceCell<Mutex<HashMap<String, UnitCard>>> = OnceCell::new();
fn cache_packed_unit(
    id: String,
    unit: PackedUnit,
    cache: &mut MutexGuard<HashMap<String, UnitCard>>,
    world: &mut World,
) -> Result<()> {
    cache.insert(id, UnitCard::from_packed(unit, world)?);
    debug!("cache inserted");
    Ok(())
}
pub fn cached_fused_card(unit: &FusedUnit, ui: &mut Ui, world: &mut World) -> Result<()> {
    let id = unit.id.to_string();
    let mut cache = UNIT_CARD_CACHE.get_or_init(|| default()).lock().unwrap();
    if !cache.contains_key(&id) {
        let unit: PackedUnit = unit.clone().into();
        cache_packed_unit(id.clone(), unit, &mut cache, world)?;
    }
    cache
        .get(&id)
        .context("Failed to get cached card context")?
        .ui(ui);
    Ok(())
}

pub fn cached_base_card(unit: &TBaseUnit, ui: &mut Ui, world: &mut World) -> Result<()> {
    let id = unit.name.to_string();
    let mut cache = UNIT_CARD_CACHE.get_or_init(|| default()).lock().unwrap();
    if !cache.contains_key(&id) {
        let unit: PackedUnit = unit.clone().into();
        cache_packed_unit(id.clone(), unit, &mut cache, world)?;
    }
    cache
        .get(&id)
        .context("Failed to get cached card context")?
        .ui(ui);
    Ok(())
}
