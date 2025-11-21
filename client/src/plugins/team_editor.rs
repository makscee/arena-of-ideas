use crate::prelude::*;
use crate::ui::{DndArea, MatRect};
use bevy_egui::egui::{DragAndDrop, Response};

const SIZE: egui::Vec2 = egui::Vec2::splat(70.0);

#[derive(Debug, Clone)]
pub struct DraggedUnit {
    pub unit_id: u64,
    pub from_location: UnitLocation,
}

#[derive(Debug, Clone, PartialEq)]
pub enum UnitLocation {
    Bench,
    Slot { index: i32 },
}

pub struct TeamEditor {
    empty_slot_actions: Vec<(
        String,
        Box<dyn Fn(&mut NTeam, i32, &ClientContext, &mut Ui)>,
    )>,
    filled_slot_actions: Vec<(
        String,
        Box<dyn Fn(&mut NTeam, u64, i32, &ClientContext, &mut Ui)>,
    )>,
    action_handler: Option<Box<dyn Fn(&ClientContext, &TeamAction) -> NodeResult<()>>>,
    drag_over_handler: Option<Box<dyn Fn(u64, u64, &ClientContext) -> DragOverResult>>,
}

pub enum DragOverResult {
    Swap,
    Fuse,
    Stack,
    Cancel,
}

pub enum TeamAction {
    MoveToBench {
        unit_id: u64,
    },
    MoveToEmptySlot {
        unit_id: u64,
        slot_index: i32,
    },
    MoveOverUnit {
        unit_id: u64,
        target_unit_id: u64,
    },
    CustomEmptySlotAction {
        name: String,
        slot_index: i32,
    },
    CustomFilledSlotAction {
        name: String,
        unit_id: u64,
        slot_index: i32,
    },
}

impl TeamEditor {
    pub fn new() -> Self {
        Self {
            empty_slot_actions: vec![],
            filled_slot_actions: vec![],
            action_handler: None,
            drag_over_handler: None,
        }
    }

    pub fn with_action_handler<F>(mut self, handler: F) -> Self
    where
        F: Fn(&ClientContext, &TeamAction) -> NodeResult<()> + 'static,
    {
        self.action_handler = Some(Box::new(handler));
        self
    }

    pub fn empty_slot_action<F>(mut self, name: String, action: F) -> Self
    where
        F: Fn(&mut NTeam, i32, &ClientContext, &mut Ui) + 'static,
    {
        self.empty_slot_actions.push((name, Box::new(action)));
        self
    }

    pub fn filled_slot_action<F>(mut self, name: String, action: F) -> Self
    where
        F: Fn(&mut NTeam, u64, i32, &ClientContext, &mut Ui) + 'static,
    {
        self.filled_slot_actions.push((name, Box::new(action)));
        self
    }

    pub fn with_drag_over_handler<F>(mut self, handler: F) -> Self
    where
        F: Fn(u64, u64, &ClientContext) -> DragOverResult + 'static,
    {
        self.drag_over_handler = Some(Box::new(handler));
        self
    }

    pub fn edit(mut self, team: &NTeam, ctx: &ClientContext, ui: &mut Ui) -> Option<NTeam> {
        let mut actions = Vec::new();
        let mut result_team = team.clone();

        self.draw_drag_visual(ui);

        ui.vertical(|ui| {
            self.render_slots(ui, &result_team, ctx, &mut actions);
            ui.separator();
            self.render_bench(ui, &result_team, ctx, &mut actions);
        });

        if !actions.is_empty() {
            if let Some(ref action_handler) = self.action_handler {
                for action in actions {
                    action_handler(ctx, &action).notify_error_op();
                }
                None
            } else {
                let mut has_changes = false;
                for action in actions {
                    match &action {
                        TeamAction::CustomEmptySlotAction { name, slot_index } => {
                            for (action_name, action_fn) in self.empty_slot_actions.iter() {
                                if action_name == name {
                                    action_fn(&mut result_team, *slot_index, ctx, ui);
                                    has_changes = true;
                                    break;
                                }
                            }
                        }
                        TeamAction::CustomFilledSlotAction {
                            name,
                            unit_id,
                            slot_index,
                        } => {
                            for (action_name, action_fn) in self.filled_slot_actions.iter() {
                                if action_name == name {
                                    action_fn(&mut result_team, *unit_id, *slot_index, ctx, ui);
                                    has_changes = true;
                                    break;
                                }
                            }
                        }
                        other => {
                            result_team.apply_action(other).notify_error_op();
                            has_changes = true;
                        }
                    }
                }

                if has_changes { Some(result_team) } else { None }
            }
        } else {
            None
        }
    }

