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

#[derive(Resource, Default, Debug)]
pub struct NotificationsResource {
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
        let d = &mut world.resource_mut::<NotificationsResource>().shown;
        d.push_front((t, self));
        d.make_contiguous();
    }
    pub fn show_recent(ctx: &egui::Context, world: &mut World) {
        Area::new(Id::new("recent_notifications"))
            .anchor(Align2::RIGHT_TOP, [0.0, 20.0])
            .order(Order::Foreground)
            .show(ctx, |ui| {
                let now = now_micros();
                let notifications = world
                    .resource::<NotificationsResource>()
                    .shown
                    .iter()
                    .filter(|(t, _)| now - *t < 5000000)
                    .map(|(_, n)| n.clone())
                    .rev()
                    .collect_vec();
                if notifications.is_empty() {
                    return;
                }
                ui.with_layout(Layout::top_down(Align::Max), |ui| {
                    ui.set_max_width(300.0);
                    for n in notifications {
                        FRAME.show(ui, |ui| {
                            n.text
                                .as_label(ui)
                                .wrap_mode(egui::TextWrapMode::Wrap)
                                .ui(ui);
                        });
                    }
                });
            });
    }
    pub fn show_all_table(ui: &mut Ui, world: &mut World) {
        world.resource_scope(|world, nr: Mut<NotificationsResource>| {
            Table::new("Notifications")
                .column_cstr("text", |(_, n): &(i64, Notification), _| n.text.clone())
                .column_ts("time", |(t, _)| *t as u64)
                .ui(&nr.shown.as_slices().0, ui, world);
        })
    }
}

const FRAME: Frame = Frame {
    inner_margin: Margin::same(13.0),
    outer_margin: Margin::symmetric(13.0, 0.0),
    rounding: Rounding::same(13.0),
    fill: BG_DARK,
    shadow: SHADOW,
    stroke: Stroke {
        width: 1.0,
        color: VISIBLE_BRIGHT,
    },
};

pub trait NotificationPusher: ToString {
    fn notify(&self, world: &mut World) {
        let s = self.to_string();
        info!("{} {}", "Notify: ".dimmed(), s);
        Notification::new_string(s).push(world)
    }
    fn notify_op(&self) {
        let s = self.to_string();
        OperationsPlugin::add(move |world| s.notify(world))
    }
    fn notify_error(&self, world: &mut World) {
        let s = self.to_string();
        error!("{}{s}", "Notify error: ".dimmed());
        Notification::new_string(s).error().push(world)
    }
    fn notify_error_op(&self) {
        let s = self.to_string();
        OperationsPlugin::add(move |world| s.notify_error(world))
    }
}

impl NotificationPusher for String {}
impl NotificationPusher for str {}

pub trait NotifyStatus {
    fn notify(&self, world: &mut World);
    fn notify_op(&self);
}
