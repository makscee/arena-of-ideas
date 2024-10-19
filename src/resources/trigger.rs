use super::*;

#[derive(Deserialize, Serialize, Clone, Debug, Display, PartialEq, EnumIter)]
#[serde(deny_unknown_fields)]
pub enum Trigger {
    Fire {
        #[serde(default)]
        triggers: Vec<(FireTrigger, Option<String>)>,
        #[serde(default)]
        targets: Vec<(Expression, Option<String>)>,
        #[serde(default)]
        effects: Vec<(Effect, Option<String>)>,
    },
    Change {
        trigger: DeltaTrigger,
        expr: Expression,
    },
    List(Vec<Box<Trigger>>),
}

impl Default for Trigger {
    fn default() -> Self {
        Self::Fire {
            triggers: default(),
            targets: default(),
            effects: default(),
        }
    }
}

impl Trigger {
    pub fn fire(&mut self, event: &Event, context: &Context, world: &mut World) -> bool {
        match self {
            Trigger::List(list) => {
                let mut result = false;
                for trigger in list {
                    result |= trigger.fire(event, context, world);
                }
                result
            }
            Trigger::Fire {
                triggers,
                targets,
                effects,
            } => {
                let mut result = false;
                for (trigger, _) in triggers {
                    if trigger.catch(event, context, world) {
                        result = true;
                        for (effect, _) in effects.iter() {
                            match effect {
                                Effect::UseAbility(name, _) => {
                                    Event::UseAbility(name.clone()).send_with_context(
                                        context.clone().set_caster(context.owner()).take(),
                                        world,
                                    );
                                }
                                _ => {}
                            }
                            if targets.is_empty() {
                                ActionPlugin::action_push_back(
                                    effect.clone(),
                                    context.clone(),
                                    world,
                                );
                            } else {
                                for (target, _) in targets.iter() {
                                    let effect = Effect::WithTarget(
                                        target.clone(),
                                        Box::new(effect.clone()),
                                    );
                                    ActionPlugin::action_push_back(effect, context.clone(), world);
                                }
                            }
                        }
                    }
                }
                result
            }
            Trigger::Change { .. } => false,
        }
    }
    pub fn change(
        &self,
        event: &Event,
        context: &Context,
        value: &mut VarValue,
        world: &mut World,
    ) -> Result<()> {
        match self {
            Trigger::List(list) => list.iter().for_each(|t| {
                let _ = t.change(event, context, value, world);
            }),
            Trigger::Change { trigger, expr } => {
                if !trigger.catch(event) {
                    return Ok(());
                }
                let delta = expr.get_value(
                    context.clone().set_var(VarName::Value, value.clone()),
                    world,
                )?;
                *value = VarValue::sum(value, &delta)?;
            }
            Trigger::Fire { .. } => {}
        };
        Ok(())
    }
    pub fn collect_mappings(
        &self,
        context: &Context,
        world: &mut World,
    ) -> Vec<(VarName, VarValue)> {
        match self {
            Trigger::List(list) => list
                .iter()
                .flat_map(|t| t.collect_mappings(context, world))
                .collect_vec(),
            Trigger::Change { trigger, expr } => match trigger {
                DeltaTrigger::IncomingDamage => default(),
                DeltaTrigger::Var(var) => match expr.get_value(context, world) {
                    Ok(value) => [(*var, value)].into(),
                    Err(e) => {
                        debug!("{} {e}", "Mapping error:".red());
                        default()
                    }
                },
            },
            Trigger::Fire { .. } => default(),
        }
    }
    pub fn parse_fire_strings(&self) -> (Vec<Cstr>, Vec<Cstr>, Vec<Cstr>) {
        let mut cs = (Vec::new(), Vec::new(), Vec::new());
        match self {
            Trigger::Fire {
                triggers,
                targets,
                effects,
            } => {
                for (trigger, rename) in triggers {
                    if let Some(rename) = rename {
                        cs.0.push(
                            Cstr::parse(rename)
                                .replace_absent_color(VISIBLE_LIGHT)
                                .take(),
                        );
                    } else {
                        cs.0.push(trigger.cstr_expanded());
                    }
                }
                for (target, rename) in targets {
                    if let Some(rename) = rename {
                        cs.1.push(
                            Cstr::parse(rename)
                                .replace_absent_color(VISIBLE_LIGHT)
                                .take(),
                        );
                    } else {
                        cs.1.push(target.cstr_expanded());
                    }
                }
                for (effect, rename) in effects {
                    if let Some(rename) = rename {
                        cs.2.push(
                            Cstr::parse(rename)
                                .replace_absent_color(VISIBLE_LIGHT)
                                .take(),
                        );
                    } else {
                        cs.2.push(effect.cstr_expanded());
                    }
                }
            }
            _ => panic!("Has to be Trigger::Fire"),
        }
        cs
    }
}

