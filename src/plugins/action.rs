use std::collections::VecDeque;

use super::*;

pub struct ActionPlugin;

impl Plugin for ActionPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<EventQueue>()
            .init_resource::<ActionQueue>()
            .add_systems(Update, Self::update);
    }
}

#[derive(Resource, Default)]
struct EventQueue(VecDeque<(Event, Context)>);
#[derive(Resource, Default)]
struct ActionQueue(VecDeque<Action>);

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
        let mut limit = 100;
        loop {
            if limit == 0 {
                return Err(anyhow!("Limit exceeeded"));
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
        }
        Ok(processed)
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
    pub fn action_push_back_with_dealy(
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
    pub fn action_push_front_with_dealy(
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

    fn pop_action(world: &mut World) -> Option<Action> {
        world.resource_mut::<ActionQueue>().0.pop_front()
    }
    fn pop_event(world: &mut World) -> Option<(Event, Context)> {
        world.resource_mut::<EventQueue>().0.pop_front()
    }
}
