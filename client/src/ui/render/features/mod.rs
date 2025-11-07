use super::*;
use crate::ui::render::composers::recursive::{RecursiveField, RecursiveFieldMut};

mod features_impl;
mod frecursive;

pub use frecursive::*;

/// Feature for types that can provide a title
pub trait FTitle {
    fn title(&self, ctx: &ClientContext) -> Cstr;
}

/// Feature for types that can provide a colored title
pub trait FColoredTitle: FTitle {
    fn title_color(&self, ctx: &ClientContext) -> Color32;

    fn colored_title(&self, ctx: &ClientContext) -> Cstr {
        self.title(ctx).cstr_c(self.title_color(ctx))
    }
}

/// Feature for types that can provide a description
pub trait FDescription {
    fn description_cstr(&self, ctx: &ClientContext) -> Cstr;
}

/// Feature for types that can provide an icon or short representation
pub trait FIcon {
    fn icon(&self, ctx: &ClientContext) -> Cstr;
}

/// Feature for types that can provide a visual representation
pub trait FRepresentation {
    fn representation(&self, ctx: &ClientContext) -> Result<Material, NodeError>;
}

/// Feature for types that can provide stats/variables
pub trait FStats {
    fn stats(&self, ctx: &ClientContext) -> Vec<(VarName, VarValue)>;
}

/// Feature for types that can provide a compact tag view
pub trait FTag {
    fn tag_name(&self, ctx: &ClientContext) -> Cstr;
    fn tag_value(&self, ctx: &ClientContext) -> Option<Cstr>;
    fn tag_color(&self, ctx: &ClientContext) -> Color32;
}

/// Feature for types that have an expanded info string
pub trait FInfo {
    fn info(&self, ctx: &ClientContext) -> Cstr;
}

/// Feature for types that can be recursively traversed
pub trait FRecursive {
    /// Get inner fields for read-only traversal
    fn get_inner_fields(&self) -> Vec<RecursiveField<'_>> {
        vec![]
    }

    /// Get mutable inner fields for editing
    fn get_inner_fields_mut(&mut self) -> Vec<RecursiveFieldMut<'_>> {
        vec![]
    }

    /// Convert to a recursive value for unified handling
    fn to_recursive_value(&self) -> RecursiveValue<'_>;

    /// Convert to a mutable recursive value for unified handling
    fn to_recursive_value_mut(&mut self) -> RecursiveValueMut<'_>;

    fn move_inner_fields_from(&mut self, other: &mut impl FRecursive) {
        self.get_inner_fields_mut().move_from(other);
    }
}

/// Feature for types that can be displayed
pub trait FDisplay {
    fn display(&self, ctx: &ClientContext, ui: &mut Ui) -> Response;
}

pub trait FDisplayRecursive: FRecursive + FDisplay {
    fn display_recursive(&self, ctx: &ClientContext, ui: &mut Ui) -> Response {
        ui.scope(|ui| {
            ui.spacing_mut().item_spacing.x = 0.0;
            let mut resp = self.display(ctx, ui);
            for field in self.get_inner_fields() {
                recursive_value_match!(field.value, v, {
                    resp |= "[tw (]".cstr().label(ui);
                    resp |= v.display_recursive(ctx, ui);
                    resp |= "[tw )]".cstr().label(ui);
                });
            }
            resp
        })
        .inner
    }
}
impl<T: FRecursive + FDisplay> FDisplayRecursive for T {}

pub trait FTitleRecursive: FRecursive + FTitle {
    fn title_recursive(&self, ctx: &ClientContext) -> Cstr {
        let mut c = self.title(ctx);
        for field in self.get_inner_fields() {
            recursive_value_match!(field.value, v, {
                c += "[tw (]";
                c.push_str(&v.title_recursive(ctx));
                c += "[tw )]";
            });
        }
        c
    }
}
impl<T: FRecursive + FTitle> FTitleRecursive for T {}

/// Feature for types that can be edited
pub trait FEdit {
    fn edit(&mut self, ui: &mut Ui) -> Response;
    fn edit_self(&mut self, ui: &mut Ui) -> Response {
        self.edit(ui)
    }
}

/// Feature for types that can be copied to clipboard
pub trait FCopy: StringData {
    fn copy_to_clipboard(&self) {
        clipboard_set(self.get_data());
    }
}

/// Feature for types that can be pasted from clipboard
pub trait FPaste: StringData + Default {
    fn paste_from_clipboard() -> Option<Self> {
        clipboard_get().and_then(|data| {
            let mut item = Self::default();
            item.inject_data(&data).ok().map(|_| item)
        })
    }
}

