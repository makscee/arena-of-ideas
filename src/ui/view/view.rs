use super::*;

#[derive(Copy, Clone)]
pub struct ViewContext {
    interactible: bool,
}

pub trait View: Sized + ToCstr {
    fn title(&self, context: &Context, view_ctx: ViewContext) -> Cstr {
        self.cstr()
    }
    fn show_title(&self, context: &Context, view_ctx: ViewContext, ui: &mut Ui) -> Response {
        if view_ctx.interactible {
            self.title(context, view_ctx).button(ui)
        } else {
            self.title(context, view_ctx).label(ui)
        }
    }
    fn data(&self, context: &Context, view_ctx: ViewContext) -> Option<Cstr> {
        None
    }
    fn show_data_line(
        &self,
        context: &Context,
        view_ctx: ViewContext,
        ui: &mut Ui,
    ) -> Option<Response> {
        Some(self.data(context, view_ctx)?.label(ui))
    }
    fn show_data_rect(
        &self,
        context: &Context,
        view_ctx: ViewContext,
        ui: &mut Ui,
        rect: Rect,
    ) -> Option<Response> {
        None
    }
    fn type_name(&self, context: &Context, view_ctx: ViewContext) -> Cstr {
        type_name_of_val_short(self).cstr_s(CstrStyle::Small)
    }
}
