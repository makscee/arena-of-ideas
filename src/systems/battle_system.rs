use super::*;
use geng::ui::*;

pub struct BattleSystem {}

impl BattleSystem {
    pub fn new() -> Self {
        Self {}
    }

    pub fn run_battle(
        world: &mut legion::World,
        resources: &mut Resources,
        tape: &mut Option<Tape>,
    ) -> usize {
        Self::create_tape_entities(world, resources, tape);
        let mut cluster = match tape {
            Some(_) => Some(NodeCluster::default()),
            None => None,
        };
        SlotSystem::move_to_slots_animated(world, resources, cluster.as_mut());
        Event::BattleStart.send(world, resources);
        ActionSystem::spin(world, resources, cluster.as_mut());
        if let Some(tape) = tape {
            tape.push(cluster.unwrap());
        }
        let mut turns = 0;
        loop {
            turns += 1;
            resources.battle_data.turns = turns;
            if turns > 100 {
                error!(
                    "Exceeded turns limit: {turns}, {} x {}",
                    TeamSystem::get_state(Faction::Light, world).name,
                    TeamSystem::get_state(Faction::Dark, world).name
                );
                return 0;
            }
            // if turns % 10 == 0 {
            //     Self::promote_all(world);
            // }
            let result = Self::turn(world, resources, tape);
            if !result {
                let faction = if UnitSystem::collect_faction(world, Faction::Dark).is_empty() {
                    Some(Faction::Dark)
                } else {
                    if UnitSystem::collect_faction(world, Faction::Light).is_empty() {
                        Some(Faction::Light)
                    } else {
                        None
                    }
                };
                if let Some(faction) = faction {
                    if let Some(queue) = resources.battle_data.team_queue.get_mut(&faction) {
                        let team = queue.pop_front();
                        if let Some(team) = team {
                            team.unpack(&faction, world, resources);
                            continue;
                        }
                    }
                }
                break;
            }
        }
        Self::clear_tape_entities(world, resources, tape);
        resources.battle_data.team_queue.clear();
        Ladder::get_score(world)
    }

    fn create_tape_entities(
        world: &mut legion::World,
        resources: &mut Resources,
        tape: &mut Option<Tape>,
    ) {
        if tape.is_none() {
            return;
        }

        let (left, right) = VfxSystem::vfx_battle_team_names(world, resources);
        let names = (
            Self::push_tape_shader_entity(left, world),
            Self::push_tape_shader_entity(right, world),
        );
        resources.battle_data.team_names_entitities = Some(names);
    }

    fn push_tape_shader_entity(shader: ShaderChain, world: &mut legion::World) -> legion::Entity {
        let entity = world.push((shader, TapeEntityComponent {}));
        world
            .entry(entity)
            .unwrap()
            .add_component(EntityComponent::new(entity));
        entity
    }

    fn clear_tape_entities(
        world: &mut legion::World,
        resources: &mut Resources,
        tape: &mut Option<Tape>,
    ) {
        if tape.is_none() {
            return;
        }
        if let Some((left, right)) = resources.battle_data.team_names_entitities {
            world.remove(left);
            world.remove(right);
        }
    }

    pub fn queue_team(faction: Faction, team: PackedTeam, resources: &mut Resources) {
        resources
            .battle_data
            .team_queue
            .entry(faction)
            .or_default()
            .push_back(team);
    }

    pub fn init_battle(
        light: &PackedTeam,
        dark: &PackedTeam,
        world: &mut legion::World,
        resources: &mut Resources,
    ) {
        Self::clear_world(world, resources);
        light.unpack(&Faction::Light, world, resources);
        dark.unpack(&Faction::Dark, world, resources);
        resources.battle_data.turns = 0;
    }

