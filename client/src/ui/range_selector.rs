use super::*;

#[derive(Debug, Clone, Copy, PartialEq)]
enum DragState {
    None,
    TopBorder,
    BottomBorder,
}

/// A widget for selecting a range from a vertical list of items with draggable borders.
///
/// The widget automatically measures each item's height by rendering them and uses
/// those measurements for proper border positioning and drag detection.
pub struct RangeSelector {
    range_start: u8,
    range_length: u8,
    max_items: u8,
    border_thickness: f32,
    drag_threshold: f32,
    show_drag_hints: bool,
    show_debug_info: bool,
    id: egui::Id,
}

impl RangeSelector {
    pub fn new(max_items: u8) -> Self {
        Self {
            range_start: 0,
            range_length: 0,
            max_items,
            border_thickness: 4.0,
            drag_threshold: 8.0,
            show_drag_hints: true,
            show_debug_info: false,
            id: egui::Id::new("range_selector"),
        }
    }

    pub fn range(mut self, start: u8, length: u8) -> Self {
        self.range_start = start.min(self.max_items.saturating_sub(1));
        self.range_length = length.min(self.max_items.saturating_sub(self.range_start));
        self
    }

    pub fn border_thickness(mut self, thickness: f32) -> Self {
        self.border_thickness = thickness.max(1.0);
        self
    }

    pub fn drag_threshold(mut self, threshold: f32) -> Self {
        self.drag_threshold = threshold.max(1.0);
        self
    }

    pub fn show_drag_hints(mut self, show: bool) -> Self {
        self.show_drag_hints = show;
        self
    }

    pub fn show_debug_info(mut self, show: bool) -> Self {
        self.show_debug_info = show;
        self
    }

    pub fn id(mut self, id: egui::Id) -> Self {
        self.id = id;
        self
    }

