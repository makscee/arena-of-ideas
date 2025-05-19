use super::*;

pub struct ViewContext {
    part: ViewPart,
    disabled_parts: u64,
}

pub struct ViewState {}

#[derive(Default)]
pub struct ViewResponse {
    title_clicked: bool,
}

#[repr(u64)]
#[derive(Default, Clone, Copy)]
pub enum ViewPart {
    #[default]
    Title,
    Data,
    Value,
    Rating,
    ContextMenu,
}

impl ViewContext {
    pub fn part(mut self, part: ViewPart) -> Self {
        self.part = part;
        self
    }
}

impl ViewResponse {}

pub trait ViewParts {
    fn title(&self, vctx: ViewContext, context: &mut Context) -> Cstr;
    fn view_part(&self, vctx: ViewContext, context: &mut Context, ui: &mut Ui) -> Option<Response> {
        if vctx.part as u64 & vctx.disabled_parts > 0 {
            return None;
        }
        match vctx.part {
            ViewPart::Title => Some(self.title(vctx, context).button(ui)),
            _ => None,
        }
    }
}

pub trait View: ViewParts {
    fn view(&self, vctx: ViewContext, context: &mut Context, ui: &mut Ui) -> ViewResponse {
        let mut vr = ViewResponse::default();
        ui.horizontal(|ui| {
            if let Some(r) = self.view_part(vctx.part(ViewPart::Title), context, ui) {
                vr.title_clicked = r.clicked();
            }
        });

        vr
    }
}
