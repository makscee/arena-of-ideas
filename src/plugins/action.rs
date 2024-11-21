use std::collections::VecDeque;

use rand::RngCore;

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
pub struct ActionsResource {
    events: Vec<(f32, Event)>,
    turns: Vec<(f32, usize)>,
    sounds: Vec<(f32, SoundEffect)>,
    chain: HashMap<Entity, usize>,
    pub rng: Option<ChaCha8Rng>,
}
fn rm(world: &mut World) -> Mut<ActionsResource> {
    world.resource_mut::<ActionsResource>()
}

impl Plugin for ActionPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<EventQueue>()
            .init_resource::<ActionQueue>()
            .init_resource::<ActionsResource>()
            .add_systems(Update, Self::update);
    }
}

impl ActionPlugin {
    fn update(world: &mut World) {
        Self::spin(world).expect("Spin failed");
        let ph = gt().play_head();
        Self::queue_current_sound_effect(ph - gt().last_delta(), ph, world)
    }
    pub fn resource(world: &mut World) -> Mut<ActionsResource> {
        rm(world)
    }
    pub fn spin(world: &mut World) -> Result<bool> {
        let mut processed = false;
        let mut limit = 100000;
        let mut hasher = DefaultHasher::new();
        rm(world).turns.len().hash(&mut hasher);
        let br = world.resource::<BattleResource>();
        let id = if br.id > 0 {
            br.id
        } else {
            thread_rng().next_u64()
        };
        id.hash(&mut hasher);
        br.left.units.iter().for_each(|u| u.name.hash(&mut hasher));
        br.right.units.iter().for_each(|u| u.name.hash(&mut hasher));
        let rng = rng_seeded(hasher.finish());
        rm(world).rng = Some(rng);
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
                let owner = context.owner();
                if world.get::<Corpse>(owner).is_some() {
                    error!(
                        "{} is dead, drop effect {}",
                        entity_name_with_id(owner),
                        effect.cstr()
                    );
                    continue;
                }
                Self::calculate_deafness(owner, world);
                if Self::deafness(owner, world) {
                    TextColumnPlugin::add(
                        owner,
                        "Deafness! ".cstr_cs(RED, CstrStyle::Bold)
                            + &"Skip ".cstr_c(VISIBLE_LIGHT)
                            + &effect.cstr(),
                        world,
                    );
                    continue;
                }
                match effect.invoke(&mut context, world) {
                    Ok(_) => {
                        processed = true;
                        *rm(world).chain.entry(owner).or_default() += 1;
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
        rm(world).rng = None;
        Ok(processed)
    }
    fn calculate_deafness(entity: Entity, world: &mut World) {
        let chain = rm(world).chain.get(&entity).copied().unwrap_or_default();
        if let Ok(mut state) = VarState::try_get_mut(entity, world) {
            let bs = global_settings().battle;
            let chance =
                ((chain as f32 - bs.deafness_start as f32) * bs.deafness_per_turn).clamp(0.0, 1.0);
            state.set_float(VarName::Deafness, chance);
        }
    }
    fn deafness(entity: Entity, world: &mut World) -> bool {
        let chance = VarState::get(entity, world)
            .get_value_last(VarName::Deafness)
            .and_then(|v| v.get_float())
            .unwrap_or_default() as f64;
        let mut r = rm(world);
        if let Some(rng) = r.rng.as_mut() {
            rng.gen_bool(chance)
        } else {
            thread_rng().gen_bool(chance)
        }
    }
    pub fn clear_dead(world: &mut World) -> bool {
        let dead = UnitPlugin::run_death_check(world);
        let died = !dead.is_empty();
        if died {
            let _ = Self::spin(world);
        }
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
            .resource_mut::<ActionsResource>()
            .events
            .push((gt().insert_head(), event));
    }
    pub fn register_next_turn(world: &mut World) {
        let mut data = rm(world);
        let next = data.turns.last().map(|(_, r)| *r).unwrap_or_default() + 1;
        data.turns.push((gt().insert_head(), next));
        data.chain.clear();
    }
    pub fn register_sound_effect(sfx: SoundEffect, world: &mut World) {
        world
            .resource_mut::<ActionsResource>()
            .sounds
            .push((gt().insert_head(), sfx));
    }
    fn queue_current_sound_effect(from: f32, to: f32, world: &World) {
        if from >= to || to - from > 1.0 {
            return;
        }
        let Some(ad) = world.get_resource::<ActionsResource>() else {
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
    pub fn get_turn(t: f32, world: &World) -> usize {
        world
            .get_resource::<ActionsResource>()
            .and_then(|d| {
                d.turns.iter().rev().find_map(|(ts, e)| match t >= *ts {
                    true => Some(*e),
                    false => None,
                })
            })
            .unwrap_or_default()
    }
    pub fn get_event(t: f32, world: &World) -> Option<Event> {
        world.get_resource::<ActionsResource>().and_then(|d| {
            d.events.iter().rev().find_map(|(ts, e)| match t >= *ts {
                true => Some(e.clone()),
                false => None,
            })
        })
    }
    pub fn collect_events(until: f32, world: &World) -> Vec<(f32, Event)> {
        let Some(ar) = world.get_resource::<ActionsResource>() else {
            return default();
        };
        ar.events
            .iter()
            .filter(|(t, _)| *t <= until)
            .cloned()
            .collect_vec()
    }

    pub fn reset(world: &mut World) {
        *rm(world) = default();
    }
}
