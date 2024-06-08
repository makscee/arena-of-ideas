use std::collections::VecDeque;

use super::*;

pub struct ActionPlugin;

impl Plugin for ActionPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<ActionQueue>();
    }
}

impl ActionPlugin {
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
                        processed = true;
                        GameTimer::get().advance_insert(delay);
                    }
                    Err(e) => error!("Effect process error: {e}"),
                }
                continue;
            }
            break;
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
            // UnitPlugin::translate_to_slots(world);
            GameTimer::get().insert_to_end();
        } else {
            GameTimer::get().advance_insert(0.3);
        }
        died
    }
    fn pop_action(world: &mut World) -> Option<Action> {
        world.resource_mut::<ActionQueue>().0.pop_front()
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
}

#[derive(Resource, Default)]
struct ActionQueue(VecDeque<Action>);

struct Action {
    effect: Effect,
    context: Context,
    delay: f32,
}
