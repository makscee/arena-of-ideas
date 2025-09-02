use super::*;

/// Trait for types that can provide their recursive fields for display.
/// This trait focuses purely on defining which fields should be shown recursively.
pub trait RecursiveFields {
    /// Returns an iterator of field names and their recursive values
    fn recursive_fields(&self) -> Vec<RecursiveField<'_>>;
}

/// Represents a field that can be displayed recursively
pub struct RecursiveField<'a> {
    pub name: String,
    pub value: RecursiveValue<'a>,
}

pub enum RecursiveValue<'a> {
    Expr(&'a Expression),
    Action(&'a Action),
    PainterAction(&'a PainterAction),
    Var(&'a VarName),
    VarValue(&'a VarValue),
    HexColor(&'a HexColor),
    String(&'a String),
    I32(&'a i32),
    F32(&'a f32),
    Bool(&'a bool),
    Vec2(&'a Vec2),
}

/// Main trait for recursive showing - this is what gets called from the UI
pub trait SFnShowRecursive {
    fn show_recursive(&self, name: &str, context: &Context, ui: &mut Ui);
}

/// Implementation for types that provide recursive fields
impl<T> SFnShowRecursive for T
where
    T: RecursiveFields + SFnShow,
{
    fn show_recursive(&self, name: &str, context: &Context, ui: &mut Ui) {
        ui.group(|ui| {
            ui.vertical(|ui| {
                if !name.is_empty() {
                    format!("[s [tw {}:]]", name).cstr().label(ui);
                }
                self.show(context, ui);
            });
        });

        let fields = self.recursive_fields();
        ui.vertical(|ui| {
            for field in fields {
                ui.horizontal(|ui| {
                    show_recursive_field(field, context, ui);
                });
            }
        });
    }
}

/// Macro to call any function on all RecursiveValue variants
/// Examples:
/// - `call_on_recursive_value!(field.value, show_recursive, context, ui)` - shows all values
/// - `call_on_recursive_value!(field.value, show_mut, context, ui)` - mutable editing
/// - `call_on_recursive_value!(field.value, validate, context, ui)` - validation
#[macro_export]
macro_rules! call_on_recursive_value {
    ($value:expr, $name:expr, $func:ident, $context:expr, $ui:expr) => {
        match $value {
            RecursiveValue::Expr(v) => v.$func($name, $context, $ui),
            RecursiveValue::Action(v) => v.$func($name, $context, $ui),
            RecursiveValue::PainterAction(v) => v.$func($name, $context, $ui),
            RecursiveValue::Var(v) => v.$func($name, $context, $ui),
            RecursiveValue::VarValue(v) => v.$func($name, $context, $ui),
            RecursiveValue::HexColor(v) => v.$func($name, $context, $ui),
            RecursiveValue::String(v) => v.$func($name, $context, $ui),
            RecursiveValue::I32(v) => v.$func($name, $context, $ui),
            RecursiveValue::F32(v) => v.$func($name, $context, $ui),
            RecursiveValue::Bool(v) => v.$func($name, $context, $ui),
            RecursiveValue::Vec2(v) => v.$func($name, $context, $ui),
        }
    };
}

fn show_recursive_field(field: RecursiveField<'_>, context: &Context, ui: &mut Ui) {
    crate::call_on_recursive_value!(field.value, &field.name, show_recursive, context, ui);
}

// Helper function to create RecursiveField
impl<'a> RecursiveField<'a> {
    fn named(name: &str, value: RecursiveValue<'a>) -> Self {
        Self {
            name: name.to_string(),
            value,
        }
    }

    fn indexed(index: usize, value: RecursiveValue<'a>) -> Self {
        Self {
            name: index.to_string(),
            value,
        }
    }
}

// Implementations for primitive types (leaf nodes)
impl RecursiveFields for i32 {
    fn recursive_fields(&self) -> Vec<RecursiveField<'_>> {
        vec![]
    }
}

impl RecursiveFields for f32 {
    fn recursive_fields(&self) -> Vec<RecursiveField<'_>> {
        vec![]
    }
}

impl RecursiveFields for String {
    fn recursive_fields(&self) -> Vec<RecursiveField<'_>> {
        vec![]
    }
}

impl RecursiveFields for bool {
    fn recursive_fields(&self) -> Vec<RecursiveField<'_>> {
        vec![]
    }
}

impl RecursiveFields for VarName {
    fn recursive_fields(&self) -> Vec<RecursiveField<'_>> {
        vec![]
    }
}

impl RecursiveFields for VarValue {
    fn recursive_fields(&self) -> Vec<RecursiveField<'_>> {
        vec![]
    }
}

impl RecursiveFields for HexColor {
    fn recursive_fields(&self) -> Vec<RecursiveField<'_>> {
        vec![]
    }
}

impl RecursiveFields for Vec2 {
    fn recursive_fields(&self) -> Vec<RecursiveField<'_>> {
        vec![]
    }
}

// Container type implementations for Expression
impl RecursiveFields for Box<Expression> {
    fn recursive_fields(&self) -> Vec<RecursiveField<'_>> {
        self.as_ref().recursive_fields()
    }
}

impl RecursiveFields for Vec<Expression> {
    fn recursive_fields(&self) -> Vec<RecursiveField<'_>> {
        self.iter()
            .enumerate()
            .map(|(i, item)| RecursiveField::indexed(i, RecursiveValue::Expr(item)))
            .collect()
    }
}

