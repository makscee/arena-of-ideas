use rhai::{AST, Dynamic, Engine, Map, Scope};
use std::cell::RefCell;
use std::rc::Rc;

/// A collected action from a Rhai ability script execution.
#[derive(Debug, Clone)]
pub enum ScriptAction {
    DealDamage {
        target_id: u64,
        amount: i32,
    },
    HealDamage {
        target_id: u64,
        amount: i32,
    },
    StealStat {
        target_id: u64,
        stat: String,
        amount: i32,
    },
    AddShield {
        target_id: u64,
        amount: i32,
    },
    ChangeStat {
        target_id: u64,
        stat: String,
        delta: i32,
    },
}

/// Thread-local action collector for Rhai scripts.
thread_local! {
    static SCRIPT_ACTIONS: RefCell<Vec<ScriptAction>> = RefCell::new(Vec::new());
}

fn push_action(action: ScriptAction) {
    SCRIPT_ACTIONS.with(|actions| actions.borrow_mut().push(action));
}

fn take_actions() -> Vec<ScriptAction> {
    SCRIPT_ACTIONS.with(|actions| actions.borrow_mut().drain(..).collect())
}

/// Unit data exposed to Rhai scripts.
#[derive(Debug, Clone)]
pub struct ScriptUnit {
    pub id: i64,
    pub hp: i64,
    pub pwr: i64,
    pub dmg: i64,
}

impl ScriptUnit {
    pub fn to_dynamic(&self) -> Dynamic {
        let mut map = Map::new();
        map.insert("id".into(), Dynamic::from(self.id));
        map.insert("hp".into(), Dynamic::from(self.hp));
        map.insert("pwr".into(), Dynamic::from(self.pwr));
        map.insert("dmg".into(), Dynamic::from(self.dmg));
        Dynamic::from(map)
    }
}

/// Create a configured Rhai engine with all ability action functions registered.
/// Engine can be reused across multiple script executions.
pub fn create_engine() -> Engine {
    let mut engine = Engine::new();

    engine.set_max_operations(10_000);
    engine.set_max_expr_depths(32, 32);

    engine.register_fn("deal_damage", |target_id: i64, amount: i64| {
        push_action(ScriptAction::DealDamage {
            target_id: target_id as u64,
            amount: amount as i32,
        });
    });

    engine.register_fn("heal_damage", |target_id: i64, amount: i64| {
        push_action(ScriptAction::HealDamage {
            target_id: target_id as u64,
            amount: amount as i32,
        });
    });

    engine.register_fn("steal_stat", |target_id: i64, stat: String, amount: i64| {
        push_action(ScriptAction::StealStat {
            target_id: target_id as u64,
            stat,
            amount: amount as i32,
        });
    });

    engine.register_fn("add_shield", |target_id: i64, amount: i64| {
        push_action(ScriptAction::AddShield {
            target_id: target_id as u64,
            amount: amount as i32,
        });
    });

    engine.register_fn("change_stat", |target_id: i64, stat: String, delta: i64| {
        push_action(ScriptAction::ChangeStat {
            target_id: target_id as u64,
            stat,
            delta: delta as i32,
        });
    });

    engine
}

/// Compile a Rhai script, returning an AST or error message.
pub fn compile_script(engine: &Engine, script: &str) -> Result<AST, String> {
    engine.compile(script).map_err(|e| e.to_string())
}

