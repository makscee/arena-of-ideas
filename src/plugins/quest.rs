use super::*;

pub struct QuestPlugin;

impl QuestPlugin {
    pub fn new_available() -> bool {
        global_settings().quest.daily_limit as usize
            > cn().db.daily_state().current().quests_taken.len()
    }
    pub fn have_completed() -> bool {
        cn().db
            .quest()
            .iter()
            .filter(|q| q.complete && q.owner == player_id())
            .count()
            > 0
    }
    pub fn popup(world: &mut World) {
        Confirmation::new("Quests")
            .cancel(|_| {})
            .cancel_name("Close")
            .content(|ui, world| {
                show_daily_refresh_timer(ui);
                let quests_taken = cn().db.daily_state().current().quests_taken;
                let taken = quests_taken.len();
                let limit = global_settings().quest.daily_limit as usize;
                if limit > taken {
                    title("New Quests", ui);
                    ui.vertical_centered_justified(|ui| {
                        format!("Taken {taken}/{limit}")
                            .cstr_cs(VISIBLE_BRIGHT, CstrStyle::Heading2)
                            .label(ui);
                    });
                    Table::new("new quests", |_| {
                        cn().db
                            .quest()
                            .iter()
                            .filter(|q| {
                                q.owner == 0
                                    && !cn().db.daily_state().current().quests_taken.contains(&q.id)
                            })
                            .into_iter()
                            .collect_vec()
                    })
                    .column_cstr("quest", |d: &TQuest, _| d.cstr())
                    .column_btn("accept", |d, _, _| {
                        cn().reducers.quest_accept(d.id).unwrap();
                    })
                    .ui(ui, world);
                }
                if Self::have_completed() {
                    Table::new("Complete Quests", |_| {
                        cn().db
                            .quest()
                            .iter()
                            .filter(|q| q.complete && q.owner == player_id())
                            .collect_vec()
                    })
                    .title()
                    .column_cstr("quest", |d: &TQuest, _| d.cstr())
                    .column_btn("complete", |d, _, _| {
                        cn().reducers.quest_finish(d.id).unwrap();
                    })
                    .ui(ui, world);
                }
                let current_quests = cn()
                    .db
                    .quest()
                    .iter()
                    .filter(|q| !q.complete && q.owner == player_id())
                    .collect_vec();
                if !current_quests.is_empty() {
                    title("Current Quests", ui);
                    for q in current_quests {
                        q.cstr().label(ui);
                    }
                }
            })
            .push(world);
    }
}

impl ToCstr for TQuest {
    fn cstr(&self) -> Cstr {
        let goal = &self
            .goal
            .to_string()
            .cstr_cs(VISIBLE_BRIGHT, CstrStyle::Bold);
        let progress =
            format!("{}/{}", self.counter, self.goal).cstr_cs(VISIBLE_BRIGHT, CstrStyle::Bold);
        let mode = &self.mode.to_string();
        let reward = " reward: ".to_owned()
            + &format!("{}{}", self.reward, CREDITS_SYM).cstr_cs(YELLOW, CstrStyle::Bold);
        let prefix = match &self.variant {
            QuestVariant::Win => "Win ".to_owned() + goal + " battles in ",
            QuestVariant::Streak => "Get a streak of  ".to_owned() + goal + " wins in ",
            QuestVariant::Champion => "Become champion in  ".to_owned(),
            QuestVariant::FuseMany => "Fuse a hero ".to_owned() + goal + " times in ",
            QuestVariant::FuseOne => "Fuse ".to_owned() + goal + " heroes into one in ",
        };
        prefix + mode + " game mode. " + &progress + &reward
    }
}
