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
    fn process_cluster(timeframe: f32, world: &mut World) -> bool {
        let mut processed = false;
        let mut order = 0;
        while let Some(mut action) = ActionCluster::get(world).pop_next_action() {
            debug!("Invoke {action:?}");
            action.context.order = order;
            action.invoke(world);
            processed = true;
            order += 1;
        }
        let cluster = ActionCluster::take(world).unwrap();
        if !cluster.changes.is_empty() {
            cluster.commit(timeframe, world);
            processed = true;
        }
        processed
    }

    pub fn spin(timeframe: f32, world: &mut World) -> bool {
        let mut processed = false;
        while ActionQueue::start_next_cluster(world) {
            start_batch(world);
            if Self::process_cluster(timeframe, world) {
                processed = true;
                GameTimer::get_mut(world)
                    .to_batch_start()
                    .advance_insert(timeframe);
            }
            end_batch(world);
        }

        UnitPlugin::run_death_check(world);
        UnitPlugin::fill_slot_gaps(Faction::Left, world);
        UnitPlugin::fill_slot_gaps(Faction::Right, world);
        processed
    }

    fn process_checker(world: &mut World) {
        Self::spin(0.1, world);
    }

    pub fn new_cluster(effect: Effect, context: Context, world: &mut World) {
        world
            .resource_mut::<ActionQueue>()
            .0
            .push_back(ActionCluster {
                actions: [Action { effect, context }].into(),
                changes: default(),
            });
    }

    pub fn new_cluster_many(actions: Vec<Action>, world: &mut World) {
        world
            .resource_mut::<ActionQueue>()
            .0
            .push_back(ActionCluster {
                actions: actions.into(),
                changes: default(),
            });
    }
}

#[derive(Debug, Default, Resource)]
pub struct ActionCluster {
    pub actions: VecDeque<Action>,
    pub changes: HashMap<usize, Vec<QueuedChange>>,
}

#[derive(Debug)]
pub struct QueuedChange {
    pub entity: Entity,
    pub var: VarName,
    pub change: Change,
    pub timeframe: f32,
}

impl ActionCluster {
    pub fn get(world: &mut World) -> Mut<Self> {
        world.get_resource_or_insert_with(|| default())
    }
    pub fn take(world: &mut World) -> Option<Self> {
        world.remove_resource::<ActionCluster>()
    }

    pub fn push_action_front(&mut self, effect: Effect, context: Context) -> &mut Self {
        self.actions.push_front(Action { effect, context });
        self
    }

    pub fn push_action_back(&mut self, effect: Effect, context: Context) -> &mut Self {
        self.actions.push_back(Action { effect, context });
        self
    }

    pub fn pop_next_action(&mut self) -> Option<Action> {
        self.actions.pop_front()
    }

    /// Spread changes according to order trying to fit within timeframe
    pub fn commit(mut self, timeframe: f32, world: &mut World) {
        if !self.actions.is_empty() {
            panic!("All actions should be processed before commit");
        }
        let total_duration: f32 = self
            .changes
            .values()
            .map(|c| {
                c.iter()
                    .map(|c| c.timeframe)
                    .reduce(f32::max)
                    .unwrap_or_default()
            })
            .sum();
        let factor = (timeframe / total_duration).min(1.0);
        for (_, changes) in self.changes.drain().sorted_by_key(|(c, _)| *c) {
            start_batch(world);
            let mut step = 0.0_f32;
            for QueuedChange {
                entity,
                var,
                change,
                timeframe,
            } in changes
            {
                step = step.max(timeframe);
                let change = change.adjust_time(factor);
                VarState::push_back(entity, var, change, world);
                to_batch_start(world);
            }
            GameTimer::get_mut(world)
                .to_batch_start()
                .advance_insert(step)
                .end_batch();
        }
    }

    pub fn push_change(
        &mut self,
        var: VarName,
        change: Change,
        timeframe: f32,
        context: Context,
    ) -> &mut Self {
        self.changes
            .entry(context.order)
            .or_default()
            .push(QueuedChange {
                entity: context.owner(),
                var,
                change,
                timeframe,
            });
        self
    }
}

#[derive(Debug)]
pub struct Action {
    pub effect: Effect,
    pub context: Context,
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
    fn start_next_cluster(world: &mut World) -> bool {
        let cluster = ActionCluster::get(world);
        if !cluster.actions.is_empty() || !cluster.changes.is_empty() {
            return true;
        }
        let mut q = world.resource_mut::<ActionQueue>();
        if let Some(cluster) = q.0.pop_front() {
            world.insert_resource(cluster);
            true
        } else {
            false
        }
    }
}