    fn render_slots(
        &mut self,
        ui: &mut Ui,
        team: &NTeam,
        ctx: &ClientContext,
        actions: &mut Vec<TeamAction>,
    ) {
        ui.horizontal(|ui| {
            for slot in team.slots().unwrap().iter().sorted_by_key(|s| s.index) {
                self.render_slot(ui, slot, team, ctx, actions);
            }
        });
    }

    fn render_slot(
        &mut self,
        ui: &mut Ui,
        slot: &NTeamSlot,
        _team: &NTeam,
        ctx: &ClientContext,
        actions: &mut Vec<TeamAction>,
    ) {
        if let Some(unit_id) = slot.unit.id() {
            self.handle_slot_unit_interactions(ui, unit_id, slot.index, ctx, actions);
        } else {
            self.handle_empty_slot_interactions(ui, slot.index, ctx, actions);
        }
    }

    fn handle_slot_unit_interactions(
        &mut self,
        ui: &mut Ui,
        unit_id: u64,
        slot_index: i32,
        ctx: &ClientContext,
        actions: &mut Vec<TeamAction>,
    ) {
        let response = self.render_unit_with_representation(ui, unit_id, ctx);

        if response.drag_started() {
            let dragged_unit = DraggedUnit {
                unit_id,
                from_location: UnitLocation::Slot { index: slot_index },
            };
            DragAndDrop::set_payload(ui.ctx(), dragged_unit);
        }

        if let Some(dragged) = DndArea::<DraggedUnit>::new(response.rect)
            .id(format!("slot_{}_{}", slot_index, unit_id))
            .text_fn(ui, |dragged_unit| {
                if dragged_unit.unit_id != unit_id {
                    if let Some(ref handler) = self.drag_over_handler {
                        match handler(dragged_unit.unit_id, unit_id, ctx) {
                            DragOverResult::Swap => Some("Swap".to_string()),
                            DragOverResult::Fuse => Some("Fuse".to_string()),
                            DragOverResult::Stack => Some("Stack".to_string()),
                            DragOverResult::Cancel => None,
                        }
                    } else {
                        Some("Swap".to_string())
                    }
                } else {
                    None
                }
            })
            .ui(ui)
        {
            if dragged.unit_id != unit_id {
                actions.push(TeamAction::MoveOverUnit {
                    unit_id: dragged.unit_id,
                    target_unit_id: unit_id,
                });
            }
        }

        let response_for_hover = ui.interact(response.rect, response.id, egui::Sense::hover());
        response_for_hover.on_hover_ui(|ui| {
            if let Ok(unit) = ctx.load::<NUnit>(unit_id) {
                unit.as_card().compose(ctx, ui);
            }
        });

        response.show_menu(ui, |ui| {
            if ui.button("Move to Bench").clicked() {
                actions.push(TeamAction::MoveToBench { unit_id });
                ui.close();
            }

            for (name, _) in self.filled_slot_actions.iter() {
                if ui.button(name).clicked() {
                    actions.push(TeamAction::CustomFilledSlotAction {
                        name: name.clone(),
                        unit_id,
                        slot_index,
                    });
                    ui.close();
                    break;
                }
            }
        });
    }