    pub fn finish_ladder_battle(world: &mut legion::World, resources: &mut Resources) {
        resources.battle_data.last_score = Ladder::get_score(world);
        resources.battle_data.total_score += resources.battle_data.last_score;
        let level = Ladder::current_level(resources) + 1;
        let (title, text, buttons, color) = if resources.battle_data.last_score > 0 {
            let score = resources.battle_data.last_score;
            let difficulty = resources.battle_data.last_difficulty + 3;
            if Ladder::next(resources) {
                resources.transition_state = GameState::Sacrifice;
                // ShopSystem::change_g(score as i32, Some("Battle Score"), world, resources);
                // ShopSystem::change_g(
                //     difficulty as i32,
                //     Some("Enemy Difficulty"),
                //     world,
                //     resources,
                // );
            } else {
                resources.transition_state = GameState::GameOver;
            }
            let difficulty_text = match resources.battle_data.last_difficulty {
                0 => "Easy",
                1 => "Medium",
                2 => "Hard",
                _ => panic!(
                    "Wrong difficulty index {}",
                    resources.battle_data.last_difficulty
                ),
            };
            (
                "Victory",
                format!("Level {level} complete"),
                vec![PanelFooterButton::Close],
                resources.options.colors.victory,
            )
        } else {
            resources.transition_state = GameState::GameOver;
            (
                "Defeat",
                format!(
                    "Game Over\nTotal score: {}",
                    resources.battle_data.total_score
                ),
                vec![PanelFooterButton::Restart],
                resources.options.colors.defeat,
            )
        };
        Self::clear_world(world, resources);
        PanelsSystem::add_alert(color, title, &text, vec2(0.0, 0.3), buttons, resources);
        SaveSystem::save(world, resources);
    }

    pub fn clear_world(world: &mut legion::World, resources: &mut Resources) {
        UnitSystem::clear_factions(world, &Faction::battle());
    }

    fn strickers_death_check(
        left: legion::Entity,
        right: legion::Entity,
        world: &legion::World,
    ) -> bool {
        UnitSystem::get_corpse(left, world).is_some()
            || UnitSystem::get_corpse(right, world).is_some()
    }

    pub fn turn(
        world: &mut legion::World,
        resources: &mut Resources,
        tape: &mut Option<Tape>,
    ) -> bool {
        for faction in Faction::battle() {
            SlotSystem::fill_gaps(faction, world);
        }
        let mut cluster = match tape {
            Some(_) => Some(NodeCluster::default()),
            None => None,
        };
        ActionSystem::spin(world, resources, cluster.as_mut());
        SlotSystem::move_to_slots_animated(world, resources, cluster.as_mut());
        if let Some((left, right)) = Self::find_hitters(world) {
            Event::TurnStart.send(world, resources);
            ActionSystem::spin(world, resources, cluster.as_mut());
            if Self::strickers_death_check(left, right, world) {
                if let Some(tape) = tape {
                    tape.push(cluster.unwrap());
                }
                return true;
            }

            Event::BeforeStrike {
                owner: left,
                target: right,
            }
            .send(world, resources);
            ActionSystem::spin(world, resources, cluster.as_mut());
            if Self::strickers_death_check(left, right, world) {
                if let Some(tape) = tape {
                    tape.push(cluster.unwrap());
                }
                return true;
            }

            Event::BeforeStrike {
                owner: right,
                target: left,
            }
            .send(world, resources);
            ActionSystem::spin(world, resources, cluster.as_mut());
            if Self::strickers_death_check(left, right, world) {
                if let Some(tape) = tape {
                    tape.push(cluster.unwrap());
                }
                return true;
            }
            if let Some(tape) = tape {
                tape.push({
                    let mut cluster = cluster.unwrap();
                    // cluster.set_duration(1.0);
                    cluster
                });
                cluster = Some(NodeCluster::default());
            }

            let scale = resources.options.floats.slots_striker_scale;
            let (left_hit_pos, right_hit_pos) = (vec2(-1.0 * scale, 0.0), vec2(1.0 * scale, 0.0));

            if let Some(tape) = tape {
                let mut node = Node::default();
                VfxSystem::translate_animated(
                    left,
                    left_hit_pos,
                    &mut node,
                    world,
                    EasingType::Linear,
                    0.03,
                );
                VfxSystem::translate_animated(
                    right,
                    right_hit_pos,
                    &mut node,
                    world,
                    EasingType::Linear,
                    0.03,
                );
                let mut cluster =
                    NodeCluster::new(node.lock(NodeLockType::Full { world, resources }));
                cluster.push(Node::default().lock(NodeLockType::Full { world, resources }));
                cluster.set_duration(0.5);
                tape.push(cluster);
            }

            let duration = 1.5;
            if let Some(cluster) = &mut cluster {
                let mut node = Node::default();
                VfxSystem::translate_animated(
                    left,
                    SlotSystem::get_position(1, &Faction::Light, resources),
                    &mut node,
                    world,
                    EasingType::QuartOut,
                    duration,
                );
                VfxSystem::translate_animated(
                    right,
                    SlotSystem::get_position(1, &Faction::Dark, resources),
                    &mut node,
                    world,
                    EasingType::QuartOut,
                    duration,
                );
                Self::add_strike_vfx(world, resources, &mut node);
                cluster.push(node.lock(NodeLockType::Full { world, resources }));
            }
            Self::hit(left, right, cluster.as_mut(), world, resources);
            ActionSystem::spin(world, resources, cluster.as_mut());
            Event::TurnEnd.send(world, resources);
            ActionSystem::spin(world, resources, cluster.as_mut());
            if let Some(tape) = tape {
                let mut cluster = cluster.unwrap();
                cluster.set_duration(duration);
                tape.push(cluster);
            }
            return true;
        }
        false
    }

