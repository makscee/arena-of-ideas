use std::collections::VecDeque;

use super::*;

pub struct ActionPlugin;

#[derive(Resource)]
struct Timeframe(f32);

impl Default for Timeframe {
    fn default() -> Self {
        Self(0.2)
    }
}

impl Plugin for ActionPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<ActionQueue>()
            .add_systems(Update, Self::process_checker)
            .init_resource::<Timeframe>();
    }
}

impl ActionPlugin {
    fn process_cluster(timeframe: f32, world: &mut World) -> bool {
        let mut processed = false;
        while let Some(action) = ActionCluster::current(world).pop_next_action() {
            action.invoke(world);
            processed = true;
        }
        let cluster = ActionCluster::take(world).unwrap();
        if !cluster.changes.is_empty() {
            cluster.commit(timeframe, world);
            processed = true;
        }
        processed
    }

    pub fn spin(world: &mut World) -> bool {
        let mut processed = false;
        let timeframe = world.resource::<Timeframe>().0;
        loop {
            start_batch(world);
            if Self::process_cluster(timeframe, world) {
                processed = true;
                GameTimer::get_mut(world)
                    .to_batch_start()
                    .advance_insert(timeframe)
                    .end_batch()
                    .start_batch();
                if UnitPlugin::run_death_check(world) {
                    UnitPlugin::fill_slot_gaps(Faction::Left, world);
                    UnitPlugin::fill_slot_gaps(Faction::Right, world);
                    UnitPlugin::translate_to_slots(world);
                    GameTimer::get_mut(world)
                        .to_batch_start()
                        .advance_insert(timeframe)
                        .end_batch()
                        .start_batch();
                }
            } else {
                end_batch(world);
                break;
            }
            end_batch(world);
        }
        processed
    }

    fn process_checker(world: &mut World) {
        Self::spin(world);
    }

    pub fn new_cluster(effect: Effect, context: Context, world: &mut World) {
        world
            .resource_mut::<ActionQueue>()
            .0
            .push_back(ActionCluster {
                actions: [Action { effect, context }].into(),
                ..default()
            });
    }

    pub fn new_cluster_many(actions: Vec<Action>, world: &mut World) {
        world
            .resource_mut::<ActionQueue>()
            .0
            .push_back(ActionCluster {
                actions: actions.into(),
                ..default()
            });
    }

    pub fn set_timeframe(t: f32, world: &mut World) {
        world.insert_resource(Timeframe(t));
    }
}

#[derive(Debug, Default, Resource)]
pub struct ActionCluster {
    pub actions: VecDeque<Action>,
    pub changes: HashMap<usize, Vec<QueuedChange>>,
    pub order: usize,
}

#[derive(Debug)]
pub struct QueuedChange {
    entity: Entity,
    change: ChangeType,
}

#[derive(Debug)]
pub enum ChangeType {
    Var { var: VarName, change: VarChange },
    Birth,
}

impl ChangeType {
    fn process(self, entity: Entity, world: &mut World) {
        match self {
            ChangeType::Var { var, change } => {
                VarState::push_back(entity, var, change, world);
            }
            ChangeType::Birth => {
                let ts = get_insert_head(world);
                if let Ok(mut s) = VarState::try_get_mut(entity, world) {
                    s.birth = ts;
                }
            }
        }
    }

    fn adust_time(&mut self, factor: f32) -> &mut Self {
        match self {
            ChangeType::Var { change, .. } => {
                change.adjust_time(factor);
            }
            _ => {}
        };
        self
    }

    fn timeframe(&self) -> f32 {
        match self {
            ChangeType::Var { change, .. } => change.timeframe,
            ChangeType::Birth => 0.0,
        }
    }
}

impl ActionCluster {
    pub fn current(world: &mut World) -> Mut<Self> {
        if !world.contains_resource::<ActionCluster>() {
            let cluster = world
                .resource_mut::<ActionQueue>()
                .0
                .pop_front()
                .unwrap_or_default();
            world.insert_resource(cluster);
        }
        world.resource_mut::<ActionCluster>()
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
                    .map(|c| c.change.timeframe())
                    .reduce(f32::max)
                    .unwrap_or_default()
            })
            .sum();
        let orders = self.changes.len();
        let step = timeframe / orders as f32;
        let factor = (timeframe / total_duration).min(1.0);
        for (_, changes) in self.changes.drain().sorted_by_key(|(c, _)| *c) {
            start_batch(world);
            for QueuedChange { entity, mut change } in changes {
                change.adust_time(factor);
                change.process(entity, world);
                to_batch_start(world);
            }
            GameTimer::get_mut(world)
                .to_batch_start()
                .advance_insert(step)
                .end_batch();
        }
    }

    pub fn push_change(&mut self, entity: Entity, change: ChangeType) -> &mut Self {
        self.changes
            .entry(self.order)
            .or_default()
            .push(QueuedChange { entity, change });
        self
    }

    pub fn push_var_change(
        &mut self,
        var: VarName,
        change: VarChange,
        context: Context,
    ) -> &mut Self {
        self.push_change(context.owner(), ChangeType::Var { var, change });
        self
    }

    pub fn push_state_birth(&mut self, entity: Entity) -> &mut Self {
        self.push_change(entity, ChangeType::Birth);
        self
    }

    pub fn incr_order(&mut self) -> &mut Self {
        self.order += 1;
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
        if let Some(owner) = context.get_owner() {
            if UnitPlugin::is_dead(owner, world) {
                return;
            }
        }
        match effect.invoke(&mut context, world) {
            Ok(_) => {
                for entity in world
                    .query_filtered::<Entity, (With<Status>, With<VarStateDelta>, With<Parent>)>()
                    .iter(world)
                    .collect_vec()
                {
                    Status::refresh_entity_mapping(entity, world);
                }
            }
            Err(err) => error!("Effect process error {err}\n{effect}\n---\n{context}"),
        }
    }
}

#[derive(Resource, Default)]
pub struct ActionQueue(VecDeque<ActionCluster>);
