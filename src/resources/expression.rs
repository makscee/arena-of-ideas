use super::*;

#[derive(Serialize, Deserialize, Debug, Clone, Default, PartialEq, EnumIter, AsRefStr)]
pub enum Expression {
    #[default]
    Zero,

    Value(VarValue),
    Mod(Box<Expression>),
    Sin(Box<Expression>),
    Cos(Box<Expression>),

    Sum(Box<Expression>, Box<Expression>),
    Sub(Box<Expression>, Box<Expression>),
    Mul(Box<Expression>, Box<Expression>),
    Div(Box<Expression>, Box<Expression>),
}
