use std::collections::VecDeque;

use super::*;

pub struct ActionPlugin;

#[derive(Resource, Default)]
struct ActionQueue(VecDeque<Action>);
#[derive(Resource, Default)]
struct EventQueue(VecDeque<(Event, Context)>);

struct Action {
    effect: Effect,
    context: Context,
    delay: f32,
}
#[derive(Resource, Default)]
struct ActionsData {
    events: Vec<(f32, Event)>,
    turns: Vec<(f32, usize)>,
    sounds: Vec<(f32, SoundEffect)>,
    chain: usize,
}

impl Plugin for ActionPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<EventQueue>()
            .init_resource::<ActionQueue>()
            .init_resource::<ActionsData>()
            .add_systems(Update, Self::update);
    }
}

impl ActionPlugin {
    fn update(world: &mut World) {
        Self::spin(world).expect("Spin failed");
        let ph = gt().play_head();
        Self::queue_current_sound_effect(ph - gt().last_delta(), ph, world)
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
                context.t_to_insert();
                match effect.invoke(&mut context, world) {
                    Ok(_) => {
                        processed = true;
                        world.resource_mut::<ActionsData>().chain += 1;
                        gt().advance_insert(delay);
                        for unit in UnitPlugin::collect_alive(world) {
                            Status::refresh_mappings(unit, world);
                        }
                    }
                    Err(e) => error!("Effect process error: {e}"),
                }
                continue;
            }
            let mut actions_added = false;
            while let Some((event, context)) = Self::pop_event(world) {
                if event.process(context, world) {
                    gt().advance_insert(0.2);
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
            gt().advance_insert(0.5);
            UnitPlugin::fill_gaps_and_translate(world);
            gt().insert_to_end();
        } else {
            gt().advance_insert(0.3);
        }
        died
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
    pub fn event_push_back(event: Event, context: Context, world: &mut World) {
        world
            .resource_mut::<EventQueue>()
            .0
            .push_back((event, context));
    }
    fn pop_action(world: &mut World) -> Option<Action> {
        world.resource_mut::<ActionQueue>().0.pop_front()
    }
    fn pop_event(world: &mut World) -> Option<(Event, Context)> {
        world.resource_mut::<EventQueue>().0.pop_front()
    }
    pub fn register_event(event: Event, world: &mut World) {
        world
            .resource_mut::<ActionsData>()
            .events
            .push((gt().insert_head(), event));
    }
    pub fn register_next_turn(world: &mut World) {
        let mut data = world.resource_mut::<ActionsData>();
        let next = data.turns.last().map(|(_, r)| *r).unwrap_or_default() + 1;
        data.turns.push((gt().insert_head(), next));
        data.chain = 0;
    }
    pub fn register_sound_effect(sfx: SoundEffect, world: &mut World) {
        world
            .resource_mut::<ActionsData>()
            .sounds
            .push((gt().insert_head(), sfx));
    }
    fn queue_current_sound_effect(from: f32, to: f32, world: &World) {
        if from >= to || to - from > 1.0 {
            return;
        }
        let Some(ad) = world.get_resource::<ActionsData>() else {
            return;
        };
        for (ts, sfx) in ad.sounds.iter().copied() {
            if ts >= from {
                if ts <= to {
                    AudioPlugin::queue_sound(sfx);
                } else {
                    break;
                }
            }
        }
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

    pub fn reset(world: &mut World) {
        *world.resource_mut::<ActionsData>() = default();
    }
}
