use super::*;

pub struct BattlePlugin;

impl Plugin for BattlePlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(GameState::Battle), Self::on_enter)
            .add_systems(OnExit(GameState::Battle), |world: &mut World| {
                world.remove_resource::<BattleData>();
            });
    }
}

#[derive(Resource)]
pub struct BattleData {
    pub battle: Battle,
    pub source: Sources<'static>,
    pub t: f32,
    pub playback_speed: f32,
    pub playing: bool,
    pub on_done: Option<fn(u64, bool, u64)>,
}

impl BattleData {
    fn load(battle: Battle) -> Self {
        let mut source = battle.clone().to_source();

        source
            .exec_context(|ctx| BattleSimulation::start(ctx))
            .unwrap();

        Self {
            battle,
            source,
            t: 0.0,
            playing: true,
            playback_speed: 1.0,
            on_done: None,
        }
    }

    fn load_replay(battle: Battle) -> Self {
        let mut source = battle.clone().to_source();

        source
            .exec_context(|ctx| BattleSimulation::start(ctx))
            .unwrap();

        Self {
            battle,
            source,
            t: 0.0,
            playing: true,
            playback_speed: 1.0,
            on_done: None,
        }
    }
}

impl BattlePlugin {
    fn on_enter(world: &mut World) {
        if world.contains_resource::<BattleData>() {
            return;
        }

        let result = with_solid_source(|ctx| {
            let player_result = player(ctx);

            match player_result {
                Ok(p) => {
                    let m_clone = p.active_match.load_node(ctx)?;
                    if m_clone.state.is_battle() {
                        if let Some(battle_id) = m_clone.pending_battle {
                            if let Some(battle) = cn().db.battle().id().find(&battle_id) {
                                // Deserialize teams from packed strings
                                let left = if let Ok(packed) =
                                    PackedNodes::from_string(&battle.left_team)
                                {
                                    NTeam::unpack(&packed)
                                        .unwrap_or_else(|_| NTeam::default().with_id(next_id()))
                                } else {
                                    NTeam::default().with_id(next_id())
                                };
                                let right = if let Ok(packed) =
                                    PackedNodes::from_string(&battle.right_team)
                                {
                                    NTeam::unpack(&packed)
                                        .unwrap_or_else(|_| NTeam::default().with_id(next_id()))
                                } else {
                                    NTeam::default().with_id(next_id())
                                };
                                Ok(Some((battle.id, Some((left, right)), false)))
                            } else {
                                Ok(None)
                            }
                        } else {
                            Ok(None)
                        }
                    } else {
                        Ok(None)
                    }
                }
                Err(_) => Ok(None),
            }
        });

        match result {
            Ok(Some((id, teams, _is_active_battle))) => {
                if let Some((left, right)) = teams {
                    Self::load_teams(id, left, right, world);
                    Self::on_done_callback(
                        |id, result, hash| {
                            cn().reducers
                                .match_submit_battle_result(id, result, hash)
                                .notify_op();
                        },
                        world,
                    );
                } else {
                    error!("Failed to get teams for battle");
                    GameState::Shop.set_next(world);
                }
            }
            Ok(None) => {
                debug!("No active battle, go back to Shop");
                GameState::Shop.set_next(world);
            }
            Err(e) => {
                e.cstr().notify_error(world);
                GameState::Shop.set_next(world);
            }
        }
    }

    pub fn load_teams(id: u64, mut left: NTeam, mut right: NTeam, world: &mut World) {
        let slots = global_settings().team_slots as usize;
        for team in [&mut left, &mut right] {
            if !team.slots.is_loaded() {
                team.slots_set(default()).unwrap();
            }
            while team.slots.get().unwrap().len() < slots {
                let mut slot = NTeamSlot::default().with_id(next_id());
                slot.index = team.slots.get().unwrap().len() as i32;
                slot.owner = team.owner;
                team.slots.get_mut().unwrap().push(slot);
            }
        }
        world.insert_resource(BattleData::load(Battle { left, right, id }));
    }

    pub fn load_replay(mut left: NTeam, mut right: NTeam, world: &mut World) {
        let slots = global_settings().team_slots as usize;
        for team in [&mut left, &mut right] {
            if !team.slots.is_loaded() {
                team.slots_set(default()).unwrap();
            }
            while team.slots.get().unwrap().len() < slots {
                let mut slot = NTeamSlot::default().with_id(next_id());
                slot.index = team.slots.get().unwrap().len() as i32;
                slot.owner = team.owner;
                team.slots.get_mut().unwrap().push(slot);
            }
        }
        let id = next_id();
        let mut battle_data = BattleData::load_replay(Battle { left, right, id });
        battle_data.on_done = Some(|_, _, _| {
            GameState::Title.set_next_op();
        });
        world.insert_resource(battle_data);
    }

    pub fn on_done_callback(f: fn(u64, bool, u64), world: &mut World) {
        if let Some(mut r) = world.get_resource_mut::<BattleData>() {
            r.on_done = Some(f);
        }
    }

    pub fn pane_view(ui: &mut Ui, world: &mut World) -> NodeResult<()> {
        world.resource_scope(|world, mut data: Mut<BattleData>| {
            BattleCamera::builder()
                .on_battle_end(|ui, _res| {})
                .show(&mut data, ui);

            // Update time
            if data.playing {
                let duration = data
                    .source
                    .exec_context_ref(|ctx| ctx.battle().map(|s| s.duration).unwrap_or(0.0));
                data.t += gt().last_delta() * data.playback_speed;
                data.t = data.t.at_most(duration);

                if data.t >= duration {
                    let not_ended = data
                        .source
                        .exec_context_ref(|ctx| ctx.battle().map(|s| !s.ended()).unwrap_or(false));
                    if not_ended {
                        data.source
                            .exec_context(|ctx| BattleSimulation::run(ctx))
                            .log();
                    }
                }
            }
        });
        Ok(())
    }
}
