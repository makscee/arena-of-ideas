use legion::EntityStore;

use super::*;

#[derive(Clone, Debug, Deserialize)]
#[serde(tag = "type")]
pub enum ExpressionInt {
    Sum {
        a: Box<ExpressionInt>,
        b: Box<ExpressionInt>,
    },
    Const {
        value: i32,
    },
    Var {
        name: VarName,
    },
    Stat {
        stat: StatType,
    },
}

impl ExpressionInt {
    pub fn calculate(
        &self,
        context: &Context,
        world: &legion::World,
        resources: &Resources,
    ) -> i32 {
        match self {
            ExpressionInt::Sum { a, b } => {
                a.calculate(context, world, resources) + b.calculate(context, world, resources)
            }
            ExpressionInt::Const { value } => *value,
            ExpressionInt::Var { name } => context.vars.get_int(name),
            ExpressionInt::Stat { stat } => {
                let target = world.entry_ref(context.target).unwrap();
                match stat {
                    StatType::Hp => target.get_component::<HpComponent>().unwrap().current(),
                    StatType::Attack => target.get_component::<AttackComponent>().unwrap().value(),
                }
            }
        }
    }
}