    pub fn find_hitters(world: &legion::World) -> Option<(legion::Entity, legion::Entity)> {
        let mut light = None;
        let mut dark = None;
        for (entity, state) in <(&EntityComponent, &ContextState)>::query()
            .filter(component::<UnitComponent>())
            .iter(world)
        {
            if state.get_int(&VarName::Slot, world) == 1 {
                if state.get_faction(&VarName::Faction, world) == Faction::Light {
                    light = Some(entity.entity)
                } else if state.get_faction(&VarName::Faction, world) == Faction::Dark {
                    dark = Some(entity.entity)
                }
            }
        }
        if light.is_some() && dark.is_some() {
            Some((light.unwrap(), dark.unwrap()))
        } else {
            None
        }
    }

    pub fn hit(
        left: legion::Entity,
        right: legion::Entity,
        mut cluster: Option<&mut NodeCluster>,
        world: &mut legion::World,
        resources: &mut Resources,
    ) {
        ActionSystem::spin(world, resources, cluster.as_deref_mut());

        if UnitSystem::is_alive(left, world, resources)
            && UnitSystem::is_alive(right, world, resources)
        {
            resources.action_queue.push_back(Action::new(
                Context::new(ContextLayer::Unit { entity: left }, world, resources)
                    .set_target(right),
                Effect::Damage {
                    value: None,
                    on_hit: None,
                    source: default(),
                }
                .wrap(),
            ));
            resources.action_queue.push_back(Action::new(
                Context::new(ContextLayer::Unit { entity: right }, world, resources)
                    .set_target(left),
                Effect::Damage {
                    value: None,
                    on_hit: None,
                    source: default(),
                }
                .wrap(),
            ));
            ActionSystem::spin(world, resources, cluster.as_deref_mut());
        }

        if UnitSystem::is_alive(left, world, resources) {
            Event::AfterStrike {
                owner: left,
                target: right,
            }
            .send(world, resources);
            ActionSystem::spin(world, resources, cluster.as_deref_mut());
        }

        if UnitSystem::is_alive(right, world, resources) {
            Event::AfterStrike {
                owner: right,
                target: left,
            }
            .send(world, resources);
            ActionSystem::spin(world, resources, cluster.as_deref_mut());
        }
    }

    fn promote_all(world: &mut legion::World) {
        for unit in UnitSystem::collect_factions(world, &Faction::battle()) {
            let state = ContextState::get_mut(unit, world);
            let rank = state.vars.get_int(&VarName::Rank);
            if rank < 3 {
                state.vars.change_int(&VarName::Rank, 1);
            }
        }
    }

    fn add_strike_vfx(_: &mut legion::World, resources: &mut Resources, node: &mut Node) {
        let position = BATTLEFIELD_POSITION;
        node.add_effect(VfxSystem::vfx_strike(resources, position));
    }
}

impl System for BattleSystem {}