    fn handle_empty_slot_interactions(
        &mut self,
        ui: &mut Ui,
        slot_index: i32,
        ctx: &ClientContext,
        actions: &mut Vec<TeamAction>,
    ) {
        let response = MatRect::new(SIZE).ui(ui, ctx);

        if let Some(dragged) = DndArea::<DraggedUnit>::new(response.rect)
            .id(format!("empty_slot_{}", slot_index))
            .text("Place here")
            .ui(ui)
        {
            actions.push(TeamAction::MoveToEmptySlot {
                unit_id: dragged.unit_id,
                slot_index,
            });
        }

        response.show_menu(ui, |ui| {
            for (name, _) in self.empty_slot_actions.iter() {
                if ui.button(name).clicked() {
                    actions.push(TeamAction::CustomEmptySlotAction {
                        name: name.clone(),
                        slot_index,
                    });
                    ui.close();
                    break;
                }
            }
        });
    }

    fn render_bench(
        &mut self,
        ui: &mut Ui,
        team: &NTeam,
        ctx: &ClientContext,
        actions: &mut Vec<TeamAction>,
    ) {
        ui.group(|ui| {
            ui.label("Bench");
            ui.horizontal_wrapped(|ui| {
                if let Ok(benched) = team.benched() {
                    for unit in benched {
                        self.handle_bench_unit_interactions(ui, unit.id, ctx, actions);
                    }
                }
            });

            let rect = ui.available_rect_before_wrap();
            if let Some(dragged) = DndArea::<DraggedUnit>::new(rect)
                .id("bench_drop_zone")
                .text("Move to Bench")
                .ui(ui)
            {
                actions.push(TeamAction::MoveToBench {
                    unit_id: dragged.unit_id,
                });
            }
        });
    }

    fn handle_bench_unit_interactions(
        &mut self,
        ui: &mut Ui,
        unit_id: u64,
        ctx: &ClientContext,
        _actions: &mut Vec<TeamAction>,
    ) {
        let response = self.render_unit_with_representation(ui, unit_id, ctx);

        if response.drag_started() {
            let dragged_unit = DraggedUnit {
                unit_id,
                from_location: UnitLocation::Bench,
            };
            DragAndDrop::set_payload(ui.ctx(), dragged_unit);
        }

        response.on_hover_ui(|ui| {
            if let Ok(unit) = ctx.load::<NUnit>(unit_id) {
                unit.as_card().compose(ctx, ui);
            }
        });
    }

    fn render_unit_with_representation(
        &self,
        ui: &mut Ui,
        unit_id: u64,
        ctx: &ClientContext,
    ) -> Response {
        if let Ok(unit) = ctx.load::<NUnit>(unit_id) {
            if let Ok(rep) = unit.representation_ref(ctx) {
                MatRect::new(SIZE)
                    .add_mat(&rep.material, unit.id)
                    .unit_rep_with_default(unit.id)
                    .ui(ui, ctx)
            } else {
                MatRect::new(SIZE).ui(ui, ctx)
            }
        } else {
            ui.label(format!("Unit #{}", unit_id))
        }
    }

    fn draw_drag_visual(&self, ui: &mut Ui) {
        if DragAndDrop::has_payload_of_type::<DraggedUnit>(ui.ctx()) {
            if let Some(pointer_pos) = ui.ctx().pointer_latest_pos() {
                if let Some(_payload) = DragAndDrop::payload::<DraggedUnit>(ui.ctx()) {
                    let painter = ui.painter();
                    let rect = egui::Rect::from_center_size(pointer_pos, SIZE * 0.8);

                    painter.rect_filled(
                        rect,
                        4.0,
                        egui::Color32::from_rgba_premultiplied(50, 50, 50, 200),
                    );
                    painter.rect_stroke(
                        rect,
                        4.0,
                        egui::Stroke::new(2.0, egui::Color32::WHITE),
                        egui::epaint::StrokeKind::Middle,
                    );
                }
            }
        }
    }
}

