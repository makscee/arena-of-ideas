use super::*;

impl Fusion {
    fn remove_unit(&mut self, id: u64) {
        let Some(ui) = self.units.iter().position(|u| *u == id) else {
            return;
        };
        self.units.remove(ui);
        let ui = ui as u8;
        self.behavior.retain(|(t, _)| t.unit != ui);
        for (tr, ars) in &mut self.behavior {
            if tr.unit > ui {
                tr.unit -= 1;
            }
            ars.retain(|ar| ar.unit != ui);
            for ar in ars {
                if ar.unit > ui {
                    ar.unit -= 1;
                }
            }
        }
    }
    pub fn pwr_hp(&self, context: &Context) -> Result<(i32, i32), ExpressionError> {
        let mut pwr = 0;
        let mut hp = 0;
        for unit in self.units(context)? {
            let stats = unit.stats_err(context)?;
            pwr += stats.pwr;
            hp += stats.hp;
        }
        Ok((pwr, hp))
    }
    pub fn units<'a>(&self, context: &'a Context) -> Result<Vec<&'a Unit>, ExpressionError> {
        let mut units = Vec::new();
        for id in &self.units {
            let unit = context
                .get_component_by_id::<Unit>(*id)
                .to_e_fn(|| format!("Failed to get Unit#{id}"))?;
            units.push(unit);
        }
        Ok(units)
    }
    pub fn get_unit<'a>(&self, ui: u8, context: &'a Context) -> Result<&'a Unit, ExpressionError> {
        self.units(context)?
            .get(ui as usize)
            .copied()
            .to_e_fn(|| format!("Failed to find Unit as index {ui}"))
    }
    pub fn get_behavior<'a>(
        &self,
        ui: u8,
        context: &'a Context,
    ) -> Result<&'a Behavior, ExpressionError> {
        self.get_unit(ui, context)?
            .description_load(context)
            .to_e("Failed to load UnitDescription")?
            .behavior_load(context)
            .to_e("Failed to load Behavior")
    }
    pub fn get_trigger<'a>(
        &self,
        tr: &UnitTriggerRef,
        context: &'a Context,
    ) -> Result<&'a Trigger, ExpressionError> {
        let reaction = self.get_behavior(tr.unit, context)?;
        Ok(&reaction.reactions[tr.trigger as usize].trigger)
    }
    pub fn get_action<'a>(
        &self,
        ar: &UnitActionRef,
        context: &'a Context,
    ) -> Result<(Entity, &'a Action), ExpressionError> {
        let behavior = self.get_behavior(ar.unit, context)?;
        Ok((
            behavior.entity(),
            &behavior.reactions[ar.trigger as usize].actions[ar.action as usize],
        ))
    }
    pub fn react(
        &self,
        event: &Event,
        context: &mut Context,
    ) -> Result<Vec<BattleAction>, ExpressionError> {
        let mut battle_actions: Vec<BattleAction> = default();
        for (tr, actions) in &self.behavior {
            if self.get_trigger(tr, context)?.fire(event, context) {
                for ar in actions {
                    let (entity, action) = self.get_action(ar, context)?;
                    let action = action.clone();
                    context.set_caster(entity);
                    battle_actions.extend(action.process(context)?);
                }
            }
        }
        Ok(battle_actions)
    }
    pub fn paint(&self, rect: Rect, context: &Context, ui: &mut Ui) -> Result<(), ExpressionError> {
        let entity = self.entity();
        let units = self.units(context)?;
        for unit in units {
            let unit = unit.entity();
            let Some(rep) = context.get_component::<Representation>(unit) else {
                continue;
            };
            let context = context.clone().set_owner(unit).set_owner(entity).take();
            RepresentationPlugin::paint_rect(rect, &context, &rep.material, ui)?;
        }
        if let Some(rep) = context.get_component::<Representation>(entity) {
            RepresentationPlugin::paint_rect(
                rect,
                context.clone().set_owner(entity),
                &rep.material,
                ui,
            )?;
        }
        Ok(())
    }
    pub fn find_by_slot(slot: i32, world: &mut World) -> Option<Self> {
        world.query::<&Fusion>().iter(&world).find_map(|f| {
            if f.slot == slot {
                Some(f.clone())
            } else {
                None
            }
        })
    }
    pub fn show_editor(&mut self, context: &Context, ui: &mut Ui) -> Result<bool, ExpressionError> {
        let units = self.units(context)?;
        let behaviors = units
            .iter()
            .enumerate()
            .filter_map(|(i, u)| {
                if let Some(b) = u
                    .description_load(context)
                    .and_then(|d| d.behavior_load(context))
                {
                    Some((i as u8, b))
                } else {
                    None
                }
            })
            .collect_vec();
        let mut changed = false;
        ui.vertical(|ui| {
            for (u, b) in &behaviors {
                for (t, reaction) in b.reactions.iter().enumerate() {
                    if self
                        .behavior
                        .iter()
                        .any(|(r, _)| r.trigger == t as u8 && r.unit == *u)
                    {
                        continue;
                    }
                    if reaction.trigger.cstr().as_button().ui(ui).clicked() {
                        self.behavior.push((
                            UnitTriggerRef {
                                unit: *u,
                                trigger: t as u8,
                            },
                            default(),
                        ));
                        changed = true;
                    }
                }
            }
        });
        space(ui);
        ui.vertical(|ui| {
            if self.behavior.is_empty() {
                return Result::<(), ExpressionError>::Ok(());
            }
            for (u, b) in &behaviors {
                for (t, (a, action)) in b
                    .reactions
                    .iter()
                    .enumerate()
                    .flat_map(|(i, r)| r.actions.0.iter().enumerate().map(move |a| (i, a)))
                {
                    let r = UnitActionRef {
                        unit: *u,
                        trigger: t as u8,
                        action: a as u8,
                    };
                    if self
                        .behavior
                        .iter()
                        .any(|(_, actions)| actions.contains(&r))
                    {
                        continue;
                    }
                    let unit = units[*u as usize].entity();
                    if action
                        .title_cstr(ViewContext::new(ui), context.clone().set_owner(unit))
                        .button(ui)
                        .clicked()
                    {
                        self.behavior.last_mut().unwrap().1.push(r);
                        changed = true;
                    }
                }
            }
            space(ui);
            let mut new_behavior = None;
            for (ti, (tr, actions)) in self.behavior.iter().enumerate() {
                let trigger = self.get_trigger(tr, context)?;
                if trigger.cstr().button(ui).clicked() {
                    let mut behavior = self.behavior.clone();
                    behavior.remove(ti);
                    new_behavior = Some(behavior);
                }
                for (ai, ar) in actions.iter().enumerate() {
                    let (entity, action) = self.get_action(ar, context)?;
                    ui.horizontal(|ui| {
                        if ti + 1 < self.behavior.len() || ai + 1 < actions.len() {
                            if "🔽".cstr().button(ui).clicked() {
                                let mut behavior = self.behavior.clone();
                                if ai == actions.len() - 1 {
                                    behavior[ti].1.remove(ai);
                                    behavior[ti + 1].1.insert(0, *ar);
                                } else {
                                    behavior[ti].1.swap(ai, ai + 1);
                                }
                                new_behavior = Some(behavior);
                            }
                        }
                        if ti > 0 || ai > 0 {
                            if "🔼".cstr().button(ui).clicked() {
                                let mut behavior = self.behavior.clone();
                                if ai > 0 {
                                    behavior[ti].1.swap(ai, ai - 1);
                                } else {
                                    behavior[ti].1.remove(ai);
                                    behavior[ti - 1].1.push(*ar);
                                }
                                new_behavior = Some(behavior);
                            }
                        }
                        if action
                            .title_cstr(ViewContext::new(ui), context.clone().set_owner(entity))
                            .button(ui)
                            .clicked()
                        {
                            let mut behavior = self.behavior.clone();
                            behavior[ti].1.remove(ai);
                            new_behavior = Some(behavior);
                        }
                    });
                }
            }
            if let Some(mut new_behavior) = new_behavior {
                mem::swap(&mut self.behavior, &mut new_behavior);
                changed = true;
            }
            Ok(())
        })
        .inner?;
        Ok(changed)
    }
    pub fn editor(
        &self,
        response: Response,
        context: &Context,
    ) -> Result<Option<Fusion>, ExpressionError> {
        let mut edited: Option<Fusion> = None;
        let team = Team::get(
            context
                .get_parent(self.entity())
                .to_e("Fusion parent not found")?,
            context,
        )
        .to_e("Team not found")?;
        let units = team.roster_units_load(context);
        response
            .on_hover_ui(|ui| {
                self.show_card(context, ui).ui(ui);
            })
            .bar_menu(|ui| {
                ui.menu_button("add unit", |ui| {
                    for unit in &units {
                        if self.units.contains(&unit.id()) {
                            continue;
                        }
                        match unit.show_tag(context.clone().set_owner(unit.entity()), ui) {
                            Ok(response) => {
                                if response.clicked() {
                                    let mut fusion = self.clone();
                                    fusion.units.push(unit.id());
                                    edited = Some(fusion);
                                }
                            }
                            Err(e) => {
                                e.cstr().label(ui);
                            }
                        }
                    }
                });
                if !self.units.is_empty() {
                    ui.menu_button("remove unit", |ui| {
                        for unit in self.units.clone() {
                            if unit.cstr().button(ui).clicked() {
                                let mut fusion = self.clone();
                                fusion.remove_unit(unit);
                                edited = Some(fusion);
                            }
                        }
                    });
                    ui.menu_button("edit", |ui| {
                        let mut fusion = self.clone();
                        match fusion.show_editor(context, ui) {
                            Ok(c) => {
                                if c {
                                    edited = Some(fusion);
                                }
                            }
                            Err(e) => e.cstr().notify_error_op(),
                        }
                    });
                }
            });
        Ok(edited)
    }
    pub fn slots_editor(
        team: Entity,
        context: &Context,
        ui: &mut Ui,
        on_empty: impl FnOnce(&mut Ui),
        on_edited: impl FnOnce(Fusion),
    ) -> Result<(), ExpressionError> {
        let team = context
            .get_component::<Team>(team)
            .to_e("Failed to get Team component")?;
        let fusions: HashMap<usize, &Fusion> = HashMap::from_iter(
            team.fusions_load(context)
                .into_iter()
                .map(|f| (f.slot as usize, f)),
        );
        let team_slots = global_settings().team_slots as usize;
        ui.columns(team_slots, |ui| {
            for i in (0..team_slots).rev() {
                let ui = &mut ui[i];
                let i = team_slots - i - 1;
                if let Some(fusion) = fusions.get(&i) {
                    let response = slot_rect_button(ui, |rect, ui| {
                        fusion.paint(rect, context, ui).ui(ui);
                    });
                    match fusion.editor(response, context) {
                        Ok(edited) => {
                            if let Some(fusion) = edited {
                                on_edited(fusion);
                                return;
                            }
                        }
                        Err(e) => {
                            e.cstr().label(ui);
                        }
                    }
                } else {
                    on_empty(ui);
                    return;
                }
            }
        });
        Ok(())
    }
}
