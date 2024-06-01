use crate::event::Event;
use std::collections::VecDeque;

use super::*;

pub struct ActionPlugin;

impl Plugin for ActionPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<EventQueue>()
            .init_resource::<ActionQueue>()
            .init_resource::<ActionsData>()
            .add_systems(Update, Self::update);
    }
}

#[derive(Resource, Default)]
struct EventQueue(VecDeque<(Event, Context)>);
#[derive(Resource, Default)]
struct ActionQueue(VecDeque<Action>);

#[derive(Resource, Default)]
struct ActionsData {
    events: Vec<(f32, Event)>,
    turns: Vec<(f32, usize)>,
    chain: usize,
}

struct Action {
    effect: Effect,
    context: Context,
    delay: f32,
}

impl ActionPlugin {
    fn update(world: &mut World) {
        Self::spin(world).expect("Spin failed");
    }

    pub fn spin(world: &mut World) -> Result<bool> {
        let mut processed = false;
        let mut limit = 100000;
        loop {
            if limit == 0 {
                return Err(anyhow!("Limit exceeded"));
            }
            limit -= 1;
            if let Some(Action {
                effect,
                mut context,
                delay,
            }) = Self::pop_action(world)
            {
                match effect.invoke(&mut context, world) {
                    Ok(_) => {
                        world.resource_mut::<ActionsData>().chain += 1;
                        processed = true;
                        for status in world
                            .query_filtered::<Entity, (With<Status>, With<VarStateDelta>, With<Parent>)>()
                            .iter(world)
                            .collect_vec()
                        {
                            Status::refresh_status_mapping(status, world);
                        }
                        GameTimer::get().advance_insert(delay);
                    }
                    Err(e) => error!("Effect process error: {e}"),
                }
                continue;
            }
            let mut actions_added = false;
            while let Some((event, context)) = Self::pop_event(world) {
                if event.process(context, world) {
                    GameTimer::get().advance_insert(0.2);
                    actions_added = true;
                    break;
                }
            }
            if !actions_added {
                break;
            }
        }
        if processed {
            Self::clear_dead(world);
        }
        Ok(processed)
    }

    pub fn clear_dead(world: &mut World) -> bool {
        let dead = UnitPlugin::run_death_check(world);
        let died = !dead.is_empty();
        for unit in dead {
            UnitPlugin::turn_into_corpse(unit, world);
        }
        if died {
            GameTimer::get().advance_insert(0.5);
            UnitPlugin::fill_slot_gaps(Faction::Left, world);
            UnitPlugin::fill_slot_gaps(Faction::Right, world);
            UnitPlugin::translate_to_slots(world);
            GameTimer::get().insert_to_end();
        } else {
            GameTimer::get().advance_insert(0.3);
        }
        died
    }

    pub fn get_event(world: &World) -> Option<(Event, f32)> {
        let t = GameTimer::get().play_head();
        world.get_resource::<ActionsData>().and_then(|d| {
            d.events.iter().rev().find_map(|(ts, e)| match t >= *ts {
                true => Some((e.clone(), t - *ts)),
                false => None,
            })
        })
    }
    pub fn register_event(event: Event, world: &mut World) {
        world
            .resource_mut::<ActionsData>()
            .events
            .push((GameTimer::get().insert_head(), event));
    }

    pub fn get_chain_len(world: &World) -> usize {
        world.resource::<ActionsData>().chain
    }

    pub fn get_turn(t: f32, world: &World) -> (usize, f32) {
        world
            .get_resource::<ActionsData>()
            .and_then(|d| {
                d.turns.iter().rev().find_map(|(ts, e)| match t >= *ts {
                    true => Some((*e, t - *ts)),
                    false => None,
                })
            })
            .unwrap_or_default()
    }
    pub fn register_next_turn(world: &mut World) {
        let mut data = world.resource_mut::<ActionsData>();
        let next = data.turns.last().map(|(_, r)| *r).unwrap_or_default() + 1;
        data.turns.push((GameTimer::get().insert_head(), next));
        data.chain = 0;
    }

    pub fn event_push_back(event: Event, context: Context, world: &mut World) {
        world
            .resource_mut::<EventQueue>()
            .0
            .push_back((event, context));
    }
    pub fn event_push_front(event: Event, context: Context, world: &mut World) {
        world
            .resource_mut::<EventQueue>()
            .0
            .push_front((event, context));
    }
    pub fn action_push_back(effect: Effect, context: Context, world: &mut World) {
        world.resource_mut::<ActionQueue>().0.push_back(Action {
            effect,
            context,
            delay: 0.0,
        });
    }
    pub fn action_push_front(effect: Effect, context: Context, world: &mut World) {
        world.resource_mut::<ActionQueue>().0.push_front(Action {
            effect,
            context,
            delay: 0.0,
        });
    }
    pub fn action_push_back_with_delay(
        effect: Effect,
        context: Context,
        delay: f32,
        world: &mut World,
    ) {
        world.resource_mut::<ActionQueue>().0.push_back(Action {
            effect,
            context,
            delay,
        });
    }
    pub fn action_push_front_with_delay(
        effect: Effect,
        context: Context,
        delay: f32,
        world: &mut World,
    ) {
        world.resource_mut::<ActionQueue>().0.push_front(Action {
            effect,
            context,
            delay,
        });
    }

    pub fn new_battle(world: &mut World) {
        *world.resource_mut::<ActionsData>() = default();
    }

    fn pop_action(world: &mut World) -> Option<Action> {
        world.resource_mut::<ActionQueue>().0.pop_front()
    }
    fn pop_event(world: &mut World) -> Option<(Event, Context)> {
        world.resource_mut::<EventQueue>().0.pop_front()
    }
}
