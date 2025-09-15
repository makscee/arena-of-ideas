use super::*;

pub mod recursive;
pub use recursive::*;

mod advanced_composers;
pub use advanced_composers::*;

mod selector_composer;
pub use selector_composer::*;

pub mod menu;
pub use menu::*;

/// Base trait for composers that transform data into UI
pub trait Composer<T> {
    fn compose(&self, data: &T, context: &Context, ui: &mut Ui) -> Response;
}

/// Trait for composers that can modify data
pub trait ComposerMut<T> {
    fn compose_mut(&self, data: &mut T, context: &Context, ui: &mut Ui) -> bool;
}

/// Composer for rendering titles
pub struct TitleComposer;

impl<T: FTitle> Composer<T> for TitleComposer {
    fn compose(&self, data: &T, context: &Context, ui: &mut Ui) -> Response {
        data.title(context).button(ui)
    }
}

/// Composer for rendering tags
pub struct TagComposer;

impl<T: FTag> Composer<T> for TagComposer {
    fn compose(&self, data: &T, context: &Context, ui: &mut Ui) -> Response {
        let name = data.tag_name(context);
        let color = data.tag_color(context);

        if let Some(value) = data.tag_value(context) {
            TagWidget::new_name_value(name, color, value).ui(ui)
        } else {
            TagWidget::new_name(name, color).ui(ui)
        }
    }
}

/// Composer for rendering cards
pub struct CardComposer;

impl<T: FTitle + FDescription + FStats> Composer<T> for CardComposer {
    fn compose(&self, data: &T, context: &Context, ui: &mut Ui) -> Response {
        let color = context.color(ui);

        let inner_response = Frame::new()
            .inner_margin(2)
            .corner_radius(ROUNDING)
            .stroke(color.stroke())
            .show(ui, |ui| {
                let resp = ui
                    .horizontal(|ui| {
                        let resp = data.title(context).button(ui);

                        // Optional menu button
                        let _ = RectButton::new_size(5.0.v2()).ui(ui, |color, rect, _, ui| {
                            ui.painter()
                                .circle_filled(rect.center(), rect.width() * 0.5, color);
                        });

                        resp
                    })
                    .inner;

                // Description
                data.description(context).label_w(ui);

                // Stats
                ui.horizontal(|ui| {
                    for (var_name, var_value) in data.stats(context) {
                        TagWidget::new_var_value(var_name, var_value).ui(ui);
                    }
                });

                resp
            })
            .inner;

        inner_response
    }
}

/// Composer for expandable tag/card views
#[derive(Clone)]
pub struct TagCardComposer {
    default_expanded: bool,
}

impl Default for TagCardComposer {
    fn default() -> Self {
        Self {
            default_expanded: false,
        }
    }
}

impl TagCardComposer {
    pub fn new(expanded: bool) -> Self {
        Self {
            default_expanded: expanded,
        }
    }
}

impl<T: FTag + FTitle + FDescription + FStats + Node> Composer<T> for TagCardComposer {
    fn compose(&self, data: &T, context: &Context, ui: &mut Ui) -> Response {
        let expanded_id = data.egui_id().with(ui.id()).with("expanded");
        let expanded = ui
            .ctx()
            .data(|r| r.get_temp::<bool>(expanded_id))
            .unwrap_or(self.default_expanded);

        let response_result =
            context.with_layer_ref_r(ContextLayer::Owner(data.entity()), |context| {
                if expanded {
                    Ok(CardComposer.compose(data, context, ui))
                } else {
                    Ok(TagComposer.compose(data, context, ui))
                }
            });

        let response = match response_result {
            Ok(r) => r,
            Err(_) => ui.label("Error loading data"),
        };

        if response.clicked() {
            ui.ctx().data_mut(|w| w.insert_temp(expanded_id, !expanded));
        }

        response
    }
}

/// Composer for rating display
pub struct RatingComposer;

impl<T: FRating> Composer<T> for RatingComposer {
    fn compose(&self, data: &T, context: &Context, ui: &mut Ui) -> Response {
        let rating = data.rating(context).unwrap_or(0);
        let text = rating.cstr_expanded();

        let response = text.as_button().ui(ui);
        response.clone().bar_menu(|ui| {
            ui.vertical(|ui| {
                "rating".cstr().label(ui);
                ui.horizontal(|ui| {
                    if "[red [b -]]".cstr().button(ui).clicked() {
                        // Handle downvote
                    }
                    if "[green [b +]]".cstr().button(ui).clicked() {
                        // Handle upvote
                    }
                });
            });
        });
        response
    }
}

/// Composer wrapper that adds a frame
pub struct FramedComposer<C> {
    inner: C,
    color: Option<Color32>,
}

