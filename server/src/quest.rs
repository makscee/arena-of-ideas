use spacetimedb::Table;

use super::*;

#[spacetimedb::table(public, name = quest)]
#[derive(Clone, Default)]
pub struct TQuest {
    #[primary_key]
    pub id: u64,
    #[index(btree)]
    pub owner: u64,
    pub mode: GameMode,
    pub variant: QuestVariant,
    pub counter: u32,
    pub goal: u32,
    pub reward: i64,
    pub complete: bool,
}

#[derive(SpacetimeType, PartialEq, Eq, Clone, Default)]
pub enum QuestVariant {
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
    pub fn register_event(self, ctx: &ReducerContext, mode: GameMode, owner: u64) {
        let quests = ctx
            .db
            .quest()
            .owner()
            .filter(owner)
            .filter(|q| q.mode == mode)
            .collect_vec();
        match self {
            QuestEvent::Win => {
                for mut q in quests
                    .into_iter()
                    .filter(|q| q.variant == QuestVariant::Win)
                {
                    q.register_update(1);
                    ctx.db.quest().id().update(q);
                }
            }
            QuestEvent::Streak(c) => {
                for mut q in quests
                    .into_iter()
                    .filter(|q| q.variant == QuestVariant::Streak)
                {
                    q.register_update(c);
                    ctx.db.quest().id().update(q);
                }
            }
            QuestEvent::Champion => {
                for mut q in quests
                    .into_iter()
                    .filter(|q| q.variant == QuestVariant::Champion)
                {
                    q.register_update(1);
                    ctx.db.quest().id().update(q);
                }
            }
            QuestEvent::Fuse(c) => {
                for mut q in quests.into_iter().filter(|q| {
                    q.variant == QuestVariant::FuseMany || q.variant == QuestVariant::FuseOne
                }) {
                    q.register_update(c);
                    ctx.db.quest().id().update(q);
                }
            }
        }
    }
}

pub fn quests_daily_refresh(ctx: &ReducerContext) {
    ctx.db.quest().owner().delete(0_u64);
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
    for _ in 0..GlobalSettings::get(ctx).quest.daily_options {
        let i = ctx.rng().gen_range(0..options.len());
        let mut q = options.remove(i);
        q.id = next_id(ctx);
        ctx.db.quest().insert(q);
    }
}

#[spacetimedb::reducer]
fn quest_accept(ctx: &ReducerContext, id: u64) -> Result<(), String> {
    let player = ctx.player()?;
    let mut quest = ctx
        .db
        .quest()
        .id()
        .find(id)
        .with_context_str(|| format!("Quest#{id} not found"))?;
    if quest.owner != 0 {
        return Err("Wrong quest".into());
    }

    TDailyState::get(ctx, player.id).take_quest(ctx, id)?;
    quest.id = next_id(ctx);
    quest.owner = player.id;
    ctx.db.quest().insert(quest);
    Ok(())
}

#[spacetimedb::reducer]
fn quest_finish(ctx: &ReducerContext, id: u64) -> Result<(), String> {
    let player = ctx.player()?;
    let quest = ctx
        .db
        .quest()
        .id()
        .find(id)
        .with_context_str(|| format!("Quest#{id} not found"))?;
    if quest.owner != player.id {
        return Err(format!("Quest#{id} not owned by {}", player.id));
    }
    TWallet::change(ctx, player.id, quest.reward)?;
    ctx.db.quest().id().delete(quest.id);
    Ok(())
}
