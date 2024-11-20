use super::*;

pub struct RewardsPlugin;

impl RewardsPlugin {
    pub fn have_unclaimed() -> bool {
        cn().db.reward().iter().any(|r| r.owner == player_id())
    }

    pub fn open_rewards(world: &mut World) {
        Confirmation::new("Rewards")
            .cancel(|_| {})
            .cancel_name("Close")
            .content(|ui, world| {
                Self::show_rewards(ui, world);
            })
            .push(world);
    }
    fn show_rewards(ui: &mut Ui, world: &mut World) {
        for r in cn()
            .db
            .reward()
            .iter()
            .filter(|r| r.owner == player_id())
            .sorted_by_key(|r| r.ts)
            .rev()
        {
            ui.columns(3, |ui| {
                ui[0].vertical_centered_justified(|ui| {
                    if Button::new("Claim".cstr_s(CstrStyle::Bold))
                        .ui(ui)
                        .clicked()
                    {
                        Self::open_reward(r.id, world);
                    }
                });
                ui[1].vertical_centered_justified(|ui| {
                    r.source.cstr_cs(VISIBLE_LIGHT, CstrStyle::Bold).label(ui);
                });
                ui[2].vertical_centered_justified(|ui| {
                    format_timestamp(r.ts).cstr().label(ui);
                });
            });
        }
    }
    pub fn open_reward(id: u64, world: &mut World) {
        let open_ts = gt().play_head();
        Confirmation::new("Reward")
            .accept(move |_| {
                cn().reducers.reward_claim(id).unwrap();
            })
            .accept_name("Claim")
            .cancel(|_| {})
            .cancel_name("Close")
            .content(move |ui, world| {
                if let Some(r) = cn().db.reward().id().find(&id) {
                    let t = gt().play_head() - open_ts;
                    r.bundle.show(t, ui, world);
                }
            })
            .push(world);
    }
}