/// Execute an ability script with the given context.
/// Returns the list of actions the script produced.
pub fn execute_ability_script(
    engine: &Engine,
    ast: &AST,
    x: i32,
    level: i32,
    owner: &ScriptUnit,
    target: &ScriptUnit,
    _source_unit_id: u64,
    _ability_name: &str,
) -> Result<Vec<ScriptAction>, String> {
    // Clear any leftover actions
    take_actions();

    let mut scope = Scope::new();
    scope.push("X", x as i64);
    scope.push("level", level as i64);
    scope.push("owner", owner.to_dynamic());
    scope.push("target", target.to_dynamic());

    engine
        .run_ast_with_scope(&mut scope, ast)
        .map_err(|e| e.to_string())?;

    Ok(take_actions())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn engine_creates_successfully() {
        let engine = create_engine();
        assert!(compile_script(&engine, "let x = 1 + 2;").is_ok());
    }

    #[test]
    fn invalid_script_returns_error() {
        let engine = create_engine();
        assert!(compile_script(&engine, "this is not valid rhai {{{}}}").is_err());
    }

    #[test]
    fn script_deal_damage() {
        let engine = create_engine();
        let ast = compile_script(&engine, "deal_damage(target[\"id\"], X * level);").unwrap();

        let owner = ScriptUnit {
            id: 1,
            hp: 10,
            pwr: 3,
            dmg: 0,
        };
        let target = ScriptUnit {
            id: 2,
            hp: 8,
            pwr: 2,
            dmg: 0,
        };

        let actions =
            execute_ability_script(&engine, &ast, 3, 2, &owner, &target, 1, "Strike").unwrap();

        assert_eq!(actions.len(), 1);
        assert!(
            matches!(
                &actions[0],
                ScriptAction::DealDamage {
                    target_id: 2,
                    amount: 6
                }
            ),
            "Expected DealDamage(2, 6), got {:?}",
            actions[0]
        );
    }

    #[test]
    fn script_heal_damage() {
        let engine = create_engine();
        let ast = compile_script(&engine, "heal_damage(owner[\"id\"], X * level);").unwrap();

        let owner = ScriptUnit {
            id: 1,
            hp: 5,
            pwr: 4,
            dmg: 3,
        };
        let target = ScriptUnit {
            id: 2,
            hp: 8,
            pwr: 2,
            dmg: 0,
        };

        let actions =
            execute_ability_script(&engine, &ast, 4, 1, &owner, &target, 1, "Heal").unwrap();

        assert_eq!(actions.len(), 1);
        assert!(
            matches!(
                &actions[0],
                ScriptAction::HealDamage {
                    target_id: 1,
                    amount: 4
                }
            ),
            "Expected HealDamage(1, 4), got {:?}",
            actions[0]
        );
    }

    #[test]
    fn script_multiple_actions() {
        let engine = create_engine();
        let script = r#"
            deal_damage(target["id"], X);
            change_stat(target["id"], "pwr", -level);
        "#;
        let ast = compile_script(&engine, script).unwrap();

        let owner = ScriptUnit {
            id: 1,
            hp: 10,
            pwr: 5,
            dmg: 0,
        };
        let target = ScriptUnit {
            id: 2,
            hp: 8,
            pwr: 3,
            dmg: 0,
        };

        let actions =
            execute_ability_script(&engine, &ast, 5, 2, &owner, &target, 1, "Combo").unwrap();
        assert_eq!(actions.len(), 2);
    }

    #[test]
    fn script_reads_owner_stats() {
        let engine = create_engine();
        let script = r#"
            let hp = owner["hp"];
            if hp < 5 {
                heal_damage(owner["id"], X * 2);
            } else {
                deal_damage(target["id"], X);
            }
        "#;
        let ast = compile_script(&engine, script).unwrap();

        // Owner low HP — should heal
        let owner = ScriptUnit {
            id: 1,
            hp: 3,
            pwr: 4,
            dmg: 0,
        };
        let target = ScriptUnit {
            id: 2,
            hp: 8,
            pwr: 2,
            dmg: 0,
        };
        let actions =
            execute_ability_script(&engine, &ast, 4, 1, &owner, &target, 1, "Smart").unwrap();
        assert_eq!(actions.len(), 1);
        assert!(
            matches!(&actions[0], ScriptAction::HealDamage { amount: 8, .. }),
            "Low HP should heal, got {:?}",
            actions[0]
        );

        // Owner high HP — should damage
        let owner_healthy = ScriptUnit {
            id: 1,
            hp: 10,
            pwr: 4,
            dmg: 0,
        };
        let actions =
            execute_ability_script(&engine, &ast, 4, 1, &owner_healthy, &target, 1, "Smart")
                .unwrap();
        assert_eq!(actions.len(), 1);
        assert!(
            matches!(&actions[0], ScriptAction::DealDamage { amount: 4, .. }),
            "High HP should damage, got {:?}",
            actions[0]
        );
    }

    #[test]
    fn script_infinite_loop_times_out() {
        let engine = create_engine();
        let ast = compile_script(&engine, "loop {}").unwrap();

        let owner = ScriptUnit {
            id: 1,
            hp: 10,
            pwr: 3,
            dmg: 0,
        };
        let target = ScriptUnit {
            id: 2,
            hp: 8,
            pwr: 2,
            dmg: 0,
        };

        let result = execute_ability_script(&engine, &ast, 3, 1, &owner, &target, 1, "Bad");
        assert!(result.is_err());
    }

    #[test]
    fn engine_reuse_across_scripts() {
        // Verify the engine can be reused without action leakage
        let engine = create_engine();
        let ast1 = compile_script(&engine, "deal_damage(1, 5);").unwrap();
        let ast2 = compile_script(&engine, "heal_damage(2, 3);").unwrap();

        let owner = ScriptUnit {
            id: 1,
            hp: 10,
            pwr: 5,
            dmg: 0,
        };
        let target = ScriptUnit {
            id: 2,
            hp: 10,
            pwr: 5,
            dmg: 0,
        };

        let a1 = execute_ability_script(&engine, &ast1, 5, 1, &owner, &target, 1, "A").unwrap();
        assert_eq!(a1.len(), 1);
        assert!(matches!(&a1[0], ScriptAction::DealDamage { .. }));

        let a2 = execute_ability_script(&engine, &ast2, 5, 1, &owner, &target, 1, "B").unwrap();
        assert_eq!(a2.len(), 1);
        assert!(matches!(&a2[0], ScriptAction::HealDamage { .. }));
    }
}