impl ToCstr for Trigger {
    fn cstr_expanded(&self) -> Cstr {
        match self {
            Trigger::Fire {
                triggers,
                targets,
                effects,
            } => {
                let mut c = "Fire".cstr();
                let triggers = triggers
                    .iter()
                    .map(|(t, n)| {
                        n.clone()
                            .map(|s| s.cstr())
                            .unwrap_or_else(|| t.cstr_expanded())
                            .push("|".cstr())
                            .take()
                    })
                    .collect_vec();
                let triggers = Cstr::join_vec(triggers).style(CstrStyle::Small).take();
                c.push("\ntrg:".cstr());
                c.push(triggers);

                let targets = targets
                    .iter()
                    .map(|(t, n)| {
                        n.clone()
                            .map(|s| s.cstr())
                            .unwrap_or_else(|| t.cstr_expanded())
                            .push("|".cstr())
                            .take()
                    })
                    .collect_vec();
                let targets = Cstr::join_vec(targets).style(CstrStyle::Small).take();
                c.push("\ntgt:".cstr());
                c.push(targets);

                let effects = effects
                    .iter()
                    .map(|(t, n)| {
                        n.clone()
                            .map(|s| s.cstr())
                            .unwrap_or_else(|| t.cstr_expanded())
                            .push("|".cstr())
                            .take()
                    })
                    .collect_vec();
                let effects = Cstr::join_vec(effects).style(CstrStyle::Small).take();
                c.push("\neff:".cstr());
                c.push(effects);
                c
            }
            Trigger::Change { .. } => "Change".cstr(),
            Trigger::List(list) => "List "
                .cstr()
                .push(Cstr::join_vec(list.iter().map(|e| e.cstr()).collect_vec()))
                .take(),
        }
    }

    fn cstr(&self) -> Cstr {
        match self {
            Trigger::Fire { .. } => "Fire".cstr_c(YELLOW),
            Trigger::Change { .. } => "Change".cstr_c(CYAN),
            Trigger::List(..) => "List".cstr_c(LIGHT_PURPLE),
        }
    }
}

fn show_named_nodes<T: ShowEditor>(
    name: &str,
    nodes: &mut Vec<(T, Option<String>)>,
    context: &Context,
    world: &mut World,
    ui: &mut Ui,
) {
    let mut c = 0;
    ui.collapsing(name, |ui| {
        for (node, name) in nodes.iter_mut() {
            c += 1;
            ui.push_id(c, |ui| {
                ui.horizontal(|ui| {
                    if Checkbox::new(&mut name.is_some(), "").ui(ui).changed() {
                        if name.is_some() {
                            *name = None;
                        } else {
                            *name = Some(default());
                        }
                    }
                    if let Some(name) = name {
                        Input::new("rename").ui_string(name, ui);
                    }
                });
                node.show_node("", context, world, ui);
            });
        }
        if Button::click("+").ui(ui).clicked() {
            nodes.push((default(), None));
        }
    });
}
impl ShowEditor for Trigger {
    fn show_node(&mut self, _: &str, context: &Context, world: &mut World, ui: &mut Ui) {
        self.show_self(ui, world);
        self.cstr_expanded().label(ui);
        self.show_children(context, world, ui);
    }
    fn show_children(&mut self, context: &Context, world: &mut World, ui: &mut Ui) {
        match self {
            Trigger::Fire {
                triggers,
                targets,
                effects,
            } => {
                show_named_nodes("Triggers", triggers, context, world, ui);
                show_named_nodes("Targets", targets, context, world, ui);
                show_named_nodes("Effects", effects, context, world, ui);
            }
            Trigger::Change { trigger, expr } => {
                trigger.show_node("trigger", context, world, ui);
                expr.show_node("expression", context, world, ui);
            }
            Trigger::List(l) => {
                show_list_node(l, context, ui, world);
            }
        }
    }
    fn get_inner_mut(&mut self) -> Vec<&mut Box<Self>> {
        default()
    }
    fn get_variants() -> impl Iterator<Item = Self> {
        Self::iter()
    }
}