/// Feature for types that have a rating
pub trait FRating {
    fn rating(&self, ctx: &ClientContext) -> Option<i32>;
}

/// Feature for types that can be expanded/collapsed
pub trait FExpandable {
    fn default_expanded(&self) -> bool {
        false
    }
}

/// Feature for types that can be selected
pub trait FSelectable: PartialEq + Clone {
    fn selection_label(&self) -> Cstr;
}

/// Feature for types that have validation
pub trait FValidate {
    fn validate(&self, ctx: &ClientContext) -> Result<(), Vec<String>>;
}

/// Feature for types that can be searched
pub trait FSearchable {
    fn search_text(&self, ctx: &ClientContext) -> String;
    fn matches_search(&self, query: &str, ctx: &ClientContext) -> bool {
        self.search_text(ctx)
            .to_lowercase()
            .contains(&query.to_lowercase())
    }
}

/// Feature for types that can be filtered
pub trait FFilterable {
    type Filter;

    fn matches_filter(&self, filter: &Self::Filter, ctx: &ClientContext) -> bool;
}

/// Feature for types that can be sorted
pub trait FSortable {
    type SortKey: Ord;

    fn sort_key(&self, ctx: &ClientContext) -> Self::SortKey;
}

/// Feature for types that have a color
pub trait FColor {
    fn color(&self, ctx: &ClientContext) -> Color32;
}

/// Feature for types that can be previewed
pub trait FPreview {
    fn preview(&self, ctx: &ClientContext, ui: &mut Ui, rect: Rect);
}

/// Feature for types that have help/documentation
pub trait FHelp {
    fn help_text(&self) -> &'static str;
    fn help_url(&self) -> Option<&'static str> {
        None
    }
}

/// Feature for types that track changes
pub trait FDirty {
    fn is_dirty(&self) -> bool;
    fn mark_clean(&mut self);
    fn mark_dirty(&mut self);
}

/// Feature for types that can provide a compact view with hover details
pub trait FCompactView {
    /// Render the compact view
    fn render_compact(&self, ctx: &ClientContext, ui: &mut Ui);

    /// Render the hover view
    fn render_hover(&self, ctx: &ClientContext, ui: &mut Ui);
}

// FPlaceholder - creates placeholder instances for nodes
pub trait FPlaceholder {
    fn placeholder() -> Self;
}

const CARD_SIZE: egui::Vec2 = egui::vec2(180.0, 200.0);
pub trait FCard: FDescription + FTitle + FStats + FPreview + Node {
    fn render_card(&self, ctx: &ClientContext, ui: &mut Ui) -> Response
    where
        Self: Sized,
    {
        let (rect, response) = ui.allocate_exact_size(CARD_SIZE, Sense::click_and_drag());
        let ui = &mut ui.new_child(
            UiBuilder::new()
                .max_rect(rect)
                .layout(Layout::top_down(Align::Center)),
        );
        ui.set_clip_rect(rect);
        Frame::new()
            .corner_radius(ROUNDING)
            .stroke(ctx.color().stroke())
            .show(ui, |ui| {
                self.title(ctx).cstr_s(CstrStyle::Heading2).label(ui);

                // Preview in remaining space
                let available_rect = ui
                    .available_rect_before_wrap()
                    .shrink2(egui::vec2(5.0, 0.0));
                if available_rect.height() > 10.0 {
                    let ui = &mut ui.new_child(UiBuilder::new().max_rect(available_rect));
                    self.preview(ctx, ui, available_rect);
                }
                let stats = self.stats(ctx);
                if !stats.is_empty() {
                    ui.horizontal_wrapped(|ui| {
                        for (var, var_value) in self.stats(ctx) {
                            TagWidget::new_var_value(var, var_value).ui(ui);
                        }
                    });
                }
                let dsc = self.description_cstr(ctx);
                ui.with_layout(Layout::bottom_up(Align::Center), |ui| {
                    if !dsc.is_empty() {
                        let fill = colorix().subtle_background();
                        let stroke = colorix().ui_element_border_and_focus_rings().stroke();
                        Frame::new()
                            .fill(fill)
                            .stroke(stroke)
                            .corner_radius(ROUNDING)
                            .outer_margin(3)
                            .inner_margin(3)
                            .show(ui, |ui| {
                                self.description_cstr(ctx)
                                    .cstr_s(CstrStyle::Small)
                                    .label_w(ui);
                            });
                    }
                });
            });

        response
    }
}