impl RecursiveFields for Vec<Action> {
    fn recursive_fields(&self) -> Vec<RecursiveField<'_>> {
        self.iter()
            .enumerate()
            .map(|(i, item)| RecursiveField::indexed(i, RecursiveValue::Action(item)))
            .collect()
    }
}

impl RecursiveFields for Vec<Box<Action>> {
    fn recursive_fields(&self) -> Vec<RecursiveField<'_>> {
        self.iter()
            .enumerate()
            .map(|(i, item)| RecursiveField::indexed(i, RecursiveValue::Action(item.as_ref())))
            .collect()
    }
}

impl RecursiveFields for Vec<Box<PainterAction>> {
    fn recursive_fields(&self) -> Vec<RecursiveField<'_>> {
        self.iter()
            .enumerate()
            .map(|(i, item)| {
                RecursiveField::indexed(i, RecursiveValue::PainterAction(item.as_ref()))
            })
            .collect()
    }
}

impl RecursiveFields for Option<Expression> {
    fn recursive_fields(&self) -> Vec<RecursiveField<'_>> {
        match self {
            Some(expr) => vec![RecursiveField::named("value", RecursiveValue::Expr(expr))],
            None => vec![], // None is a leaf node
        }
    }
}

// Expression implementation
impl RecursiveFields for Expression {
    fn recursive_fields(&self) -> Vec<RecursiveField<'_>> {
        match self {
            // Leaf expressions (no nested expressions)
            Expression::one
            | Expression::zero
            | Expression::gt
            | Expression::owner
            | Expression::target
            | Expression::unit_size
            | Expression::pi
            | Expression::pi2
            | Expression::all_units
            | Expression::all_enemy_units
            | Expression::all_ally_units
            | Expression::all_other_ally_units
            | Expression::adjacent_ally_units
            | Expression::adjacent_back
            | Expression::adjacent_front => vec![],

            // Simple value expressions - now showing their inner values
            Expression::var(var) => vec![RecursiveField::named("var", RecursiveValue::Var(var))],
            Expression::var_sum(var) => {
                vec![RecursiveField::named("var", RecursiveValue::Var(var))]
            }
            Expression::value(val) => vec![RecursiveField::named(
                "value",
                RecursiveValue::VarValue(val),
            )],
            Expression::string(s) => {
                vec![RecursiveField::named("string", RecursiveValue::String(s))]
            }
            Expression::f32(f) => vec![RecursiveField::named("f32", RecursiveValue::F32(f))],
            Expression::f32_slider(f) => {
                vec![RecursiveField::named("f32_slider", RecursiveValue::F32(f))]
            }
            Expression::i32(i) => vec![RecursiveField::named("i32", RecursiveValue::I32(i))],
            Expression::bool(b) => vec![RecursiveField::named("bool", RecursiveValue::Bool(b))],
            Expression::color(c) => {
                vec![RecursiveField::named("color", RecursiveValue::HexColor(c))]
            }
            Expression::lua_i32(i) => {
                vec![RecursiveField::named("lua_i32", RecursiveValue::String(i))]
            }
            Expression::lua_f32(f) => {
                vec![RecursiveField::named("lua_f32", RecursiveValue::String(f))]
            }

            // vec2 with two f32 values
            Expression::vec2(x, y) => vec![
                RecursiveField::named("x", RecursiveValue::F32(x)),
                RecursiveField::named("y", RecursiveValue::F32(y)),
            ],

            // Expressions with one nested expression
            // Unary expressions
            Expression::sin(expr)
            | Expression::cos(expr)
            | Expression::even(expr)
            | Expression::abs(expr)
            | Expression::floor(expr)
            | Expression::ceil(expr)
            | Expression::fract(expr)
            | Expression::sqr(expr)
            | Expression::unit_vec(expr)
            | Expression::rand(expr)
            | Expression::random_unit(expr)
            | Expression::neg(expr)
            | Expression::to_f32(expr) => {
                vec![RecursiveField::named(
                    "expr",
                    RecursiveValue::Expr(expr.as_ref()),
                )]
            }

            // Special case: state_var has both an expression and a var
            Expression::state_var(expr, var) => vec![
                RecursiveField::named("expr", RecursiveValue::Expr(expr.as_ref())),
                RecursiveField::named("var", RecursiveValue::Var(var)),
            ],

            // Binary expressions
            Expression::vec2_ee(a, b) => vec![
                RecursiveField::named("x", RecursiveValue::Expr(a.as_ref())),
                RecursiveField::named("y", RecursiveValue::Expr(b.as_ref())),
            ],
            Expression::str_macro(template, value) => vec![
                RecursiveField::named("template", RecursiveValue::Expr(template.as_ref())),
                RecursiveField::named("value", RecursiveValue::Expr(value.as_ref())),
            ],
            Expression::sum(left, right)
            | Expression::sub(left, right)
            | Expression::mul(left, right)
            | Expression::div(left, right)
            | Expression::max(left, right)
            | Expression::min(left, right)
            | Expression::r#mod(left, right)
            | Expression::and(left, right)
            | Expression::or(left, right)
            | Expression::equals(left, right)
            | Expression::greater_then(left, right)
            | Expression::less_then(left, right) => vec![
                RecursiveField::named("left", RecursiveValue::Expr(left.as_ref())),
                RecursiveField::named("right", RecursiveValue::Expr(right.as_ref())),
            ],
            Expression::fallback(primary, fallback) => vec![
                RecursiveField::named("primary", RecursiveValue::Expr(primary.as_ref())),
                RecursiveField::named("fallback", RecursiveValue::Expr(fallback.as_ref())),
            ],

            // Ternary expressions
            Expression::r#if(condition, then_expr, else_expr) => vec![
                RecursiveField::named("condition", RecursiveValue::Expr(condition.as_ref())),
                RecursiveField::named("then", RecursiveValue::Expr(then_expr.as_ref())),
                RecursiveField::named("else", RecursiveValue::Expr(else_expr.as_ref())),
            ],
            Expression::oklch(l, c, h) => vec![
                RecursiveField::named("lightness", RecursiveValue::Expr(l.as_ref())),
                RecursiveField::named("chroma", RecursiveValue::Expr(c.as_ref())),
                RecursiveField::named("hue", RecursiveValue::Expr(h.as_ref())),
            ],
        }
    }
}

