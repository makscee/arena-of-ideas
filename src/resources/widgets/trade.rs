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

        let trade = TTrade::filter_by_id(id)
            .with_context(|| format!("Tried to open absent trade #{id}"))
            .unwrap();
        popup("Trade", ctx, |ui| {
            let items = if trade.a_user == user_id() {
                trade.b_offers_items
            } else {
                trade.a_offers_items
            };

            Table::new("Items")
                .column_cstr("name", |d: &ItemStack, _| d.item.name_cstr())
                .column_cstr("type", |d, w| d.item.type_cstr(w))
                .column_int("count", |d| d.count as i32)
                .ui(&items, ui, world);
            ui.vertical_centered_justified(|ui| {
                if Button::click("Accept".into()).ui(ui).clicked() {
                    accept_trade(id);
                    once_on_accept_trade(|_, _, status, id| match status {
                        StdbStatus::Committed => {}
                        StdbStatus::Failed(e) => {
                            format!("Failed to accept trade #{id}: {e}").notify_error()
                        }
                        _ => panic!(),
                    });
                    Self::close(ctx);
                }
            });
        });
    }
}
