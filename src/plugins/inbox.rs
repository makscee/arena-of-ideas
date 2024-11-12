use super::*;

pub struct InboxPlugin {}

impl InboxPlugin {
    pub fn add_tiles(world: &mut World) {
        Tile::new(Side::Left, |ui, world| {
            let rewards = cn()
                .db
                .reward()
                .iter()
                .filter(|r| r.owner == player_id())
                .collect_vec();
            Table::new("Rewards")
                .title()
                .column_cstr("source", |d: &TReward, _| d.source.cstr_c(VISIBLE_LIGHT))
                .column_btn("open", |d, _, world| {
                    Self::open_reward(d.id, world);
                })
                .ui(&rewards, ui, world);
        })
        .transparent()
        .pinned()
        .push(world);

        Tile::new(Side::Right, |ui, world| {
            Notification::show_all_table(ui, world)
        })
        .transparent()
        .pinned()
        .push(world)
    }

    fn open_reward(id: u64, world: &mut World) {
        Confirmation::new("Reward".cstr_cs(VISIBLE_BRIGHT, CstrStyle::Heading))
            .content(move |ui, world| {
                if let Some(reward) = cn().db.reward().id().find(&id) {
                    reward.bundle.show(ui, world);
                }
            })
            .accept(move |_| {
                cn().reducers.reward_claim(id).unwrap();
            })
            .accept_name("Claim")
            .cancel(|_| {})
            .cancel_name("Close")
            .push(world);
    }
}
