use super::*;

#[spacetimedb(table(public))]
#[derive(Clone, Default)]
struct TQuest {
    #[primarykey]
    id: u64,
    owner: u64,
    mode: GameMode,
    variant: QuestVariant,
    counter: u32,
    goal: u32,
    reward: i64,
    complete: bool,
}

#[derive(SpacetimeType, PartialEq, Eq, Clone, Default)]
enum QuestVariant {
    #[default]
    Win,
    Streak,
    Champion,
    FuseMany,
    FuseOne,
}

pub enum QuestEvent {
    Win,
    Streak(u32),
    Champion,
    Fuse(u32),
}

impl TQuest {
    fn register_update(&mut self, value: u32) {
        match self.variant {
            QuestVariant::Win | QuestVariant::FuseMany | QuestVariant::Champion => {
                self.counter += 1
            }
            QuestVariant::Streak | QuestVariant::FuseOne => {
                self.counter = value.max(self.counter);
            }
        }
        if !self.complete && self.counter >= self.goal {
            self.complete = true;
        }
    }
}

impl QuestEvent {
    pub fn register_event(self, mode: GameMode, owner: u64) {
        let quests = TQuest::filter_by_owner(&owner)
            .filter(|q| q.mode == mode)
            .collect_vec();
        match self {
            QuestEvent::Win => {
                for mut q in quests
                    .into_iter()
                    .filter(|q| q.variant == QuestVariant::Win)
                {
                    q.register_update(1);
                    TQuest::update_by_id(&q.id.clone(), q);
                }
            }
            QuestEvent::Streak(c) => {
                for mut q in quests
                    .into_iter()
                    .filter(|q| q.variant == QuestVariant::Streak)
                {
                    q.register_update(c);
                    TQuest::update_by_id(&q.id.clone(), q);
                }
            }
            QuestEvent::Champion => {
                for mut q in quests
                    .into_iter()
                    .filter(|q| q.variant == QuestVariant::Champion)
                {
                    q.register_update(1);
                    TQuest::update_by_id(&q.id.clone(), q);
                }
            }
            QuestEvent::Fuse(c) => {
                for mut q in quests.into_iter().filter(|q| {
                    q.variant == QuestVariant::FuseMany || q.variant == QuestVariant::FuseOne
                }) {
                    q.register_update(c);
                    TQuest::update_by_id(&q.id.clone(), q);
                }
            }
        }
    }
}

pub fn quests_daily_refresh() {
    TQuest::delete_by_owner(&0);
    let mut options: Vec<TQuest> = [
        TQuest {
            mode: GameMode::ArenaNormal,
            variant: QuestVariant::Win,
            goal: 5,
            reward: 25,
            ..default()
        },
        TQuest {
            mode: GameMode::ArenaRanked,
            variant: QuestVariant::Win,
            goal: 5,
            reward: 50,
            ..default()
        },
        TQuest {
            mode: GameMode::ArenaNormal,
            variant: QuestVariant::Champion,
            goal: 1,
            reward: 100,
            ..default()
        },
        TQuest {
            mode: GameMode::ArenaRanked,
            variant: QuestVariant::Champion,
            goal: 1,
            reward: 200,
            ..default()
        },
        TQuest {
            mode: GameMode::ArenaNormal,
            variant: QuestVariant::FuseOne,
            goal: 5,
            reward: 30,
            ..default()
        },
        TQuest {
            mode: GameMode::ArenaRanked,
            variant: QuestVariant::FuseOne,
            goal: 5,
            reward: 60,
            ..default()
        },
        TQuest {
            mode: GameMode::ArenaNormal,
            variant: QuestVariant::FuseMany,
            goal: 7,
            reward: 25,
            ..default()
        },
        TQuest {
            mode: GameMode::ArenaRanked,
            variant: QuestVariant::FuseMany,
            goal: 7,
            reward: 50,
            ..default()
        },
        TQuest {
            mode: GameMode::ArenaNormal,
            variant: QuestVariant::Streak,
            goal: 5,
            reward: 40,
            ..default()
        },
        TQuest {
            mode: GameMode::ArenaRanked,
            variant: QuestVariant::Streak,
            goal: 7,
            reward: 80,
            ..default()
        },
    ]
    .into();
    for _ in 0..GlobalSettings::get().quest.daily_options {
        let i = rng().gen_range(0..options.len());
        let mut q = options.remove(i);
        q.id = next_id();
        TQuest::insert(q).unwrap();
    }
}

#[spacetimedb(reducer)]
fn quest_accept(ctx: ReducerContext, id: u64) -> Result<(), String> {
    let user = ctx.user()?;
    let mut quest =
        TQuest::filter_by_id(&id).with_context_str(|| format!("Quest#{id} not found"))?;
    if quest.owner != 0 {
        return Err("Wrong quest".into());
    }
    TDailyState::get(user.id).take_quest(id)?;
    quest.id = next_id();
    quest.owner = user.id;
    TQuest::insert(quest)?;
    Ok(())
}

#[spacetimedb(reducer)]
fn quest_finish(ctx: ReducerContext, id: u64) -> Result<(), String> {
    let user = ctx.user()?;
    let quest = TQuest::filter_by_id(&id).with_context_str(|| format!("Quest#{id} not found"))?;
    if quest.owner != user.id {
        return Err(format!("Quest#{id} not owned by {}", user.id));
    }
    TWallet::change(user.id, quest.reward)?;
    TQuest::delete_by_id(&quest.id);
    Ok(())
}
