use super::*;
use crate::resources::battle::BattleAction;
use ::rhai::Engine;
use schema::{AbilityAction, StatusAction, UnitAction};

/// Trait for converting script actions to battle actions
pub trait ToBattleAction {
    fn to_battle_action(&self, ctx: &ClientContext, owner_id: u64) -> NodeResult<BattleAction>;
}

impl ToBattleAction for UnitAction {
    fn to_battle_action(&self, _ctx: &ClientContext, owner_id: u64) -> NodeResult<BattleAction> {
        match self {
            UnitAction::UseAbility {
                ability_name,
                target_id,
            } => Ok(BattleAction::use_ability {
                caster_id: owner_id,
                target_id: *target_id,
                ability_path: ability_name.clone(),
            }),
            UnitAction::ApplyStatus {
                status_name,
                target_id,
                stacks: _,
            } => Ok(BattleAction::apply_status {
                caster_id: owner_id,
                target_id: *target_id,
                status_path: status_name.clone(),
            }),
        }
    }
}

impl ToBattleAction for StatusAction {
    fn to_battle_action(&self, _ctx: &ClientContext, owner_id: u64) -> NodeResult<BattleAction> {
        match self {
            StatusAction::DealDamage { target_id, amount } => {
                Ok(BattleAction::damage(owner_id, *target_id, *amount))
            }
            StatusAction::HealDamage { target_id, amount } => {
                Ok(BattleAction::heal(owner_id, *target_id, *amount))
            }
            StatusAction::UseAbility {
                ability_name,
                target_id,
            } => Ok(BattleAction::vfx(
                vec![
                    ContextLayer::Caster(owner_id),
                    ContextLayer::Target(*target_id),
                ],
                format!("ability:{}", ability_name),
            )),
            StatusAction::ModifyStacks { delta } => Ok(BattleAction::var_set(
                owner_id,
                VarName::stax,
                VarValue::i32(*delta),
            )),
        }
    }
}

impl ToBattleAction for AbilityAction {
    fn to_battle_action(&self, _ctx: &ClientContext, caster_id: u64) -> NodeResult<BattleAction> {
        match self {
            AbilityAction::DealDamage { target_id, amount } => {
                Ok(BattleAction::damage(caster_id, *target_id, *amount))
            }
            AbilityAction::HealDamage { target_id, amount } => {
                Ok(BattleAction::heal(caster_id, *target_id, *amount))
            }
            AbilityAction::ChangeStatus {
                status_name,
                target_id,
                delta: _,
            } => Ok(BattleAction::apply_status {
                caster_id,
                target_id: *target_id,
                status_path: status_name.clone(),
            }),
        }
    }
}

pub fn register_unit_type(engine: &mut Engine) {
    engine
        .register_type_with_name::<NUnit>("Unit")
        .register_get("id", |unit: &mut NUnit| unit.id() as u64)
        .register_get("unit_name", |unit: &mut NUnit| unit.unit_name.clone());
}

pub fn register_unit_actions_type(engine: &mut Engine) {
    engine
        .register_type_with_name::<Vec<UnitAction>>("UnitActions")
        .register_fn(
            "use_ability",
            |actions: &mut Vec<UnitAction>, ability_name: String, target_id: u64| {
                actions.push(UnitAction::UseAbility {
                    ability_name,
                    target_id,
                });
            },
        )
        .register_fn(
            "apply_status",
            |actions: &mut Vec<UnitAction>, status_name: String, target_id: u64, stacks: i32| {
                actions.push(UnitAction::ApplyStatus {
                    status_name,
                    target_id,
                    stacks,
                });
            },
        );
}

pub fn register_status_type(engine: &mut Engine) {
    engine
        .register_type_with_name::<NStatusMagic>("Status")
        .register_get("id", |status: &mut NStatusMagic| status.id() as u64)
        .register_get("status_name", |status: &mut NStatusMagic| {
            status.status_name.clone()
        });
}

pub fn register_status_actions_type(engine: &mut Engine) {
    engine
        .register_type_with_name::<Vec<StatusAction>>("StatusActions")
        .register_fn(
            "deal_damage",
            |actions: &mut Vec<StatusAction>, target_id: u64, amount: i32| {
                actions.push(StatusAction::DealDamage {
                    target_id: target_id as u64,
                    amount: amount as i32,
                });
            },
        )
        .register_fn(
            "heal_damage",
            |actions: &mut Vec<StatusAction>, target_id: u64, amount: i32| {
                actions.push(StatusAction::HealDamage {
                    target_id: target_id as u64,
                    amount: amount as i32,
                });
            },
        )
        .register_fn(
            "use_ability",
            |actions: &mut Vec<StatusAction>, ability_name: String, target_id: u64| {
                actions.push(StatusAction::UseAbility {
                    ability_name,
                    target_id: target_id as u64,
                });
            },
        )
        .register_fn(
            "modify_stacks",
            |actions: &mut Vec<StatusAction>, delta: i32| {
                actions.push(StatusAction::ModifyStacks {
                    delta: delta as i32,
                });
            },
        );
}

