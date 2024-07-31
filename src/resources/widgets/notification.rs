use std::collections::VecDeque;

use super::*;

#[derive(Clone, Debug)]
pub struct Notification {
    text: String,
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
    shown: VecDeque<(f32, Notification)>,
}

impl Notification {
    pub fn new(text: String) -> Self {
        Self {
            text,
            r#type: default(),
        }
    }
    pub fn error(mut self) -> Self {
        self.r#type = NotificationType::Error;
        self
    }
    pub fn push_op(self) {
        OperationsPlugin::add(|w| self.push(w));
    }
    pub fn push(self, world: &mut World) {
        let t = elapsed_time(world);
        world
            .resource_mut::<WidgetData>()
            .notifications
            .shown
            .push_back((t, self));
    }
    pub fn show_all(wd: &WidgetData, ctx: &egui::Context, world: &mut World) {
        let elapsed = elapsed_time(world);
        let notifications = wd
            .notifications
            .shown
            .iter()
            .filter(|(t, _)| elapsed - *t < 5.0)
            .map(|(_, n)| n.clone())
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
                        match n.r#type {
                            NotificationType::Alert => n.text.cstr_c(VISIBLE_LIGHT),
                            NotificationType::Error => n.text.cstr_c(RED),
                        }
                        .label(ui);
                    });
                }
            });
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
        Notification::new(s).push_op()
    }
    fn notify_error(&self) {
        let s = self.to_string();
        error!("{}{s}", "Notify error: ".dimmed());
        Notification::new(s).error().push_op()
    }
}

impl NotificationPusher for String {}
impl NotificationPusher for str {}
