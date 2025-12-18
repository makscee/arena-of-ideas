use super::*;
use crate::resources::battle::BattleAction;
use ::rhai::{Array, Engine};
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
        .register_get("id".register_completer(), |unit: &mut NUnit| {
            unit.id() as u64
        })
        .register_get("unit_name".register_completer(), |unit: &mut NUnit| {
            unit.unit_name.clone()
        })
        .register_get("dmg".register_completer(), |unit: &mut NUnit| {
            unit.state.get().unwrap().dmg
        })
        .register_get("hp".register_completer(), |unit: &mut NUnit| {
            unit.behavior.get().unwrap().stats.get().unwrap().hp
        })
        .register_get("pwr".register_completer(), |unit: &mut NUnit| {
            unit.behavior.get().unwrap().stats.get().unwrap().pwr
        });
}

pub fn register_unit_actions_type(engine: &mut Engine) {
    engine
        .register_type_with_name::<Vec<UnitAction>>("UnitActions")
        .register_fn(
            "use_ability".register_completer(),
            |actions: &mut Vec<UnitAction>, ability_name: String, target_id: u64| {
                actions.push(UnitAction::UseAbility {
                    ability_name,
                    target_id,
                });
            },
        )
        .register_fn(
            "apply_status".register_completer(),
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
        .register_get("id".register_completer(), |status: &mut NStatusMagic| {
            status.id() as u64
        })
        .register_get(
            "status_name".register_completer(),
            |status: &mut NStatusMagic| status.status_name.clone(),
        );
}

pub fn register_status_actions_type(engine: &mut Engine) {
    engine
        .register_type_with_name::<Vec<StatusAction>>("StatusActions")
        .register_fn(
            "deal_damage".register_completer(),
            |actions: &mut Vec<StatusAction>, target_id: u64, amount: i32| {
                actions.push(StatusAction::DealDamage {
                    target_id: target_id as u64,
                    amount: amount as i32,
                });
            },
        )
        .register_fn(
            "heal_damage".register_completer(),
            |actions: &mut Vec<StatusAction>, target_id: u64, amount: i32| {
                actions.push(StatusAction::HealDamage {
                    target_id: target_id as u64,
                    amount: amount as i32,
                });
            },
        )
        .register_fn(
            "use_ability".register_completer(),
            |actions: &mut Vec<StatusAction>, ability_name: String, target_id: u64| {
                actions.push(StatusAction::UseAbility {
                    ability_name,
                    target_id: target_id as u64,
                });
            },
        )
        .register_fn(
            "modify_stacks".register_completer(),
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
        .register_get("id".register_completer(), |ability: &mut NAbilityMagic| {
            ability.id() as u64
        })
        .register_get(
            "ability_name".register_completer(),
            |ability: &mut NAbilityMagic| ability.ability_name.clone(),
        );
}

pub fn register_ability_actions_type(engine: &mut Engine) {
    engine
        .register_type_with_name::<Vec<AbilityAction>>("AbilityActions")
        .register_fn(
            "deal_damage".register_completer(),
            |actions: &mut Vec<AbilityAction>, target_id: u64, amount: i32| {
                actions.push(AbilityAction::DealDamage {
                    target_id: target_id as u64,
                    amount: amount as i32,
                });
            },
        )
        .register_fn(
            "heal_damage".register_completer(),
            |actions: &mut Vec<AbilityAction>, target_id: u64, amount: i32| {
                actions.push(AbilityAction::HealDamage {
                    target_id: target_id as u64,
                    amount: amount as i32,
                });
            },
        )
        .register_fn(
            "change_status".register_completer(),
            |actions: &mut Vec<AbilityAction>, status_name: String, target_id: u64, delta: i32| {
                actions.push(AbilityAction::ChangeStatus {
                    status_name,
                    target_id: target_id as u64,
                    delta: delta as i32,
                });
            },
        );
}

pub fn register_painter_type(engine: &mut Engine) {
    engine
        .register_type_with_name::<Vec<PainterAction>>("PainterActions")
        .register_fn(
            "circle".register_completer(),
            |actions: &mut Vec<PainterAction>, radius: f32| {
                actions.push(PainterAction::Circle { radius });
            },
        )
        .register_fn(
            "rectangle".register_completer(),
            |actions: &mut Vec<PainterAction>, width: f32, height: f32| {
                actions.push(PainterAction::Rectangle { width, height });
            },
        )
        .register_fn(
            "curve".register_completer(),
            |actions: &mut Vec<PainterAction>, thickness: f32, curvature: f32| {
                actions.push(PainterAction::Curve {
                    thickness,
                    curvature,
                });
            },
        )
        .register_fn(
            "text".register_completer(),
            |actions: &mut Vec<PainterAction>, text: String| {
                actions.push(PainterAction::Text { text });
            },
        )
        .register_fn(
            "hollow".register_completer(),
            |actions: &mut Vec<PainterAction>, width: f32| {
                actions.push(PainterAction::Hollow { width });
            },
        )
        .register_fn(
            "solid".register_completer(),
            |actions: &mut Vec<PainterAction>| {
                actions.push(PainterAction::Solid);
            },
        )
        .register_fn(
            "translate".register_completer(),
            |actions: &mut Vec<PainterAction>, v: Array| {
                actions.push(PainterAction::Translate {
                    x: v[0].as_float().unwrap_or_default(),
                    y: v[1].as_float().unwrap_or_default(),
                });
            },
        )
        .register_fn(
            "rotate".register_completer(),
            |actions: &mut Vec<PainterAction>, angle: f32| {
                actions.push(PainterAction::Rotate { angle });
            },
        )
        .register_fn(
            "scale_mesh".register_completer(),
            |actions: &mut Vec<PainterAction>, scale: f32| {
                actions.push(PainterAction::ScaleMesh { scale });
            },
        )
        .register_fn(
            "scale_rect".register_completer(),
            |actions: &mut Vec<PainterAction>, scale: f32| {
                actions.push(PainterAction::ScaleRect { scale });
            },
        )
        .register_fn(
            "color".register_completer(),
            |actions: &mut Vec<PainterAction>, r: i32, g: i32, b: i32, a: i32| {
                let color = Color32::from_rgba_premultiplied(
                    (r as u8).clamp(0, 255),
                    (g as u8).clamp(0, 255),
                    (b as u8).clamp(0, 255),
                    (a as u8).clamp(0, 255),
                );
                actions.push(PainterAction::Color { color });
            },
        )
        .register_fn(
            "color".register_completer(),
            |actions: &mut Vec<PainterAction>, hex: String| {
                match Color32::from_hex(&hex) {
                    Ok(color) => actions.push(PainterAction::Color { color }),
                    Err(e) => error!("Failed to parse color {e:?}"),
                };
            },
        )
        .register_fn(
            "color".register_completer(),
            |actions: &mut Vec<PainterAction>, color: Color32| {
                actions.push(PainterAction::Color { color });
            },
        )
        .register_fn(
            "alpha".register_completer(),
            |actions: &mut Vec<PainterAction>, alpha: f32| {
                actions.push(PainterAction::Alpha { alpha });
            },
        )
        .register_fn(
            "feathering".register_completer(),
            |actions: &mut Vec<PainterAction>, amount: f32| {
                actions.push(PainterAction::Feathering { amount });
            },
        )
        .register_fn(
            "paint".register_completer(),
            |actions: &mut Vec<PainterAction>| {
                actions.push(PainterAction::Paint);
            },
        )
        .register_fn(
            "exit".register_completer(),
            |actions: &mut Vec<PainterAction>| {
                actions.push(PainterAction::Exit);
            },
        );
}

pub fn register_animator_type(engine: &mut Engine) {
    engine
        .register_type_with_name::<Vec<AnimAction>>("AnimatorActions")
        .register_fn(
            "translate".register_completer(),
            |actions: &mut Vec<AnimAction>, v: Array| {
                let Some((x, y)) = array_to_vec2(&v) else {
                    return;
                };
                actions.push(AnimAction::Translate { x, y });
            },
        )
        .register_fn(
            "set_target".register_completer(),
            |actions: &mut Vec<AnimAction>, target: u64| {
                actions.push(AnimAction::SetTarget { target });
            },
        )
        .register_fn(
            "add_target".register_completer(),
            |actions: &mut Vec<AnimAction>, target: u64| {
                actions.push(AnimAction::AddTarget { target });
            },
        )
        .register_fn(
            "duration".register_completer(),
            |actions: &mut Vec<AnimAction>, duration: f32| {
                actions.push(AnimAction::Duration { duration });
            },
        )
        .register_fn(
            "timeframe".register_completer(),
            |actions: &mut Vec<AnimAction>, timeframe: f32| {
                actions.push(AnimAction::Timeframe { timeframe });
            },
        )
        .register_fn(
            "wait".register_completer(),
            |actions: &mut Vec<AnimAction>, duration: f32| {
                actions.push(AnimAction::Wait { duration });
            },
        )
        .register_fn(
            "spawn_painter".register_completer(),
            |actions: &mut Vec<AnimAction>, code: String| {
                actions.push(AnimAction::SpawnPainter { code });
            },
        );
}
