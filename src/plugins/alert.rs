use super::*;

use bevy::input::common_conditions::input_just_pressed;
use bevy_egui::egui::Order;

type AlertAction = Box<dyn FnOnce(&mut World) + Send + Sync>;
pub struct AlertPlugin;

lazy_static! {
    static ref ALERTS: Mutex<AlertData> = Mutex::new(default());
}

#[derive(Default)]
pub struct AlertData {
    alerts: HashMap<usize, Alert>,
    id: usize,
}

struct Alert {
    title: Option<String>,
    text: String,
    action: Option<AlertAction>,
    r#type: AlertType,
}

enum AlertType {
    Error,
    Normal,
}

impl Plugin for AlertPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, Self::ui)
            .add_systems(
                Update,
                Self::close_all.run_if(input_just_pressed(KeyCode::Escape)),
            )
            .add_systems(OnExit(GameState::MainMenu), Self::close_all);
    }
}

impl AlertType {
    fn color(&self) -> Color32 {
        match self {
            AlertType::Error => red(),
            AlertType::Normal => yellow(),
        }
    }
}

impl Alert {
    fn do_add(self) {
        let mut data = ALERTS.lock().unwrap();
        data.id += 1;
        let id = data.id;
        data.alerts.insert(id, self);
    }
}

impl AlertPlugin {
    pub fn add(title: Option<String>, text: String, action: Option<AlertAction>) {
        Alert {
            title,
            text,
            action,
            r#type: AlertType::Normal,
        }
        .do_add()
    }
    pub fn add_error(title: Option<String>, text: String, action: Option<AlertAction>) {
        error!("{title:?} {text}");
        Alert {
            title,
            text,
            action,
            r#type: AlertType::Error,
        }
        .do_add()
    }

    pub fn close_all(world: &mut World) {
        let mut data = ALERTS.lock().unwrap();
        for alert in data.alerts.values_mut() {
            if let Some(action) = alert.action.take() {
                action(world);
            }
        }
        data.alerts.clear();
    }

    fn ui(world: &mut World) {
        let ctx = &if let Some(context) = egui_context(world) {
            context
        } else {
            return;
        };
        let mut data = ALERTS.lock().unwrap();
        let mut actions: Vec<AlertAction> = default();
        let offset = 15.0;
        let mut closed: HashSet<usize> = default();

        for (c, (key, alert)) in data
            .alerts
            .iter_mut()
            .sorted_by_key(|(k, _)| *k)
            .enumerate()
        {
            let c = c as f32;
            window(alert.title.as_ref().unwrap_or(&"Alert".to_owned()))
                .id(key)
                .set_color(alert.r#type.color())
                .order(Order::Foreground)
                .default_pos(ctx.screen_rect().center() + egui::vec2(offset * c, offset * c))
                .set_min_width(400.0)
                .show(ctx, |ui| {
                    frame(ui, |ui| {
                        ui.vertical_centered(|ui| {
                            alert.text.add_color(white()).as_label(ui).wrap(true).ui(ui);
                        });
                    });
                    frame(ui, |ui| {
                        let has_action = alert.action.is_some();
                        ui.columns(1 + has_action as usize, |ui| {
                            ui[0].vertical_centered_justified(|ui| {
                                if ui.button("CLOSE").clicked() {
                                    closed.insert(*key);
                                }
                            });
                            if has_action {
                                ui[1].vertical_centered_justified(|ui| {
                                    if ui.button_primary("OK").clicked() {
                                        actions.push(alert.action.take().unwrap());
                                        closed.insert(*key);
                                    }
                                });
                            }
                        })
                    })
                });
        }
        data.alerts.retain(|k, _| !closed.contains(k));
        for action in actions {
            action(world);
        }
    }
}
