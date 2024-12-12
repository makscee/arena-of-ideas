use spacetimedb_sdk::{Event, Table};

use super::*;

pub trait EntityExt {
    fn get_parent(self, world: &World) -> Option<Entity>;
    fn get_parent_query(self, query: &Query<&Parent>) -> Option<Entity>;
}

impl EntityExt for Entity {
    fn get_parent(self, world: &World) -> Option<Entity> {
        world.get::<Parent>(self).map(|p| p.get())
    }
    fn get_parent_query(self, query: &Query<&Parent>) -> Option<Entity> {
        query.get(self).ok().map(|p| p.get())
    }
}
pub trait TableSingletonExt: Table {
    fn current(&self) -> Self::Row {
        *Self::get_current(self).unwrap()
    }
    fn get_current(&self) -> Option<Box<Self::Row>> {
        Self::iter(self).exactly_one().ok().map(|d| Box::new(d))
    }
}

impl TableSingletonExt for GlobalDataTableHandle<'static> {}
impl TableSingletonExt for GlobalSettingsTableHandle<'static> {}
impl TableSingletonExt for WalletTableHandle<'static> {}

pub trait StdbStatusExt {
    fn on_success(&self, f: impl FnOnce(&mut World) + Send + Sync + 'static);
    fn notify_error(&self);
}

impl<R> StdbStatusExt for Event<R> {
    fn on_success(&self, f: impl FnOnce(&mut World) + Send + Sync + 'static) {
        match self {
            Event::Reducer(r) => match &r.status {
                spacetimedb_sdk::Status::Committed => OperationsPlugin::add(f),
                spacetimedb_sdk::Status::Failed(e) => e.notify_error_op(),
                _ => panic!(),
            },
            Event::SubscribeApplied | Event::UnsubscribeApplied => OperationsPlugin::add(f),
            Event::SubscribeError(e) => e.to_string().notify_error_op(),
            Event::UnknownTransaction => "Unknown transaction".notify_error_op(),
            _ => panic!(),
        }
    }
    fn notify_error(&self) {
        match self {
            Event::Reducer(r) => match &r.status {
                spacetimedb_sdk::Status::Committed => {}
                spacetimedb_sdk::Status::Failed(e) => e.notify_error_op(),
                _ => panic!(),
            },
            Event::SubscribeError(e) => e.to_string().notify_error_op(),
            Event::UnknownTransaction => "Unknown transaction".notify_error_op(),
            _ => panic!(),
        }
    }
}

pub trait GIDExt {
    fn get_player(self) -> TPlayer;
}

impl GIDExt for u64 {
    fn get_player(self) -> TPlayer {
        if self == 0 {
            return TPlayer::default();
        }
        cn().db.player().id().find(&self).unwrap_or_default()
    }
}

// impl TTeam {
//     pub fn hover_label(&self, ui: &mut Ui, world: &mut World) {
//         let resp = self
//             .cstr()
//             .as_label(ui)
//             .sense(Sense::click())
//             .selectable(false)
//             .ui(ui);
//         if resp.hovered() {
//             cursor_window(ui.ctx(), |ui| {
//                 Frame {
//                     inner_margin: Margin::same(8.0),
//                     rounding: Rounding::same(13.0),
//                     fill: BG_TRANSPARENT,
//                     ..default()
//                 }
//                 .show(ui, |ui| {
//                     self.show(1.0, ui, world);
//                 });
//             });
//             if resp.clicked() {
//                 let packed = PackedTeam::from_id(self.id);
//                 let s = ron::to_string(&packed).unwrap();
//                 copy_to_clipboard(&s, world);
//                 Notification::new(
//                     format!("Team#{} copied to clipboard", self.id).cstr_c(VISIBLE_LIGHT),
//                 )
//                 .push(world);
//             }
//         }
//     }
// }

impl Default for TPlayer {
    fn default() -> Self {
        Self {
            id: 0,
            name: "...".into(),
            identities: default(),
            pass_hash: default(),
            online: default(),
            last_login: default(),
        }
    }
}
impl TPlayer {
    pub fn get_supporter_level(&self) -> u8 {
        const SUPPORTER_TAG_NAMES: [&str; 4] = [
            "SupporterCommon",
            "SupporterRare",
            "SupporterEpic",
            "SupporterLegendary",
        ];
        for tag in cn().db.player_tag().iter().filter_map(|t| {
            if t.owner == self.id {
                Some(t.tag)
            } else {
                None
            }
        }) {
            if let Some(i) = SUPPORTER_TAG_NAMES.iter().position(|n| n == &tag) {
                return i as u8 + 1;
            }
        }
        0
    }
}
impl EventContext {
    pub fn check_identity(&self) -> bool {
        match &self.event {
            Event::Reducer(r) => r.caller_identity == player_identity(),
            _ => true,
        }
    }
}
