use super::*;

#[derive(Serialize, Deserialize, Clone, Debug, Default, PartialEq, AsRefStr, EnumIter)]
pub enum Effect {
    #[default]
    Noop,
    Damage,
    Kill,
    Heal,
    ChangeStatus(String),
    ClearStatus(String),
    StealStatus(String),
    ChangeAllStatuses,
    ClearAllStatuses,
    StealAllStatuses,
    UseAbility(String, i32),
    Summon(String, Option<Box<Effect>>),
    WithTarget(Expression, Box<Effect>),
    WithOwner(Expression, Box<Effect>),
    WithVar(VarName, Expression, Box<Effect>),
    List(Vec<Box<Effect>>),
    Repeat(Expression, Box<Effect>),
    If(Expression, Box<Effect>, Box<Effect>),
    Vfx(String),
    StateAddVar(VarName, Expression, Expression),
    StatusSetVar(Expression, String, VarName, Expression),
    Text(Expression),
    FullCopy,
}
