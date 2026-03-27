use super::*;

/// Run a quick battle preview: the unit being created vs a dummy unit.
/// Returns a summary string of what happened.
pub fn run_battle_preview(
    trigger: Trigger,
    target: Target,
    effect_code: &str,
    pwr: i32,
    hp: i32,
) -> String {
    if effect_code.is_empty() {
        return "No script to preview".to_string();
    }

    let left_team = build_team(10000, "Preview", pwr, hp, trigger, target, effect_code);
    let right_team = build_team(
        20000,
        "Dummy",
        1,
        5,
        Trigger::BattleStart,
        Target::Owner,
        "",
    );

    let battle = Battle {
        id: 99999,
        left: left_team,
        right: right_team,
    };

    let mut source = battle.to_source();

    if let Err(e) = source.exec_context(|ctx| BattleSimulation::start(ctx)) {
        return format!("Battle start error: {e}");
    }

    let mut iterations = 0;
    let max_iterations = 10;
    while iterations < max_iterations {
        let ended = source.battle().map_or(true, |s| s.ended());
        if ended {
            break;
        }
        if let Err(e) = source.exec_context(|ctx| BattleSimulation::run(ctx)) {
            return format!("Battle error at round {}: {e}", iterations + 1);
        }
        iterations += 1;
    }

    match source.battle() {
        Ok(sim) => {
            let left_alive = sim.left_units().len();
            let right_alive = sim.right_units().len();
            let winner = if sim.ended() {
                match (left_alive == 0, right_alive == 0) {
                    (true, false) => "Enemy wins",
                    (false, true) => "Your unit wins!",
                    (true, true) => "Draw (both dead)",
                    (false, false) => "Draw (timeout)",
                }
            } else {
                "Battle didn't finish"
            };
            format!(
                "{} | {} rounds | Your unit: {}, Enemy: {}",
                winner,
                iterations,
                if left_alive > 0 { "alive" } else { "dead" },
                if right_alive > 0 { "alive" } else { "dead" },
            )
        }
        Err(e) => format!("Failed to get battle state: {e}"),
    }
}

fn build_team(
    base_id: u64,
    name: &str,
    pwr: i32,
    hp: i32,
    trigger: Trigger,
    target: Target,
    effect_code: &str,
) -> NTeam {
    let mut stats = NUnitStats::default();
    stats.set_id(base_id + 1);
    stats.pwr = pwr;
    stats.hp = hp;

    let mut state = NUnitState::default();
    state.set_id(base_id + 2);
    state.stax = 1;

    let mut representation = NRepresentation::default();
    representation.set_id(base_id + 4);

    let mut behavior = NUnitBehavior::default();
    behavior.set_id(base_id + 3);
    behavior.trigger = trigger;
    behavior.target = target;
    if !effect_code.is_empty() {
        behavior.effect =
            RhaiScript::new(effect_code.to_string()).with_description("Preview".to_string());
    }
    behavior.stats.set_loaded(stats);
    behavior.representation.set_loaded(representation);

    let mut unit = NUnit::default();
    unit.set_id(base_id + 100);
    unit.unit_name = format!("{name} Unit");
    unit.state.set_loaded(state);
    unit.behavior.set_loaded(behavior);

    let slot = NTeamSlot::new(base_id + 200, 0, 0).with_unit(unit);

    let mut color = NHouseColor::default();
    color.set_id(base_id + 301);
    color.color = HexColor("#FF0000".to_string());

    let mut house = NHouse::default();
    house.set_id(base_id + 300);
    house.house_name = format!("{name} House");
    house.color.set_loaded(color);

    let mut team = NTeam::default();
    team.set_id(base_id);
    team.slots = OwnedMultiple::new_loaded(base_id, vec![slot]);
    team.houses = OwnedMultiple::new_loaded(base_id, vec![house]);
    team
}