impl NTeam {
    pub fn apply_action(&mut self, action: &TeamAction) -> NodeResult<()> {
        match action {
            TeamAction::MoveToBench { unit_id } => {
                self.move_unit_to_bench(*unit_id)?;
            }
            TeamAction::MoveToEmptySlot {
                unit_id,
                slot_index,
            } => {
                self.move_unit_to_slot(*unit_id, *slot_index)?;
            }
            TeamAction::MoveOverUnit {
                unit_id,
                target_unit_id,
            } => {
                self.swap_units(*unit_id, *target_unit_id)?;
            }
            TeamAction::CustomEmptySlotAction { .. } => {}
            TeamAction::CustomFilledSlotAction { .. } => {}
        }
        Ok(())
    }

    fn move_unit_to_bench(&mut self, unit_id: u64) -> NodeResult<()> {
        let unit = self.find_and_unlink_unit(unit_id)?;
        self.benched_push(unit)?;
        Ok(())
    }

    fn move_unit_to_slot(&mut self, unit_id: u64, slot_index: i32) -> NodeResult<()> {
        let unit = self.find_and_unlink_unit(unit_id)?;
        let slot = self
            .slots
            .get_mut()?
            .iter_mut()
            .find(|s| s.index == slot_index)
            .to_not_found()?;
        slot.unit = Owned::new_loaded(unit);
        Ok(())
    }

    fn swap_units(&mut self, unit_id1: u64, unit_id2: u64) -> NodeResult<()> {
        let loc1 = self.find_unit_location(unit_id1)?;
        let loc2 = self.find_unit_location(unit_id2)?;

        match (loc1, loc2) {
            (UnitLocation::Slot { index: idx1 }, UnitLocation::Slot { index: idx2 }) => {
                if let Ok(slots) = self.slots.get_mut() {
                    if idx1 != idx2 {
                        let mut unit1_ref = Owned::none();
                        let mut unit2_ref = Owned::none();

                        for slot in slots.iter_mut() {
                            if slot.index == idx1 {
                                unit1_ref = slot.unit.clone();
                                slot.unit = Owned::none();
                            } else if slot.index == idx2 {
                                unit2_ref = slot.unit.clone();
                                slot.unit = Owned::none();
                            }
                        }

                        for slot in slots.iter_mut() {
                            if slot.index == idx1 {
                                slot.unit = unit2_ref.clone();
                            } else if slot.index == idx2 {
                                slot.unit = unit1_ref.clone();
                            }
                        }
                    }
                }
            }
            _ => {}
        }

        Ok(())
    }

    fn find_slot_with_unit_mut(&mut self, unit_id: u64) -> NodeResult<Option<&mut NTeamSlot>> {
        if let Ok(slots) = self.slots.get_mut() {
            for slot in slots {
                if slot.unit.id() == Some(unit_id) {
                    return Ok(Some(slot));
                }
            }
        }
        Ok(None)
    }

    fn find_unit_location(&self, unit_id: u64) -> NodeResult<UnitLocation> {
        if let Ok(slots) = self.slots() {
            for slot in slots {
                if slot.unit.id() == Some(unit_id) {
                    return Ok(UnitLocation::Slot { index: slot.index });
                }
            }
        }

        if let Ok(benched) = self.benched() {
            for unit in benched {
                if unit.id == unit_id {
                    return Ok(UnitLocation::Bench);
                }
            }
        }

        Err(NodeError::custom("Unit not found"))
    }

    fn find_and_unlink_unit(&mut self, unit_id: u64) -> NodeResult<NUnit> {
        if let Some(slot) = self.find_slot_with_unit_mut(unit_id)? {
            if let Owned::Loaded(unit) = slot.unit.take() {
                return Ok(unit);
            }
        }

        if let Ok(benched) = self.benched.get_mut() {
            if let Some(unit_pos) = benched.iter().position(|u| u.id == unit_id) {
                return Ok(benched.remove(unit_pos));
            }
        }

        Err(NodeError::not_found(unit_id))
    }
}
