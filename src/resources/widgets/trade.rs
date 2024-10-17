use super::*;

pub struct Trade {}

fn open_trade_id() -> Id {
    static TRADE_DATA: OnceCell<Id> = OnceCell::new();
    *TRADE_DATA.get_or_init(|| Id::new("trade_data"))
}

impl Trade {
    pub fn open(id: u64, ctx: &egui::Context) {
        ctx.data_mut(|w| w.insert_temp(open_trade_id(), id));
    }
    pub fn close(ctx: &egui::Context) {
        ctx.data_mut(|w| w.remove_temp::<u64>(open_trade_id()));
    }
    pub fn show_active(ctx: &egui::Context, world: &mut World) {
        let Some(id) = ctx.data(|r| r.get_temp::<u64>(open_trade_id())) else {
            return;
        };

        let trade = TTrade::find_by_id(id)
            .with_context(|| format!("Tried to open absent trade #{id}"))
            .unwrap();
        popup("Trade", ctx, |ui| {
            let items = if trade.a_user == user_id() {
                trade.a_offer
            } else {
                trade.a_offer
            };
            let units = items
                .units
                .into_iter()
                .map(|id| id.unit_item().unit)
                .collect_vec();
            if !units.is_empty() {
                units.show_table("Units", ui, world);
            }
            let unit_shards = items
                .unit_shards
                .into_iter()
                .map(|id| id.unit_shard_item())
                .collect_vec();
            if !unit_shards.is_empty() {
                unit_shards.show_table("Unit Shards", ui, world);
            }
            let lootboxes = items
                .lootboxes
                .into_iter()
                .map(|id| id.lootbox_item())
                .collect_vec();
            if !lootboxes.is_empty() {
                lootboxes.show_table("Lootboxes", ui, world);
            }
            ui.vertical_centered_justified(|ui| {
                if Button::click("Accept").ui(ui).clicked() {
                    accept_trade(id);
                    once_on_accept_trade(|_, _, status, id| match status {
                        StdbStatus::Committed => {}
                        StdbStatus::Failed(e) => {
                            format!("Failed to accept trade #{id}: {e}").notify_error_op()
                        }
                        _ => panic!(),
                    });
                    Self::close(ctx);
                }
            });
        });
    }
}
