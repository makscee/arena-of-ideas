use std::collections::VecDeque;

use super::*;

pub struct ActionPlugin;

impl Plugin for ActionPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<ActionQueue>()
            .add_systems(Update, Self::process_checker);
    }
}

impl ActionPlugin {
    fn process_queue(action_delay: f32, world: &mut World) -> bool {
        start_batch(world);
        let mut processed = false;
        while let Some(action) = world.resource_mut::<ActionQueue>().pop_action() {
            to_batch_start(world);
            action.invoke(world);
            processed = true;
            GameTimer::get_mut(world)
                .to_batch_start()
                .advance_insert(action_delay)
                .end_batch()
                .start_batch();
        }
        end_batch(world);
        world.resource_mut::<ActionQueue>().remove_empty_front();
        processed
    }

    fn process_checker(world: &mut World) {
        Self::spin(0.1, 0.02, world);
    }

    pub fn spin(cluster_delay: f32, action_delay: f32, world: &mut World) -> bool {
        let mut inserted = false;
        let mut died = false;
        loop {
            let t = get_end(world);
            if Self::process_queue(action_delay, world) {
                died |= UnitPlugin::run_death_check(world);
                let end = get_end(world);
                if t != end {
                    inserted = true;
                    if cluster_delay > 0.0 {
                        let insert = ((end / cluster_delay).floor() + 1.0) * cluster_delay;
                        GameTimer::get_mut(world).insert_head_to(insert);
                    }
                }
            } else {
                break;
            }
        }
        if died {
            UnitPlugin::fill_slot_gaps(Faction::Left, world);
            UnitPlugin::fill_slot_gaps(Faction::Right, world);
        }
        GameTimer::get_mut(world).end_batch();
        inserted
    }

    pub fn push_back_cluster(cluster: ActionCluster, world: &mut World) {
        world
            .get_resource_mut::<ActionQueue>()
            .unwrap()
            .push_back(cluster);
    }

    pub fn push_back(effect: Effect, context: Context, world: &mut World) {
        world
            .resource_mut::<ActionQueue>()
            .last_mut()
            .unwrap()
            .actions
            .push_back(Action { effect, context });
    }

    pub fn push_front_cluster(cluster: ActionCluster, world: &mut World) {
        world
            .get_resource_mut::<ActionQueue>()
            .unwrap()
            .push_front(cluster);
    }

    pub fn push_front(effect: Effect, context: Context, world: &mut World) {
        world
            .resource_mut::<ActionQueue>()
            .first_mut()
            .unwrap()
            .actions
            .push_front(Action { effect, context });
    }
}

#[derive(Debug, Default)]
pub struct ActionCluster {
    pub actions: VecDeque<Action>,
}

#[derive(Debug)]
pub struct Action {
    pub effect: Effect,
    pub context: Context,
}

impl ActionCluster {
    pub fn single(context: Context, effect: Effect) -> Self {
        Self {
            actions: [Action { context, effect }].into(),
        }
    }
}

impl Action {
    pub fn invoke(self, world: &mut World) {
        let Action {
            effect,
            mut context,
        } = self;
        match effect.invoke(&mut context, world) {
            Ok(_) => {
                for entity in world
                    .query_filtered::<Entity, (With<Status>, With<VarStateDelta>, With<Parent>)>()
                    .iter(world)
                    .collect_vec()
                {
                    Status::apply_delta(entity, world);
                }
            }
            Err(err) => error!("Effect process error {err}\n{effect}\n---\n{context}"),
        }
    }
}

#[derive(Resource, Default)]
pub struct ActionQueue(VecDeque<ActionCluster>);

impl ActionQueue {
    pub fn push_back(&mut self, action: ActionCluster) {
        self.0.push_back(action)
    }
    pub fn push_front(&mut self, action: ActionCluster) {
        self.0.push_front(action)
    }
    pub fn last_mut(&mut self) -> Option<&mut ActionCluster> {
        self.0.back_mut()
    }
    pub fn first_mut(&mut self) -> Option<&mut ActionCluster> {
        self.0.front_mut()
    }
    pub fn pop_action(&mut self) -> Option<Action> {
        if let Some(action) = self.first_mut() {
            action.actions.pop_front()
        } else {
            None
        }
    }
    pub fn remove_empty_front(&mut self) {
        if let Some(front) = self.0.front() {
            if front.actions.is_empty() {
                self.0.pop_front();
            }
        }
    }
}