pub fn register_ability_type(engine: &mut Engine) {
    engine
        .register_type_with_name::<NAbilityMagic>("Ability")
        .register_get("id", |ability: &mut NAbilityMagic| ability.id() as u64)
        .register_get("ability_name", |ability: &mut NAbilityMagic| {
            ability.ability_name.clone()
        });
}

pub fn register_ability_actions_type(engine: &mut Engine) {
    engine
        .register_type_with_name::<Vec<AbilityAction>>("AbilityActions")
        .register_fn(
            "deal_damage",
            |actions: &mut Vec<AbilityAction>, target_id: u64, amount: i32| {
                actions.push(AbilityAction::DealDamage {
                    target_id: target_id as u64,
                    amount: amount as i32,
                });
            },
        )
        .register_fn(
            "heal_damage",
            |actions: &mut Vec<AbilityAction>, target_id: u64, amount: i32| {
                actions.push(AbilityAction::HealDamage {
                    target_id: target_id as u64,
                    amount: amount as i32,
                });
            },
        )
        .register_fn(
            "change_status",
            |actions: &mut Vec<AbilityAction>, status_name: String, target_id: u64, delta: i32| {
                actions.push(AbilityAction::ChangeStatus {
                    status_name,
                    target_id: target_id as u64,
                    delta: delta as i32,
                });
            },
        );
}

pub fn register_painter_type(engine: &mut ::rhai::Engine) {
    engine
        .register_type_with_name::<Vec<PainterAction>>("PainterActions")
        .register_fn("circle", |actions: &mut Vec<PainterAction>, radius: f32| {
            actions.push(PainterAction::Circle { radius });
        })
        .register_fn(
            "rectangle",
            |actions: &mut Vec<PainterAction>, width: f32, height: f32| {
                actions.push(PainterAction::Rectangle { width, height });
            },
        )
        .register_fn(
            "curve",
            |actions: &mut Vec<PainterAction>, thickness: f32, curvature: f32| {
                actions.push(PainterAction::Curve {
                    thickness,
                    curvature,
                });
            },
        )
        .register_fn("text", |actions: &mut Vec<PainterAction>, text: String| {
            actions.push(PainterAction::Text { text });
        })
        .register_fn("hollow", |actions: &mut Vec<PainterAction>, width: f32| {
            actions.push(PainterAction::Hollow { width });
        })
        .register_fn("solid", |actions: &mut Vec<PainterAction>| {
            actions.push(PainterAction::Solid);
        })
        .register_fn(
            "translate",
            |actions: &mut Vec<PainterAction>, x: f32, y: f32| {
                actions.push(PainterAction::Translate { x, y });
            },
        )
        .register_fn("rotate", |actions: &mut Vec<PainterAction>, angle: f32| {
            actions.push(PainterAction::Rotate { angle });
        })
        .register_fn(
            "scale_mesh",
            |actions: &mut Vec<PainterAction>, scale: f32| {
                actions.push(PainterAction::ScaleMesh { scale });
            },
        )
        .register_fn(
            "scale_rect",
            |actions: &mut Vec<PainterAction>, scale: f32| {
                actions.push(PainterAction::ScaleRect { scale });
            },
        )
        .register_fn(
            "color",
            |actions: &mut Vec<PainterAction>, r: i32, g: i32, b: i32, a: i32| {
                actions.push(PainterAction::Color {
                    r: (r as u8).clamp(0, 255),
                    g: (g as u8).clamp(0, 255),
                    b: (b as u8).clamp(0, 255),
                    a: (a as u8).clamp(0, 255),
                });
            },
        )
        .register_fn("alpha", |actions: &mut Vec<PainterAction>, alpha: f32| {
            actions.push(PainterAction::Alpha { alpha });
        })
        .register_fn(
            "feathering",
            |actions: &mut Vec<PainterAction>, amount: f32| {
                actions.push(PainterAction::Feathering { amount });
            },
        )
        .register_fn("paint", |actions: &mut Vec<PainterAction>| {
            actions.push(PainterAction::Paint);
        })
        .register_fn("exit", |actions: &mut Vec<PainterAction>| {
            actions.push(PainterAction::Exit);
        });
}
