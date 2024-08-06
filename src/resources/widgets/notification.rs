use std::collections::VecDeque;

use super::*;

#[derive(Clone, Debug)]
pub struct Notification {
    text: Cstr,
    r#type: NotificationType,
}

#[derive(Default, Clone, Debug)]
enum NotificationType {
    #[default]
    Alert,
    Error,
}

#[derive(Default, Debug)]
pub struct NotificationsData {
    shown: VecDeque<(i64, Notification)>,
}

impl Notification {
    pub fn new_string(text: String) -> Self {
        Self::new(text.cstr_c(VISIBLE_LIGHT))
    }
    pub fn new(text: Cstr) -> Self {
        Self {
            text,
            r#type: default(),
        }
    }
    pub fn error(mut self) -> Self {
        self.r#type = NotificationType::Error;
        self.text = "Error: ".cstr_c(RED).push(self.text).take();
        self
    }
    pub fn push_op(self) {
        OperationsPlugin::add(|w| self.push(w));
    }
    pub fn push(self, world: &mut World) {
        let t = now_micros();
        let d = &mut world.resource_mut::<WidgetData>().notifications.shown;
        d.push_front((t, self));
        d.make_contiguous();
    }
    pub fn show_recent(wd: &WidgetData, ctx: &egui::Context, _: &mut World) {
        let now = now_micros();
        let notifications = wd
            .notifications
            .shown
            .iter()
            .filter(|(t, _)| now - *t < 5000000)
            .map(|(_, n)| n.clone())
            .rev()
            .collect_vec();
        if notifications.is_empty() {
            return;
        }
        Tile::right("Notifications")
            .transparent()
            .non_resizable()
            .open()
            .show(ctx, |ui| {
                for n in notifications {
                    FRAME.show(ui, |ui| {
                        n.text.label(ui);
                    });
                }
            });
    }
    pub fn show_all_table(wd: &WidgetData, ui: &mut Ui, world: &mut World) {
        Table::new("Notifications")
            .column_cstr("text", |(_, n): &(i64, Notification)| n.text.clone())
            .column_ts("time", |(t, _)| *t as u64)
            .ui(&wd.notifications.shown.as_slices().0, ui, world);
    }
}

const FRAME: Frame = Frame {
    inner_margin: Margin::same(13.0),
    rounding: Rounding::same(13.0),
    fill: BG_LIGHT,
    outer_margin: Margin::ZERO,
    shadow: SHADOW,
    stroke: Stroke::NONE,
};

pub trait NotificationPusher: ToString {
    fn notify(&self) {
        let s = self.to_string();
        info!("{} {}", "Notify: ".dimmed(), s);
        Notification::new_string(s).push_op()
    }
    fn notify_error(&self) {
        let s = self.to_string();
        error!("{}{s}", "Notify error: ".dimmed());
        Notification::new_string(s).error().push_op()
    }
}

impl NotificationPusher for String {}
impl NotificationPusher for str {}