// Action implementation
impl RecursiveFields for Action {
    fn recursive_fields(&self) -> Vec<RecursiveField<'_>> {
        match self {
            // Leaf actions
            Action::noop
            | Action::deal_damage
            | Action::heal_damage
            | Action::use_ability
            | Action::apply_status => vec![],

            // Actions with one expression
            Action::debug(expr)
            | Action::set_value(expr)
            | Action::add_value(expr)
            | Action::subtract_value(expr)
            | Action::add_target(expr) => {
                vec![RecursiveField::named(
                    "expr",
                    RecursiveValue::Expr(expr.as_ref()),
                )]
            }

            // Complex action: repeat
            Action::repeat(count_expr, actions) => {
                let mut fields = vec![RecursiveField::named(
                    "count",
                    RecursiveValue::Expr(count_expr.as_ref()),
                )];
                for (i, action) in actions.iter().enumerate() {
                    fields.push(RecursiveField::indexed(
                        i,
                        RecursiveValue::Action(action.as_ref()),
                    ));
                }
                fields
            }
        }
    }
}

// PainterAction implementation
impl RecursiveFields for PainterAction {
    fn recursive_fields(&self) -> Vec<RecursiveField<'_>> {
        match self {
            // Leaf action
            PainterAction::paint => vec![],

            // Actions with one expression
            PainterAction::circle(radius) => {
                vec![RecursiveField::named(
                    "radius",
                    RecursiveValue::Expr(radius.as_ref()),
                )]
            }
            PainterAction::rectangle(size) => {
                vec![RecursiveField::named(
                    "size",
                    RecursiveValue::Expr(size.as_ref()),
                )]
            }
            PainterAction::text(content) => {
                vec![RecursiveField::named(
                    "content",
                    RecursiveValue::Expr(content.as_ref()),
                )]
            }
            PainterAction::hollow(thickness) => {
                vec![RecursiveField::named(
                    "thickness",
                    RecursiveValue::Expr(thickness.as_ref()),
                )]
            }
            PainterAction::translate(offset) => {
                vec![RecursiveField::named(
                    "offset",
                    RecursiveValue::Expr(offset.as_ref()),
                )]
            }
            PainterAction::rotate(angle) => {
                vec![RecursiveField::named(
                    "angle",
                    RecursiveValue::Expr(angle.as_ref()),
                )]
            }
            PainterAction::scale_mesh(factor) => {
                vec![RecursiveField::named(
                    "factor",
                    RecursiveValue::Expr(factor.as_ref()),
                )]
            }
            PainterAction::scale_rect(factor) => {
                vec![RecursiveField::named(
                    "factor",
                    RecursiveValue::Expr(factor.as_ref()),
                )]
            }
            PainterAction::color(color_expr) => {
                vec![RecursiveField::named(
                    "color",
                    RecursiveValue::Expr(color_expr.as_ref()),
                )]
            }
            PainterAction::alpha(alpha_expr) => {
                vec![RecursiveField::named(
                    "alpha",
                    RecursiveValue::Expr(alpha_expr.as_ref()),
                )]
            }
            PainterAction::feathering(amount) => {
                vec![RecursiveField::named(
                    "amount",
                    RecursiveValue::Expr(amount.as_ref()),
                )]
            }

            // Complex actions
            PainterAction::curve {
                thickness,
                curvature,
            } => vec![
                RecursiveField::named("thickness", RecursiveValue::Expr(thickness.as_ref())),
                RecursiveField::named("curvature", RecursiveValue::Expr(curvature.as_ref())),
            ],

            PainterAction::repeat(count, action) => vec![
                RecursiveField::named("count", RecursiveValue::Expr(count.as_ref())),
                RecursiveField::named("action", RecursiveValue::PainterAction(action.as_ref())),
            ],

            PainterAction::list(actions) => actions
                .iter()
                .enumerate()
                .map(|(i, action)| {
                    RecursiveField::indexed(i, RecursiveValue::PainterAction(action.as_ref()))
                })
                .collect(),
        }
    }
}

