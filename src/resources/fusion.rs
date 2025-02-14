use super::*;

impl Fusion {
    pub fn init_all(world: &mut World) -> Result<(), ExpressionError> {
        for fusion in world.query::<&Fusion>().iter(world).cloned().collect_vec() {
            fusion.init(world)?;
        }
        Ok(())
    }
    pub fn init(self, world: &mut World) -> Result<(), ExpressionError> {
        let entity = self.entity();
        let units = self.units_entities(&Context::new_world(world))?;
        let mut fusion_stats = UnitStats::default();
        for u in units {
            let stats = world.get::<UnitStats>(u).to_e("Unit stats not found")?;
            fusion_stats.hp += stats.hp;
            fusion_stats.dmg += stats.dmg;
            fusion_stats.pwr += stats.pwr;
        }
        NodeState::from_world_mut(entity, world)
            .unwrap()
            .init_vars(fusion_stats.get_vars());
        world.entity_mut(entity).insert(fusion_stats);
        Ok(())
    }
    pub fn units_entities(&self, context: &Context) -> Result<Vec<Entity>, ExpressionError> {
        let mut units: Vec<Entity> = default();
        for unit in &self.units {
            units.push(context.entity_by_name(unit)?);
        }
        Ok(units)
    }
    pub fn get_unit(&self, unit: u8, context: &Context) -> Result<Entity, ExpressionError> {
        let unit = &self.units[unit as usize];
        context.entity_by_name(unit)
    }
    pub fn remove_unit(&mut self, u: u8) {
        self.units.remove(u as usize);
        self.triggers
            .retain_mut(|(UnitTriggerRef { unit, trigger: _ }, _)| {
                if *unit == u {
                    return false;
                } else if *unit > u {
                    *unit -= 1;
                }
                true
            });
        for (_, actions) in self.triggers.iter_mut() {
            actions.retain_mut(
                |UnitActionRef {
                     unit,
                     trigger: _,
                     action: _,
                 }| {
                    if *unit == u {
                        return false;
                    } else if *unit > u {
                        *unit -= 1;
                    }
                    true
                },
            );
        }
    }
    pub fn remove_trigger(&mut self, r: UnitTriggerRef) {
        self.triggers.retain(|(t, _)| !r.eq(t));
    }
    pub fn remove_action(&mut self, r: UnitActionRef) {
        for (_, a) in self.triggers.iter_mut() {
            a.retain(|a| !r.eq(a));
        }
    }
    pub fn get_reaction<'a>(
        &self,
        unit: u8,
        context: &'a Context,
    ) -> Result<&'a Reaction, ExpressionError> {
        let unit = self.get_unit(unit, context)?;
        context
            .get_component::<Reaction>(unit)
            .to_e("Reaction not found")
    }
    pub fn get_trigger<'a>(
        &self,
        unit: u8,
        trigger: u8,
        context: &'a Context,
    ) -> Result<&'a Trigger, ExpressionError> {
        let reaction = self.get_reaction(unit, context)?;
        Ok(&reaction.triggers[trigger as usize].0)
    }
    pub fn get_action<'a>(
        &self,
        r: &UnitActionRef,
        context: &'a Context,
    ) -> Result<(Entity, &'a Action), ExpressionError> {
        let reaction = self.get_reaction(r.unit, context)?;
        Ok((
            reaction.entity(),
            &reaction.triggers[r.trigger as usize].1[r.action as usize],
        ))
    }
    pub fn react(
        &self,
        event: &Event,
        context: &mut Context,
    ) -> Result<Vec<BattleAction>, ExpressionError> {
        let mut battle_actions: Vec<BattleAction> = default();
        for (UnitTriggerRef { unit, trigger }, actions) in &self.triggers {
            if self
                .get_trigger(*unit, *trigger, context)?
                .fire(event, context)
            {
                for action in actions {
                    let (entity, action) = self.get_action(action, context)?;
                    battle_actions.extend(action.process(context.clone().set_caster(entity))?);
                }
            }
        }
        Ok(battle_actions)
    }
    pub fn paint(&self, rect: Rect, ui: &mut Ui, world: &World) -> Result<(), ExpressionError> {
        let entity = self.entity();
        let units = self.units_entities(&Context::new_world(world))?;
        for unit in units {
            let Some(rep) = world.get::<Representation>(unit) else {
                continue;
            };
            let context = Context::new_world(world)
                .set_owner(unit)
                .set_owner(entity)
                .take();
            RepresentationPlugin::paint_rect(rect, &context, &rep.material, ui)?;
        }
        Ok(())
    }

    pub fn open_editor_window(
        entity: Entity,
        world: &mut World,
        team_world: &World,
        on_save: fn(Fusion, &mut World),
    ) -> Result<(), ExpressionError> {
        let slot = team_world
            .get::<UnitSlot>(entity)
            .to_e("Fusion not found")?;

        let team = slot.find_up::<Team>(team_world).unwrap();
        let mut team = Team::pack(team.entity(), team_world).to_e("Failed to pack Team")?;
        team.clear_entities();
        let mut team_world = World::new();
        let team_entity = team_world.spawn_empty().id();
        team.unpack(team_entity, &mut team_world);
        Fusion::init_all(&mut team_world)?;
        let mut fusion = team_world
            .query::<(&UnitSlot, &Fusion)>()
            .iter(&team_world)
            .find_map(|(s, f)| {
                if s.slot == slot.slot {
                    Some(f.clone())
                } else {
                    None
                }
            })
            .unwrap();
        Window::new("Fusion Editor", move |ui, world| {
            let mut init_fusion = false;
            ui.horizontal(|ui| {
                for (unit, stats) in team_world.query::<(&Unit, &UnitStats)>().iter(&team_world) {
                    let selected = fusion.units.contains(&unit.name);
                    FRAME
                        .stroke(if selected { STROKE_YELLOW } else { STROKE_DARK })
                        .show(ui, |ui| {
                            show_unit_tag(unit, stats, ui, &team_world);
                            if "select".cstr_s(CstrStyle::Bold).button(ui).clicked() {
                                if selected {
                                    let i =
                                        fusion.units.iter().position(|u| unit.name.eq(u)).unwrap();
                                    fusion.remove_unit(i as u8);
                                } else {
                                    fusion.units.push(unit.name.clone());
                                }
                                init_fusion = true;
                            }
                        });
                }
            });
            ui.horizontal(|ui| {
                let context = &Context::new_world(&team_world);
                ui.vertical(|ui| {
                    "Select Triggers".cstr_s(CstrStyle::Heading2).label(ui);
                    for u in 0..fusion.units.len() {
                        let triggers = &fusion.get_reaction(u as u8, context).unwrap().triggers;
                        for (t, (trigger, _)) in triggers.iter().enumerate() {
                            let t_ref = UnitTriggerRef {
                                unit: u as u8,
                                trigger: t as u8,
                            };
                            let selected = fusion.triggers.iter().any(|(r, _)| r.eq(&t_ref));
                            FRAME
                                .stroke(if selected { STROKE_YELLOW } else { STROKE_DARK })
                                .show(ui, |ui| {
                                    ui.horizontal(|ui| {
                                        trigger.show(None, context, ui);
                                        if ">".cstr_s(CstrStyle::Bold).button(ui).clicked() {
                                            if selected {
                                                fusion.remove_trigger(t_ref);
                                            } else {
                                                fusion.triggers.push((
                                                    UnitTriggerRef {
                                                        unit: u as u8,
                                                        trigger: t as u8,
                                                    },
                                                    default(),
                                                ));
                                            }
                                        }
                                    })
                                });
                        }
                    }
                });
                ui.vertical(|ui| {
                    if fusion.triggers.is_empty() {
                        return;
                    }
                    "Select Actions".cstr_s(CstrStyle::Heading2).label(ui);
                    for u in 0..fusion.units.len() {
                        let reaction = &fusion.get_reaction(u as u8, context).unwrap();
                        let triggers = &reaction.triggers;
                        let entity = reaction.entity();
                        for (t, (_, actions)) in triggers.iter().enumerate() {
                            for (a, action) in actions.0.iter().enumerate() {
                                let a_ref = UnitActionRef {
                                    unit: u as u8,
                                    trigger: t as u8,
                                    action: a as u8,
                                };
                                let selected = fusion
                                    .triggers
                                    .iter()
                                    .any(|(_, a)| a.iter().any(|a| a_ref.eq(a)));
                                FRAME
                                    .stroke(if selected { STROKE_YELLOW } else { STROKE_DARK })
                                    .show(ui, |ui| {
                                        ui.horizontal(|ui| {
                                            action.show(
                                                None,
                                                context.clone().set_owner(entity),
                                                ui,
                                            );
                                            if ">".cstr_s(CstrStyle::Bold).button(ui).clicked() {
                                                if selected {
                                                    fusion.remove_action(a_ref);
                                                } else {
                                                    fusion
                                                        .triggers
                                                        .last_mut()
                                                        .unwrap()
                                                        .1
                                                        .push(a_ref);
                                                }
                                            }
                                        })
                                    });
                            }
                        }
                    }
                });
            });
            FRAME.show(ui, |ui| {
                ui.horizontal(|ui| {
                    let context = &Context::new_world(&team_world)
                        .set_owner(fusion.entity())
                        .take();
                    ui.vertical(|ui| {
                        "Result".cstr_s(CstrStyle::Heading2).label(ui);
                        let mut remove_t = None;
                        let mut remove_a = None;
                        let mut swap = None;
                        for (t_i, (t_ref, actions)) in fusion.triggers.iter().enumerate() {
                            let trigger = fusion
                                .get_trigger(t_ref.unit, t_ref.trigger, context)
                                .unwrap();
                            ui.horizontal(|ui| {
                                if "-".cstr_cs(RED, CstrStyle::Bold).button(ui).clicked() {
                                    remove_t = Some(*t_ref);
                                }
                                trigger.show(None, context, ui);
                            });
                            FRAME.show(ui, |ui| {
                                for (a_i, a_ref) in actions.iter().enumerate() {
                                    let (entity, action) =
                                        fusion.get_action(a_ref, context).unwrap();
                                    ui.horizontal(|ui| {
                                        if "-".cstr_cs(RED, CstrStyle::Bold).button(ui).clicked() {
                                            remove_a = Some(*a_ref);
                                        }
                                        if (t_i > 0 || a_i > 0)
                                            && "^"
                                                .cstr_cs(VISIBLE_BRIGHT, CstrStyle::Bold)
                                                .button(ui)
                                                .clicked()
                                        {
                                            if a_i == 0 {
                                                swap = Some((
                                                    (t_i, a_i),
                                                    (t_i - 1, fusion.triggers[t_i - 1].1.len()),
                                                ));
                                            } else {
                                                swap = Some(((t_i, a_i), (t_i, a_i - 1)));
                                            }
                                        }
                                        if (t_i + 1 < fusion.triggers.len()
                                            || a_i + 1 < actions.len())
                                            && "v"
                                                .cstr_cs(VISIBLE_BRIGHT, CstrStyle::Bold)
                                                .button(ui)
                                                .clicked()
                                        {
                                            if a_i == actions.len() - 1 {
                                                swap = Some(((t_i, a_i), (t_i + 1, 0)));
                                            } else {
                                                swap = Some(((t_i, a_i), (t_i, a_i + 1)));
                                            }
                                        }
                                        action.show(None, context.clone().set_owner(entity), ui);
                                    });
                                }
                            });
                        }
                        if let Some(((from_t, from_a), (to_t, to_a))) = swap {
                            let action = fusion.triggers[from_t].1.remove(from_a);
                            fusion.triggers[to_t].1.insert(to_a, action);
                        }
                        if let Some(r) = remove_a {
                            fusion.remove_action(r);
                        }
                        if let Some(r) = remove_t {
                            fusion.remove_trigger(r);
                        }
                        if "save"
                            .cstr_cs(VISIBLE_BRIGHT, CstrStyle::Heading2)
                            .button(ui)
                            .clicked()
                        {
                            WindowPlugin::close_current(world);
                            on_save(fusion.clone(), world);
                        }
                    });

                    let size = ui.available_size();
                    let size = size.x.at_most(size.y).at_least(150.0);
                    let rect = ui
                        .allocate_exact_size(egui::vec2(size, size), Sense::hover())
                        .0;
                    fusion.paint(rect, ui, &team_world).log();
                    unit_rep().paint(rect.shrink(15.0), context, ui).log();
                });
            });
            if init_fusion {
                fusion.clone().init(&mut team_world).log();
            }
        })
        .push(world);
        Ok(())
    }
}

const FRAME: Frame = Frame {
    inner_margin: Margin::same(5.0),
    outer_margin: Margin::same(5.0),
    rounding: ROUNDING,
    shadow: Shadow::NONE,
    fill: TRANSPARENT,
    stroke: STROKE_DARK,
};
