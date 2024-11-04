use super::*;

pub struct QuestPlugin;

impl QuestPlugin {
    pub fn new_available() -> bool {
        global_settings().quest.daily_limit as usize
            > TDailyState::get_current()
                .map(|ds| ds.quests_taken.len())
                .unwrap_or_default()
    }
    pub fn have_completed() -> bool {
        TQuest::filter_by_owner(player_id())
            .filter(|q| q.complete)
            .count()
            > 0
    }
    pub fn add_tiles(world: &mut World) {
        Tile::new(Side::Left, |ui, world| {
            show_daily_refresh_timer(ui);
            let quests_taken = TDailyState::get_current()
                .map(|ds| ds.quests_taken)
                .unwrap_or_default();
            let taken = quests_taken.len();
            let limit = global_settings().quest.daily_limit as usize;
            if limit > taken {
                title("New Quests", ui);
                ui.vertical_centered_justified(|ui| {
                    format!("Taken {taken}/{limit}")
                        .cstr_cs(VISIBLE_BRIGHT, CstrStyle::Heading2)
                        .label(ui);
                });
                Table::new("new quests")
                    .column_cstr("quest", |d: &TQuest, _| d.cstr())
                    .column_btn("accept", |d, _, _| {
                        quest_accept(d.id);
                    })
                    .ui(
                        &TQuest::filter_by_owner(0)
                            .filter(|q| !quests_taken.contains(&q.id))
                            .into_iter()
                            .collect_vec(),
                        ui,
                        world,
                    );
            }
            let complete_quests = TQuest::filter_by_owner(player_id())
                .filter(|q| q.complete)
                .collect_vec();
            if !complete_quests.is_empty() {
                Table::new("Complete Quests")
                    .title()
                    .column_cstr("quest", |d: &TQuest, _| d.cstr())
                    .column_btn("complete", |d, _, _| quest_finish(d.id))
                    .ui(&complete_quests, ui, world);
            }
            let current_quests = TQuest::filter_by_owner(player_id())
                .filter(|q| !q.complete)
                .collect_vec();
            if !current_quests.is_empty() {
                title("Current Quests", ui);
                for q in current_quests {
                    q.cstr().label(ui);
                }
            }
        })
        .pinned()
        .push(world);
    }
}

impl ToCstr for TQuest {
    fn cstr(&self) -> Cstr {
        let goal = self
            .goal
            .to_string()
            .cstr_cs(VISIBLE_BRIGHT, CstrStyle::Bold);
        let progress =
            format!("{}/{}", self.counter, self.goal).cstr_cs(VISIBLE_BRIGHT, CstrStyle::Bold);
        let mode = self.mode.cstr().style(CstrStyle::Normal).take();
        let reward = " reward: "
            .cstr()
            .push(format!("{}{}", self.reward, CREDITS_SYM).cstr_cs(YELLOW, CstrStyle::Bold))
            .take();
        match &self.variant {
            QuestVariant::Win => "Win ".cstr().push(goal).push(" battles in ".cstr()).take(),
            QuestVariant::Streak => "Get a streak of  "
                .cstr()
                .push(goal)
                .push(" wins in ".cstr())
                .take(),
            QuestVariant::Champion => "Become champion in  ".cstr(),
            QuestVariant::FuseMany => "Fuse a hero "
                .cstr()
                .push(goal)
                .push(" times in ".cstr())
                .take(),
            QuestVariant::FuseOne => "Fuse "
                .cstr()
                .push(goal)
                .push(" heroes into one in ".cstr())
                .take(),
        }
        .push(mode)
        .push(" game mode. ".cstr())
        .push(progress)
        .push(reward)
        .take()
    }
}
