use super::*;

use event::Event;
use strum_macros::Display;

#[derive(Deserialize, Serialize, Clone, Debug, Default, Display, PartialEq, EnumIter)]
pub enum Trigger {
    AfterDamageTaken(Effect),
    AfterDamageDealt(Effect),
    BattleStart(Effect),
    TurnStart(Effect),
    BeforeStrike(Effect),
    AllyDeath(Effect),
    BeforeDeath(Effect),
    AfterKill(Effect),
    ChangeVar(VarName, Expression),
    List(Vec<Box<Trigger>>),
    #[default]
    Noop,
}

impl Trigger {
    pub fn catch_event(&self, event: &Event) -> Vec<Trigger> {
        match self {
            Trigger::Noop | Trigger::ChangeVar(..) => default(),
            Trigger::List(triggers) => triggers
                .into_iter()
                .map(|t| t.catch_event(event))
                .flatten()
                .collect_vec(),
            Trigger::AfterDamageTaken(..) => match event {
                Event::DamageTaken { .. } => vec![self.clone()],
                _ => default(),
            },
            Trigger::AfterDamageDealt(..) => match event {
                Event::DamageDealt { .. } => vec![self.clone()],
                _ => default(),
            },
            Trigger::BattleStart(..) => match event {
                Event::BattleStart => vec![self.clone()],
                _ => default(),
            },
            Trigger::TurnStart(..) => match event {
                Event::TurnStart => vec![self.clone()],
                _ => default(),
            },
            Trigger::BeforeStrike(..) => match event {
                Event::BeforeStrike(..) => vec![self.clone()],
                _ => default(),
            },
            Trigger::AllyDeath(..) => match event {
                Event::Death(..) => vec![self.clone()],
                _ => default(),
            },
            Trigger::BeforeDeath(..) => match event {
                Event::Death(..) => vec![self.clone()],
                _ => default(),
            },
            Trigger::AfterKill(..) => match event {
                Event::Kill { .. } => vec![self.clone()],
                _ => default(),
            },
        }
    }

    pub fn fire(self, event: &Event, context: &Context, status: Entity, world: &mut World) {
        let mut context = mem::take(
            context
                .clone()
                .set_owner(get_parent(status, world), world)
                .set_status(status, world),
        );
        match self {
            Trigger::AfterDamageTaken(effect)
            | Trigger::AfterDamageDealt(effect)
            | Trigger::BattleStart(effect)
            | Trigger::TurnStart(effect)
            | Trigger::BeforeStrike(effect) => {
                ActionPlugin::push_back(effect, context, world);
            }
            Trigger::AllyDeath(effect) => {
                let dead = match event {
                    Event::Death(unit) => *unit,
                    _ => panic!(),
                };
                let owner = get_parent(status, world);
                if UnitPlugin::get_faction(dead, world).eq(&UnitPlugin::get_faction(owner, world)) {
                    ActionPlugin::push_back(effect, context, world);
                }
            }
            Trigger::BeforeDeath(effect) => {
                let dead = match event {
                    Event::Death(unit) => *unit,
                    _ => panic!(),
                };
                let owner = get_parent(status, world);
                if dead.eq(&owner) {
                    ActionPlugin::push_back(effect, context, world);
                }
            }
            Trigger::AfterKill(effect) => {
                let target = match event {
                    Event::Kill { owner: _, target } => *target,
                    _ => panic!(),
                };
                context.set_target(target, world);
                ActionPlugin::push_back(effect, context, world);
            }
            _ => panic!("Trigger {self} can not be fired"),
        }
    }

    pub fn collect_delta_triggers(&self) -> Vec<Trigger> {
        match self {
            Trigger::ChangeVar(_, _) => vec![self.clone()],
            Trigger::List(triggers) => triggers
                .into_iter()
                .map(|t| t.collect_delta_triggers())
                .flatten()
                .collect_vec(),
            _ => default(),
        }
    }

    pub fn show_editor_root(
        &mut self,
        entity: Option<Entity>,
        editing_data: &mut EditingData,
        name: String,
        show_name: bool,
        ui: &mut Ui,
        world: &mut World,
    ) {
        ui.horizontal(|ui| {
            if show_name {
                ui.label(name.clone());
            }
            self.show_editor(editing_data, name, ui);
        });
    }

    pub fn show_editor(&mut self, editing_data: &mut EditingData, name: String, ui: &mut Ui) {
        let hovered = if let Some(hovered) = editing_data.hovered.as_ref() {
            hovered.eq(&name)
        } else {
            false
        };
        let color = match hovered {
            true => hex_color!("#FF9100"),
            false => hex_color!("#1E88E5"),
        };
        ui.style_mut().visuals.hyperlink_color = color;
        let mut now_hovered = false;
        ui.horizontal(|ui| {
            let left = ui.link(RichText::new("("));
            if left.clicked() {
                let ts = Trigger::TurnStart(Effect::Noop);
                *self = ts;
            }
            now_hovered |= left.hovered();
            ui.vertical(|ui| {
                let link = ui.link(RichText::new(format!("{self}")));
                if link.clicked() {
                    editing_data.lookup.clear();
                    link.request_focus();
                }
                now_hovered |= link.hovered();
                if link.has_focus() || link.lost_focus() {
                    let mut need_clear = false;
                    ui.horizontal_wrapped(|ui| {
                        ui.label(editing_data.lookup.to_owned());
                        Trigger::iter()
                            .filter_map(|e| {
                                match e
                                    .to_string()
                                    .to_lowercase()
                                    .starts_with(editing_data.lookup.to_lowercase().as_str())
                                {
                                    true => Some(e),
                                    false => None,
                                }
                            })
                            .for_each(|e| {
                                let button = ui.button(e.to_string());
                                if button.gained_focus() || button.clicked() {
                                    *self = e;
                                    need_clear = true;
                                }
                            })
                    });
                    if need_clear {
                        editing_data.lookup.clear();
                    }
                }
            });
        });

        match self {
            Trigger::AfterDamageTaken(e)
            | Trigger::AfterDamageDealt(e)
            | Trigger::BattleStart(e)
            | Trigger::TurnStart(e)
            | Trigger::BeforeStrike(e)
            | Trigger::AllyDeath(e)
            | Trigger::BeforeDeath(e)
            | Trigger::AfterKill(e) => todo!(),
            Trigger::ChangeVar(_, _) => todo!(),
            Trigger::List(_) => todo!(),
            Trigger::Noop => todo!(),
        }
        ui.style_mut().visuals.hyperlink_color = color;
        let right = ui.link(RichText::new(")"));
        if right.clicked() {}
        now_hovered |= right.hovered();
        if now_hovered && !editing_data.hovered.as_ref().eq(&Some(&name)) {
            editing_data.hovered = Some(name.clone());
        }
    }
}