// Reaction implementation
impl RecursiveFields for Reaction {
    fn recursive_fields(&self) -> Vec<RecursiveField<'_>> {
        self.actions
            .iter()
            .enumerate()
            .map(|(i, action)| RecursiveField::indexed(i, RecursiveValue::Action(action)))
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn example_nested_expression() -> Expression {
        Expression::sum(
            Box::new(Expression::i32(5)),
            Box::new(Expression::mul(
                Box::new(Expression::var(VarName::hp)),
                Box::new(Expression::f32(2.5)),
            )),
        )
    }

    fn complex_conditional_expression() -> Expression {
        Expression::r#if(
            Box::new(Expression::greater_then(
                Box::new(Expression::var(VarName::hp)),
                Box::new(Expression::i32(0)),
            )),
            Box::new(Expression::sum(
                Box::new(Expression::var(VarName::pwr)),
                Box::new(Expression::i32(10)),
            )),
            Box::new(Expression::zero),
        )
    }

    fn example_action() -> Action {
        Action::repeat(
            Box::new(Expression::i32(3)),
            vec![
                Box::new(Action::set_value(Box::new(Expression::var(VarName::hp)))),
                Box::new(Action::add_value(Box::new(Expression::sum(
                    Box::new(Expression::var(VarName::pwr)),
                    Box::new(Expression::i32(5)),
                )))),
            ],
        )
    }

    #[test]
    fn test_expression_recursive_fields() {
        let expr = example_nested_expression();
        let fields = expr.recursive_fields();
        assert_eq!(fields.len(), 2); // left and right
    }

    #[test]
    fn test_complex_expression_recursive_fields() {
        let expr = complex_conditional_expression();
        let fields = expr.recursive_fields();
        assert_eq!(fields.len(), 3); // condition, then, else
    }

    #[test]
    fn test_action_recursive_fields() {
        let action = example_action();
        let fields = action.recursive_fields();
        assert_eq!(fields.len(), 3); // count + 2 actions
    }

    #[test]
    fn test_primitive_no_recursive_fields() {
        let num = 42i32;
        let fields = num.recursive_fields();
        assert_eq!(fields.len(), 0); // Primitives have no recursive fields
    }

    #[test]
    fn test_vec_recursive_fields() {
        let vec_expr = vec![
            Expression::i32(1),
            Expression::var(VarName::hp),
            Expression::sum(Box::new(Expression::i32(2)), Box::new(Expression::i32(3))),
        ];
        let fields = vec_expr.recursive_fields();
        assert_eq!(fields.len(), 3); // 3 expressions
    }

    #[test]
    fn test_var_expression_shows_var_name() {
        let expr = Expression::var(VarName::hp);
        let fields = expr.recursive_fields();
        assert_eq!(fields.len(), 1); // Should have one field for the VarName
        assert_eq!(fields[0].name, "var");
        match &fields[0].value {
            RecursiveValue::Var(_) => (),
            _ => panic!("Expected Var recursive value"),
        }
    }

    #[test]
    fn test_state_var_expression_shows_both_fields() {
        let expr = Expression::state_var(Box::new(Expression::i32(5)), VarName::pwr);
        let fields = expr.recursive_fields();
        assert_eq!(fields.len(), 2); // Should have expr and var fields
        assert_eq!(fields[0].name, "expr");
        assert_eq!(fields[1].name, "var");
        match (&fields[0].value, &fields[1].value) {
            (RecursiveValue::Expr(_), RecursiveValue::Var(_)) => (),
            _ => panic!("Expected Expr and Var recursive values"),
        }
    }

    #[test]
    fn test_primitive_values_shown_in_expressions() {
        let expr = Expression::i32(42);
        let fields = expr.recursive_fields();
        assert_eq!(fields.len(), 1); // Should have one field for the i32
        assert_eq!(fields[0].name, "i32");
        match &fields[0].value {
            RecursiveValue::I32(val) => assert_eq!(**val, 42),
            _ => panic!("Expected I32 recursive value"),
        }
    }
}
