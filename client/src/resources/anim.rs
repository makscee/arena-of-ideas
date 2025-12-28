use egui::NumExt;
use schema::RhaiScript;
use serde::{Deserialize, Serialize};

use super::*;
use crate::plugins::rhai::RhaiScriptAnimatorExt;

pub struct Animator {
    targets: Vec<u64>,
    duration: f32,
    timeframe: f32,
}

#[derive(Serialize, Deserialize, Clone, Debug, Default, PartialEq)]
pub struct Anim {
    script: RhaiScript<AnimAction>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum AnimAction {
    Translate { x: f32, y: f32 },
    SetTarget { target: u64 },
    AddTarget { target: u64 },
    Duration { duration: f32 },
    Timeframe { timeframe: f32 },
    Wait { duration: f32 },
    SpawnPainter { code: String },
}

impl schema::ScriptAction for AnimAction {
    fn actions_var_name() -> &'static str {
        "animator"
    }
}

impl Anim {
    pub fn new(code: String) -> Self {
        Self {
            script: RhaiScript::new(code),
        }
    }

    pub fn apply(&self, ctx: &mut ClientContext) -> NodeResult<()> {
        let actions = self
            .script
            .execute_animator(ctx)
            .map_err(|e| format!("Animation script error: {}", e))?;

        let a = &mut Animator::new();
        for action in &actions {
            action.apply(a, ctx).track()?;
        }
        Ok(())
    }
}

#[derive(BevyComponent)]
pub struct Vfx;

impl AnimAction {
    fn apply(&self, a: &mut Animator, ctx: &mut ClientContext) -> NodeResult<()> {
        match self {
            AnimAction::Translate { x, y } => {
                let pos = vec2(*x, *y);
                let mut t = ctx.battle_mut()?.duration;
                for target in a.targets.iter().copied() {
                    // Use set_var to track history in nodes directly
                    ctx.source_mut()
                        .set_var(target, VarName::position, pos.into())?;
                    t += a.timeframe;
                }
                ctx.battle_mut()?.duration = t;
            }
            AnimAction::SetTarget { target } => {
                a.targets = vec![*target];
            }
            AnimAction::AddTarget { target } => {
                a.targets.push(*target);
            }
            AnimAction::Duration { duration } => {
                a.duration = *duration;
            }
            AnimAction::Timeframe { timeframe } => {
                a.timeframe = *timeframe;
                a.duration = a.duration.at_least(a.timeframe);
            }

            AnimAction::SpawnPainter { code } => {
                let entity = ctx.world_mut()?.spawn_empty().id();
                let id = next_id();
                let material = RhaiScript::new(code.clone());
                let mut t = ctx.battle_mut()?.duration;
                let mut rep = NRepresentation::new(
                    id,
                    0,
                    ctx.get_var(VarName::position)
                        .get_vec2()
                        .unwrap_or_default(),
                    true,
                    t,
                    a.duration,
                    material,
                );
                ctx.world_mut()?.entity_mut(entity).insert(Vfx);
                for (var, value) in ctx.get_vars_layers() {
                    rep.script.scope.insert(var, value);
                }
                rep.visible_history.insert(0.0, false);
                rep.visible_history.insert(t, true);
                if a.duration > 0.0 {
                    rep.visible_history.insert(t + a.duration, false);
                }
                rep.spawn(ctx, Some(entity)).track()?;
                ctx.battle_mut()?.duration = t + a.duration;

                ctx.battle_mut()?.duration = t;
                a.targets = vec![id];
                t += a.timeframe;
                ctx.battle_mut()?.duration = t;
            }
            AnimAction::Wait { duration } => {
                ctx.battle_mut()?.duration += duration;
            }
        };
        Ok(())
    }
}

impl Animator {
    pub fn new() -> Self {
        Self {
            targets: Vec::new(),
            duration: animation_time(),
            timeframe: 0.0,
        }
    }
}

impl Default for AnimAction {
    fn default() -> Self {
        Self::Translate { x: 0.0, y: 0.0 }
    }
}

impl ToCstr for AnimAction {
    fn cstr(&self) -> Cstr {
        match self {
            AnimAction::Translate { .. } => "translate",
            AnimAction::SetTarget { .. } => "set_target",
            AnimAction::AddTarget { .. } => "add_target",
            AnimAction::Duration { .. } => "duration",
            AnimAction::Timeframe { .. } => "timeframe",
            AnimAction::Wait { .. } => "wait",
            AnimAction::SpawnPainter { .. } => "spawn_painter",
        }
        .cstr_c(PURPLE)
    }
}

impl ToCstr for Anim {
    fn cstr(&self) -> Cstr {
        "animation".cstr_c(PURPLE)
    }
}
