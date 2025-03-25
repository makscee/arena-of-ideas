use super::*;

pub struct FusionEditorPlugin;

impl Plugin for FusionEditorPlugin {
    fn build(&self, app: &mut App) {}
}

#[derive(Resource)]
struct FusionEditorData {
    fusion: Fusion,
    on_save: Box<dyn Fn(Fusion, &mut World) -> Result<(), ExpressionError> + Send + Sync>,
}

impl FusionEditorPlugin {
    pub fn edit_entity(
        entity: Entity,
        world: &mut World,
        on_save: impl Fn(Fusion, &mut World) -> Result<(), ExpressionError> + Send + Sync + 'static,
    ) -> Result<(), ExpressionError> {
        let fusion = Fusion::pack(entity, world).to_e("Failed to pack Fusion")?;
        world.insert_resource(FusionEditorData {
            fusion,
            on_save: Box::new(on_save),
        });
        GameState::FusionEditor.set_next(world);
        Ok(())
    }
    pub fn pane_roster(ui: &mut Ui, world: &mut World) -> Result<(), ExpressionError> {
        world.resource_scope(|world, mut r: Mut<FusionEditorData>| {
            let FusionEditorData { fusion, on_save: _ } = r.as_mut();
            let mut changed = false;
            for unit in fusion.team_load(world)?.roster_units_load(world) {
                let selected = fusion.units.contains(&unit.name);
                let stats = unit.description_load(world)?.stats_load(world)?;
                dark_frame()
                    .stroke(if selected {
                        Stroke::new(1.0, tokens_global().ui_element_border_and_focus_rings())
                    } else {
                        Stroke::new(1.0, tokens_global().subtle_borders_and_separators())
                    })
                    .show(ui, |ui| {
                        // show_unit_tag(
                        //     unit,
                        //     stats,
                        //     Context::new_world(world).set_owner(unit.entity()),
                        //     ui,
                        // );
                        if "select".cstr_s(CstrStyle::Bold).button(ui).clicked() {
                            changed = true;
                            if selected {
                                let i = fusion.units.iter().position(|u| unit.name.eq(u)).unwrap();
                                fusion.remove_unit(i as u8);
                            } else {
                                fusion.units.push(unit.name.clone());
                            }
                        }
                    });
            }
            if changed {
                world.entity_mut(fusion.entity()).insert(fusion.clone());
                Fusion::init(fusion.entity(), world).log();
            }
            Ok(())
        })
    }
    pub fn pane_triggers(ui: &mut Ui, world: &mut World) {
        // world.resource_scope(|world, mut r: Mut<FusionEditorData>| {
        //     let FusionEditorData { fusion, on_save: _ } = r.as_mut();
        //     let context = &Context::new_world(&world);
        //     ui.vertical(|ui| {
        //         for u in 0..fusion.units.len() {
        //             let triggers = &fusion.get_behavior(u as u8, context).unwrap().triggers;
        //             for (
        //                 t,
        //                 Reaction {
        //                     trigger,
        //                     actions: _,
        //                 },
        //             ) in triggers.iter().enumerate()
        //             {
        //                 let t_ref = UnitTriggerRef {
        //                     unit: u as u8,
        //                     trigger: t as u8,
        //                 };
        //                 let selected = fusion.triggers.iter().any(|(r, _)| r.eq(&t_ref));
        //                 dark_frame()
        //                     .stroke(if selected {
        //                         Stroke::new(
        //                             1.0,
        //                             tokens_global().ui_element_border_and_focus_rings(),
        //                         )
        //                     } else {
        //                         Stroke::new(1.0, tokens_global().subtle_borders_and_separators())
        //                     })
        //                     .show(ui, |ui| {
        //                         ui.horizontal(|ui| {
        //                             trigger.show(None, context, ui);
        //                             if ">".cstr_s(CstrStyle::Bold).button(ui).clicked() {
        //                                 if selected {
        //                                     fusion.remove_trigger(t_ref);
        //                                 } else {
        //                                     fusion.triggers.push((
        //                                         UnitTriggerRef {
        //                                             unit: u as u8,
        //                                             trigger: t as u8,
        //                                         },
        //                                         default(),
        //                                     ));
        //                                 }
        //                             }
        //                         })
        //                     });
        //             }
        //         }
        //     });
        // });
    }
    pub fn pane_actions(ui: &mut Ui, world: &mut World) {
        // world.resource_scope(|world, mut r: Mut<FusionEditorData>| {
        //     let FusionEditorData { fusion, on_save: _ } = r.as_mut();
        //     let context = &Context::new_world(&world);
        //     if fusion.triggers.is_empty() {
        //         "Select at least one trigger"
        //             .cstr_s(CstrStyle::Bold)
        //             .label(ui);
        //         return;
        //     }
        //     for u in 0..fusion.units.len() {
        //         let reaction = &fusion.get_behavior(u as u8, context).unwrap();
        //         let triggers = &reaction.triggers;
        //         let entity = reaction.entity();
        //         for (
        //             t,
        //             Reaction {
        //                 trigger: _,
        //                 actions,
        //             },
        //         ) in triggers.iter().enumerate()
        //         {
        //             for (a, action) in actions.0.iter().enumerate() {
        //                 let a_ref = UnitActionRef {
        //                     unit: u as u8,
        //                     trigger: t as u8,
        //                     action: a as u8,
        //                 };
        //                 let selected = fusion
        //                     .triggers
        //                     .iter()
        //                     .any(|(_, a)| a.iter().any(|a| a_ref.eq(a)));
        //                 dark_frame()
        //                     .stroke(if selected {
        //                         Stroke::new(1.0, tokens_global().hovered_ui_element_border())
        //                     } else {
        //                         Stroke::new(1.0, tokens_global().subtle_borders_and_separators())
        //                     })
        //                     .show(ui, |ui| {
        //                         ui.horizontal(|ui| {
        //                             action.show(None, context.clone().set_owner(entity), ui);
        //                             if ">".cstr_s(CstrStyle::Bold).button(ui).clicked() {
        //                                 if selected {
        //                                     fusion.remove_action(a_ref);
        //                                 } else {
        //                                     fusion.triggers.last_mut().unwrap().1.push(a_ref);
        //                                 }
        //                             }
        //                         })
        //                     });
        //             }
        //         }
        //     }
        // });
    }
    pub fn pane_fusion_result(ui: &mut Ui, world: &mut World) -> Result<(), ExpressionError> {
        let mut save = false;
        // world.resource_scope(|world, mut r: Mut<FusionEditorData>| {
        //     let FusionEditorData { fusion, on_save: _ } = r.as_mut();
        //     let context = &Context::new_world(&world).set_owner(fusion.entity()).take();
        //     let size = ui.available_size();
        //     let size = size.x.at_most(size.y).at_least(150.0);
        //     ui.horizontal(|ui| {
        //         let rect = ui
        //             .allocate_exact_size(egui::vec2(size, size), Sense::hover())
        //             .0;
        //         fusion.paint(rect, &context, ui).log();
        //         unit_rep().paint(rect.shrink(15.0), context, ui).log();

        //         ui.vertical(|ui| {
        //             "Result".cstr_s(CstrStyle::Heading2).label(ui);
        //             let mut remove_t = None;
        //             let mut remove_a = None;
        //             let mut swap = None;
        //             for (t_i, (t_ref, actions)) in fusion.triggers.iter().enumerate() {
        //                 let trigger = fusion
        //                     .get_trigger(t_ref.unit, t_ref.trigger, context)
        //                     .unwrap();
        //                 ui.horizontal(|ui| {
        //                     if "-".cstr_cs(RED, CstrStyle::Bold).button(ui).clicked() {
        //                         remove_t = Some(*t_ref);
        //                     }
        //                     trigger.show(None, context, ui);
        //                 });
        //                 dark_frame().show(ui, |ui| {
        //                     for (a_i, a_ref) in actions.iter().enumerate() {
        //                         let (entity, action) = fusion.get_action(a_ref, context).unwrap();
        //                         ui.horizontal(|ui| {
        //                             if "-".cstr_cs(RED, CstrStyle::Bold).button(ui).clicked() {
        //                                 remove_a = Some(*a_ref);
        //                             }
        //                             if (t_i > 0 || a_i > 0)
        //                                 && "^"
        //                                     .cstr_cs(
        //                                         tokens_global().high_contrast_text(),
        //                                         CstrStyle::Bold,
        //                                     )
        //                                     .button(ui)
        //                                     .clicked()
        //                             {
        //                                 if a_i == 0 {
        //                                     swap = Some((
        //                                         (t_i, a_i),
        //                                         (t_i - 1, fusion.triggers[t_i - 1].1.len()),
        //                                     ));
        //                                 } else {
        //                                     swap = Some(((t_i, a_i), (t_i, a_i - 1)));
        //                                 }
        //                             }
        //                             if (t_i + 1 < fusion.triggers.len() || a_i + 1 < actions.len())
        //                                 && "v"
        //                                     .cstr_cs(
        //                                         tokens_global().high_contrast_text(),
        //                                         CstrStyle::Bold,
        //                                     )
        //                                     .button(ui)
        //                                     .clicked()
        //                             {
        //                                 if a_i == actions.len() - 1 {
        //                                     swap = Some(((t_i, a_i), (t_i + 1, 0)));
        //                                 } else {
        //                                     swap = Some(((t_i, a_i), (t_i, a_i + 1)));
        //                                 }
        //                             }
        //                             action.show(None, context.clone().set_owner(entity), ui);
        //                         });
        //                     }
        //                 });
        //             }
        //             if let Some(((from_t, from_a), (to_t, to_a))) = swap {
        //                 let action = fusion.triggers[from_t].1.remove(from_a);
        //                 fusion.triggers[to_t].1.insert(to_a, action);
        //             }
        //             if let Some(r) = remove_a {
        //                 fusion.remove_action(r);
        //             }
        //             if let Some(r) = remove_t {
        //                 fusion.remove_trigger(r);
        //             }
        //             if "save"
        //                 .cstr_cs(tokens_global().high_contrast_text(), CstrStyle::Heading2)
        //                 .button(ui)
        //                 .clicked()
        //             {
        //                 save = true;
        //             }
        //         });
        //     });
        // });
        if save {
            world.resource_scope(|world, d: Mut<FusionEditorData>| {
                let fusion = d.fusion.clone();
                let entity = fusion.entity();
                fusion.unpack(entity, world);
                (d.on_save)(d.fusion.clone(), world)
            })?;
        }
        Ok(())
    }
}