    pub fn ui<F>(
        mut self,
        ui: &mut Ui,
        context: &ClientContext,
        render_item: F,
    ) -> (Response, Option<(u8, u8)>)
    where
        F: Fn(&mut Ui, &ClientContext, usize, bool) -> Result<(), ExpressionError>,
    {
        let mut range_changed = None;

        // Extract colors from egui style
        let selected_color;
        let unselected_color;
        let border_color;
        let border_hover_color;
        let border_drag_area_color;
        let panel_fill;
        {
            let style = ui.style();
            let visuals = &style.visuals;
            selected_color = visuals.widgets.active.bg_fill;
            unselected_color = visuals.widgets.inactive.weak_bg_fill;
            border_color = visuals.widgets.active.fg_stroke.color;
            border_hover_color = visuals.widgets.hovered.fg_stroke.color;
            border_drag_area_color = visuals.widgets.inactive.bg_fill;
            panel_fill = visuals.panel_fill;
        }

        // First pass: measure all items to get their heights and positions
        let mut item_heights = Vec::with_capacity(self.max_items as usize);
        let mut item_positions = Vec::with_capacity(self.max_items as usize);
        let mut total_height = 0.0;

        for i in 0..self.max_items {
            item_positions.push(total_height);

            let available_width = ui.available_width();
            let is_in_range = i >= self.range_start && i < self.range_start + self.range_length;

            // Create invisible UI for measurement
            let invisible_rect = Rect::from_min_size(
                egui::Pos2::new(-10000.0, -10000.0),
                egui::Vec2::new(available_width, 1000.0),
            );
            let mut measure_ui = ui.new_child(
                UiBuilder::new()
                    .max_rect(invisible_rect)
                    .invisible()
                    .layout(egui::Layout::top_down(egui::Align::LEFT)),
            );

            let before_cursor = measure_ui.cursor();
            let _ = render_item(&mut measure_ui, context, i as usize, is_in_range);
            let after_cursor = measure_ui.cursor();

            let item_height = (after_cursor.top() - before_cursor.top()).max(20.0);
            item_heights.push(item_height);
            total_height += item_height;
        }

        let widget_height = total_height + self.border_thickness;

        let (rect, response) = ui.allocate_exact_size(
            egui::Vec2::new(ui.available_width(), widget_height),
            Sense::click_and_drag(),
        );

        if ui.is_rect_visible(rect) {
            // Calculate border positions using measured heights
            let top_border_y = if self.range_start == 0 {
                rect.min.y
            } else {
                rect.min.y + item_positions[self.range_start as usize]
            };

            let bottom_border_index = (self.range_start + self.range_length) as usize;
            let bottom_border_y = if bottom_border_index >= item_positions.len() {
                rect.min.y + total_height
            } else {
                rect.min.y + item_positions[bottom_border_index]
            };

            // Create hit areas for borders
            let top_hit_rect = Rect::from_min_size(
                egui::Pos2::new(rect.min.x, top_border_y),
                egui::Vec2::new(rect.width(), self.drag_threshold),
            );
            let bottom_hit_rect = Rect::from_min_size(
                egui::Pos2::new(rect.min.x, bottom_border_y - self.drag_threshold),
                egui::Vec2::new(rect.width(), self.drag_threshold),
            );

            let top_border_hovered = ui.rect_contains_pointer(top_hit_rect);
            let bottom_border_hovered = ui.rect_contains_pointer(bottom_hit_rect);

            // Get drag state from memory
            let mut drag_state = ui.memory_mut(|mem| {
                mem.data
                    .get_persisted::<DragState>(self.id)
                    .unwrap_or(DragState::None)
            });

            // Handle drag state initialization
            if response.drag_started() {
                if let Some(pointer_pos) = ui.ctx().pointer_latest_pos() {
                    if (pointer_pos.y - top_border_y).abs()
                        < (pointer_pos.y - bottom_border_y).abs()
                    {
                        drag_state = DragState::TopBorder;
                    } else {
                        drag_state = DragState::BottomBorder;
                    }
                }
            }

            // Handle drag end
            if response.drag_stopped() {
                drag_state = DragState::None;
            }

            // Store drag state in memory
            ui.memory_mut(|mem| {
                mem.data.insert_persisted(self.id, drag_state);
            });

            // Handle active dragging
            if response.dragged() && drag_state != DragState::None {
                if let Some(pointer_pos) = ui.ctx().pointer_latest_pos() {
                    let relative_y = pointer_pos.y - rect.min.y;

                    // Find which item the pointer is over by checking cumulative heights
                    let mut item_index = 0;
                    for (i, &pos) in item_positions.iter().enumerate() {
                        if relative_y >= pos {
                            item_index = i;
                        } else {
                            break;
                        }
                    }
                    let clamped_index =
                        (item_index as i32).clamp(0, self.max_items as i32 - 1) as u8;

                    match drag_state {
                        DragState::TopBorder => {
                            // Move top border - adjust start position only
                            let range_end = self.range_start + self.range_length;
                            let new_start = clamped_index.min(range_end.saturating_sub(1));

                            if new_start != self.range_start {
                                let new_length = range_end - new_start;
                                range_changed = Some((new_start, new_length));
                                self.range_start = new_start;
                                self.range_length = new_length;
                            }
                        }
                        DragState::BottomBorder => {
                            // Move bottom border - adjust end position only
                            let new_end = clamped_index + 1;
                            let min_end = self.range_start + 1;
                            let final_end = new_end.clamp(min_end, self.max_items);
                            let new_length = final_end - self.range_start;

                            if new_length != self.range_length {
                                range_changed = Some((self.range_start, new_length));
                                self.range_length = new_length;
                            }
                        }
                        DragState::None => {}
                    }
                }
            }

            // Set cursor when hovering over borders
            if top_border_hovered || bottom_border_hovered || drag_state != DragState::None {
                ui.ctx().set_cursor_icon(egui::CursorIcon::ResizeVertical);
            }

            // Draw drag area indicators when hovered or being dragged
            if top_border_hovered || drag_state == DragState::TopBorder {
                ui.painter()
                    .rect_filled(top_hit_rect, 1.0, border_drag_area_color);

                if self.show_drag_hints {
                    ui.painter().text(
                        egui::Pos2::new(rect.min.x + 5.0, top_border_y - 10.0),
                        egui::Align2::LEFT_CENTER,
                        "↕ Start",
                        egui::FontId::proportional(10.0),
                        border_hover_color,
                    );
                }
            }

            if bottom_border_hovered || drag_state == DragState::BottomBorder {
                ui.painter()
                    .rect_filled(bottom_hit_rect, 1.0, border_drag_area_color);

                if self.show_drag_hints {
                    ui.painter().text(
                        egui::Pos2::new(rect.min.x + 5.0, bottom_border_y + 10.0),
                        egui::Align2::LEFT_CENTER,
                        "↕ End",
                        egui::FontId::proportional(10.0),
                        border_hover_color,
                    );
                }
            }

            // Second pass: actually render the items with proper backgrounds
            for i in 0..self.max_items {
                let item_height = item_heights[i as usize];
                let item_rect = Rect::from_min_size(
                    rect.min + egui::Vec2::new(0.0, item_positions[i as usize]),
                    egui::Vec2::new(rect.width(), item_height),
                );

                let is_in_range = i >= self.range_start && i < self.range_start + self.range_length;
                let bg_color = if is_in_range {
                    selected_color
                } else {
                    unselected_color
                };

                // Draw background
                ui.painter().rect_filled(item_rect, 2.0, bg_color);

                // Create child UI for rendering the item
                let mut item_ui = ui.new_child(UiBuilder::new().max_rect(item_rect));
                let _ = render_item(&mut item_ui, context, i as usize, is_in_range);
            }

            // Draw borders with state-aware styling
            let top_border_rect = Rect::from_min_size(
                egui::Pos2::new(rect.min.x, top_border_y - self.border_thickness / 2.0),
                egui::Vec2::new(rect.width(), self.border_thickness),
            );
            let bottom_border_rect = Rect::from_min_size(
                egui::Pos2::new(rect.min.x, bottom_border_y - self.border_thickness / 2.0),
                egui::Vec2::new(rect.width(), self.border_thickness),
            );

            let top_color = if drag_state == DragState::TopBorder {
                border_hover_color.gamma_multiply(1.5)
            } else if top_border_hovered {
                border_hover_color
            } else {
                border_color
            };

            let bottom_color = if drag_state == DragState::BottomBorder {
                border_hover_color.gamma_multiply(1.5)
            } else if bottom_border_hovered {
                border_hover_color
            } else {
                border_color
            };

            // Draw borders
            ui.painter().rect_filled(top_border_rect, 2.0, top_color);
            ui.painter()
                .rect_filled(bottom_border_rect, 2.0, bottom_color);

            // Add visual distinction with notches if border is thick enough
            if self.border_thickness >= 3.0 {
                // Top border - 3 wider notches
                let notch_width = 8.0;
                for i in 0..3 {
                    let x = rect.min.x + (i as f32 + 1.0) * rect.width() / 4.0 - notch_width / 2.0;
                    let notch_rect = Rect::from_min_size(
                        egui::Pos2::new(x, top_border_y - 1.0),
                        egui::Vec2::new(notch_width, 2.0),
                    );
                    ui.painter().rect_filled(notch_rect, 1.0, panel_fill);
                }

                // Bottom border - 5 narrower notches
                let notch_width = 4.0;
                for i in 0..5 {
                    let x = rect.min.x + (i as f32 + 1.0) * rect.width() / 6.0 - notch_width / 2.0;
                    let notch_rect = Rect::from_min_size(
                        egui::Pos2::new(x, bottom_border_y - 1.0),
                        egui::Vec2::new(notch_width, 2.0),
                    );
                    ui.painter().rect_filled(notch_rect, 1.0, panel_fill);
                }
            }

            // Optional debug info
            if self.show_debug_info {
                let debug_text = format!(
                    "Range: {}-{} (len: {}), State: {:?}",
                    self.range_start,
                    self.range_start + self.range_length - 1,
                    self.range_length,
                    drag_state
                );
                ui.painter().text(
                    egui::Pos2::new(rect.min.x + 5.0, rect.max.y - 15.0),
                    egui::Align2::LEFT_BOTTOM,
                    debug_text,
                    egui::FontId::monospace(10.0),
                    border_color,
                );
            }
        }

        (response, range_changed)
    }
}

impl Default for RangeSelector {
    fn default() -> Self {
        Self::new(0)
    }
}