impl<C> FramedComposer<C> {
    pub fn new(inner: C) -> Self {
        Self { inner, color: None }
    }

    pub fn with_color(mut self, color: Color32) -> Self {
        self.color = Some(color);
        self
    }
}

impl<T, C: Composer<T>> Composer<T> for FramedComposer<C> {
    fn compose(&self, data: &T, context: &Context, ui: &mut Ui) -> Response {
        let color = self.color.unwrap_or_else(|| context.color(ui));

        Frame::new()
            .inner_margin(2)
            .corner_radius(ROUNDING)
            .stroke(color.stroke())
            .show(ui, |ui| self.inner.compose(data, context, ui))
            .inner
    }
}

/// Composer that shows info on hover
pub struct WithTooltipComposer<C> {
    inner: C,
}

impl<C> WithTooltipComposer<C> {
    pub fn new(inner: C) -> Self {
        Self { inner }
    }
}

impl<T: FInfo, C: Composer<T>> Composer<T> for WithTooltipComposer<C> {
    fn compose(&self, data: &T, context: &Context, ui: &mut Ui) -> Response {
        self.inner
            .compose(data, context, ui)
            .on_hover_text(data.info(context).get_text())
    }
}

/// Composer that adds validation feedback
pub struct ValidatedComposer<C> {
    inner: C,
}

impl<C> ValidatedComposer<C> {
    pub fn new(inner: C) -> Self {
        Self { inner }
    }
}

impl<T: FValidate, C: Composer<T>> Composer<T> for ValidatedComposer<C> {
    fn compose(&self, data: &T, context: &Context, ui: &mut Ui) -> Response {
        let response = self.inner.compose(data, context, ui);

        if let Err(errors) = data.validate(context) {
            ui.vertical(|ui| {
                for error in errors {
                    error.cstr_c(RED).label(ui);
                }
            });
        }

        response
    }
}

/// Composer for showing children
/// Note: Currently disabled due to associated type constraints with NodeExt
// pub struct ChildrenComposer<C> {
//     child_composer: C,
// }

// impl<C> ChildrenComposer<C> {
//     pub fn new(child_composer: C) -> Self {
//         Self { child_composer }
//     }
// }

// impl<T: FChildren, C: Clone> Composer<T> for ChildrenComposer<C>
// where
//     C: for<'a> Composer<Box<dyn NodeExt + 'a>>,
// {
//     fn compose(&self, data: &T, context: &Context, ui: &mut Ui) -> Response {
//         ui.vertical(|ui| {
//             if let Ok(children) = data.children(context) {
//                 for child in children {
//                     self.child_composer.compose(&child, context, ui);
//                 }
//             }
//         })
//         .response
//     }
// }

/// Composer that adds search/filter functionality
pub struct SearchableComposer<C> {
    inner: C,
    search_query: String,
}

impl<C> SearchableComposer<C> {
    pub fn new(inner: C) -> Self {
        Self {
            inner,
            search_query: String::new(),
        }
    }
}

impl<T: FSearchable, C: Composer<T>> Composer<T> for SearchableComposer<C> {
    fn compose(&self, data: &T, context: &Context, ui: &mut Ui) -> Response {
        if self.search_query.is_empty() || data.matches_search(&self.search_query, context) {
            self.inner.compose(data, context, ui)
        } else {
            ui.label("")
        }
    }
}

/// Composer chain that applies multiple composers in sequence
pub struct ComposerChain<T> {
    composers: Vec<Box<dyn Composer<T>>>,
}

impl<T> ComposerChain<T> {
    pub fn new() -> Self {
        Self {
            composers: Vec::new(),
        }
    }

    pub fn add<C: Composer<T> + 'static>(mut self, composer: C) -> Self {
        self.composers.push(Box::new(composer));
        self
    }
}

impl<T> Composer<T> for ComposerChain<T> {
    fn compose(&self, data: &T, context: &Context, ui: &mut Ui) -> Response {
        let mut response = ui.label("");
        for composer in &self.composers {
            response = response.union(composer.compose(data, context, ui));
        }
        response
    }
}

/// Composer that renders in a collapsing header
pub struct CollapsingComposer<C> {
    inner: C,
    title: String,
    default_open: bool,
}

impl<C> CollapsingComposer<C> {
    pub fn new(inner: C, title: String) -> Self {
        Self {
            inner,
            title,
            default_open: false,
        }
    }

    pub fn default_open(mut self, open: bool) -> Self {
        self.default_open = open;
        self
    }
}

impl<T, C: Composer<T>> Composer<T> for CollapsingComposer<C> {
    fn compose(&self, data: &T, context: &Context, ui: &mut Ui) -> Response {
        ui.collapsing(self.title.clone(), |ui| {
            self.inner.compose(data, context, ui)
        })
        .header_response
    }
}
