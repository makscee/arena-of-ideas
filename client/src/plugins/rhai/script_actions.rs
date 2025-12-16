use super::*;
use ::rhai::Engine;
use serde::{Deserialize, Serialize};

/// Trait for script action types that defines how they are executed
pub trait ScriptAction: Clone + Send + Sync + 'static {
    /// The name of the variable in the script scope that holds the actions vec
    fn actions_var_name() -> &'static str;

    /// Register the type and its methods with the Rhai engine
    fn register_type(engine: &mut Engine);
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum UnitAction {
    UseAbility {
        ability_name: String,
        target_id: u64,
    },
    ApplyStatus {
        status_name: String,
        target_id: u64,
        stacks: i32,
    },
}

impl ScriptAction for UnitAction {
    fn actions_var_name() -> &'static str {
        "unit_actions"
    }

    fn register_type(engine: &mut Engine) {
        register_unit_type(engine);
        register_unit_actions_type(engine);
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum StatusAction {
    DealDamage {
        target_id: u64,
        amount: i32,
    },
    HealDamage {
        target_id: u64,
        amount: i32,
    },
    UseAbility {
        ability_name: String,
        target_id: u64,
    },
    ModifyStacks {
        delta: i32,
    },
}

impl ScriptAction for StatusAction {
    fn actions_var_name() -> &'static str {
        "status_actions"
    }

    fn register_type(engine: &mut Engine) {
        register_status_type(engine);
        register_status_actions_type(engine);
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AbilityAction {
    DealDamage {
        target_id: u64,
        amount: i32,
    },
    HealDamage {
        target_id: u64,
        amount: i32,
    },
    ChangeStatus {
        status_name: String,
        target_id: u64,
        delta: i32,
    },
}

impl ScriptAction for AbilityAction {
    fn actions_var_name() -> &'static str {
        "ability_actions"
    }

    fn register_type(engine: &mut Engine) {
        register_ability_type(engine);
        register_ability_actions_type(engine);
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RhaiPainterAction(pub String);

impl ScriptAction for RhaiPainterAction {
    fn actions_var_name() -> &'static str {
        "painter_actions"
    }

    fn register_type(engine: &mut Engine) {
        register_painter_functions(engine);
    }
}

pub fn register_unit_type(engine: &mut Engine) {
    engine
        .register_type_with_name::<NUnit>("Unit")
        .register_get("id", |unit: &mut NUnit| unit.id() as i64)
        .register_get("unit_name", |unit: &mut NUnit| unit.unit_name.clone());
}

pub fn register_unit_actions_type(engine: &mut Engine) {
    engine
        .register_type_with_name::<Vec<UnitAction>>("UnitActions")
        .register_fn(
            "use_ability",
            |actions: &mut Vec<UnitAction>, ability_name: String, target_id: i64| {
                actions.push(UnitAction::UseAbility {
                    ability_name,
                    target_id: target_id as u64,
                });
            },
        )
        .register_fn(
            "apply_status",
            |actions: &mut Vec<UnitAction>, status_name: String, target_id: i64, stacks: i64| {
                actions.push(UnitAction::ApplyStatus {
                    status_name,
                    target_id: target_id as u64,
                    stacks: stacks as i32,
                });
            },
        );
}

pub fn register_status_type(engine: &mut Engine) {
    engine
        .register_type_with_name::<NStatusMagic>("Status")
        .register_get("id", |status: &mut NStatusMagic| status.id() as i64)
        .register_get("status_name", |status: &mut NStatusMagic| {
            status.status_name.clone()
        });
}

pub fn register_status_actions_type(engine: &mut Engine) {
    engine
        .register_type_with_name::<Vec<StatusAction>>("StatusActions")
        .register_fn(
            "deal_damage",
            |actions: &mut Vec<StatusAction>, target_id: i64, amount: i64| {
                actions.push(StatusAction::DealDamage {
                    target_id: target_id as u64,
                    amount: amount as i32,
                });
            },
        )
        .register_fn(
            "heal_damage",
            |actions: &mut Vec<StatusAction>, target_id: i64, amount: i64| {
                actions.push(StatusAction::HealDamage {
                    target_id: target_id as u64,
                    amount: amount as i32,
                });
            },
        )
        .register_fn(
            "use_ability",
            |actions: &mut Vec<StatusAction>, ability_name: String, target_id: i64| {
                actions.push(StatusAction::UseAbility {
                    ability_name,
                    target_id: target_id as u64,
                });
            },
        )
        .register_fn(
            "modify_stacks",
            |actions: &mut Vec<StatusAction>, delta: i64| {
                actions.push(StatusAction::ModifyStacks {
                    delta: delta as i32,
                });
            },
        );
}

pub fn register_ability_type(engine: &mut Engine) {
    engine
        .register_type_with_name::<NAbilityMagic>("Ability")
        .register_get("id", |ability: &mut NAbilityMagic| ability.id() as i64)
        .register_get("ability_name", |ability: &mut NAbilityMagic| {
            ability.ability_name.clone()
        });
}

pub fn register_ability_actions_type(engine: &mut Engine) {
    engine
        .register_type_with_name::<Vec<AbilityAction>>("AbilityActions")
        .register_fn(
            "deal_damage",
            |actions: &mut Vec<AbilityAction>, target_id: i64, amount: i64| {
                actions.push(AbilityAction::DealDamage {
                    target_id: target_id as u64,
                    amount: amount as i32,
                });
            },
        )
        .register_fn(
            "heal_damage",
            |actions: &mut Vec<AbilityAction>, target_id: i64, amount: i64| {
                actions.push(AbilityAction::HealDamage {
                    target_id: target_id as u64,
                    amount: amount as i32,
                });
            },
        )
        .register_fn(
            "change_status",
            |actions: &mut Vec<AbilityAction>, status_name: String, target_id: i64, delta: i64| {
                actions.push(AbilityAction::ChangeStatus {
                    status_name,
                    target_id: target_id as u64,
                    delta: delta as i32,
                });
            },
        );
}

pub fn register_painter_functions(engine: &mut Engine) {
    engine
        .register_fn(
            "painter_circle",
            |actions: &mut Vec<String>, radius: f64| {
                actions.push(format!("circle:{}", radius));
            },
        )
        .register_fn(
            "painter_rectangle",
            |actions: &mut Vec<String>, width: f64, height: f64| {
                actions.push(format!("rect:{}:{}", width, height));
            },
        )
        .register_fn("painter_text", |actions: &mut Vec<String>, text: String| {
            actions.push(format!("text:{}", text));
        })
        .register_fn(
            "painter_translate",
            |actions: &mut Vec<String>, x: f64, y: f64| {
                actions.push(format!("translate:{}:{}", x, y));
            },
        )
        .register_fn("painter_rotate", |actions: &mut Vec<String>, angle: f64| {
            actions.push(format!("rotate:{}", angle));
        })
        .register_fn("painter_scale", |actions: &mut Vec<String>, factor: f64| {
            actions.push(format!("scale:{}", factor));
        })
        .register_fn(
            "painter_color",
            |actions: &mut Vec<String>, r: i64, g: i64, b: i64| {
                actions.push(format!("color:{}:{}:{}", r, g, b));
            },
        )
        .register_fn("painter_alpha", |actions: &mut Vec<String>, a: f64| {
            actions.push(format!("alpha:{}", a));
        })
        .register_fn(
            "painter_hollow",
            |actions: &mut Vec<String>, thickness: f64| {
                actions.push(format!("hollow:{}", thickness));
            },
        )
        .register_fn("painter_paint", |actions: &mut Vec<String>| {
            actions.push("paint".to_string());
        });
}
