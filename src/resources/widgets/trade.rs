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

        let trade = cn()
            .db
            .trade()
            .id()
            .find(&id)
            .with_context(|| format!("Tried to open absent trade #{id}"))
            .unwrap();
        popup("Trade", ctx, |ui| {
            let items = if trade.a_player == player_id() {
                trade.a_offer
            } else {
                trade.a_offer
            };
            items.show(1.0, ui, world);
            ui.vertical_centered_justified(|ui| {
                if Button::new("Accept").ui(ui).clicked() {
                    cn().reducers.accept_trade(id).unwrap();
                    Self::close(ctx);
                }
            });
        });
    }
}
