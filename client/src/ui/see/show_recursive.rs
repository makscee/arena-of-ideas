use super::*;

/// Trait for types that can be displayed recursively with proper nested field handling.
/// This trait focuses on defining which fields should be shown recursively,
/// while the actual display logic is reused.
pub trait RecursiveShow {
    /// Returns an iterator of field names and their recursive values
    fn recursive_fields(&self) -> Vec<RecursiveField<'_>>;

    /// Main recursive display method with layout handling
    fn show_recursive(&self, context: &Context, ui: &mut Ui, depth: usize)
    where
        Self: SFnShow,
    {
        self.show_recursive_content(context, ui, depth);
    }

    /// Internal method to show the actual content
    fn show_recursive_content(&self, context: &Context, ui: &mut Ui, depth: usize)
    where
        Self: SFnShow,
    {
        let fields = self.recursive_fields();

        if fields.is_empty() {
            self.show(context, ui);
        } else {
            self.show(context, ui);

            ui.vertical(|ui| {
                for field in fields {
                    ui.horizontal(|ui| match field {
                        RecursiveField::NamedExpr(name, value) => {
                            format!("[s [tw {}:]]", name).cstr().label(ui);
                            SFnShowRecursive::show_recursive(value, context, ui, depth + 1);
                        }
                        RecursiveField::NamedAction(name, value) => {
                            format!("[s [tw {}:]]", name).cstr().label(ui);
                            SFnShowRecursive::show_recursive(value, context, ui, depth + 1);
                        }
                        RecursiveField::NamedPainterAction(name, value) => {
                            format!("[s [tw {}:]]", name).cstr().label(ui);
                            SFnShowRecursive::show_recursive(value, context, ui, depth + 1);
                        }
                        RecursiveField::IndexedExpr(index, value) => {
                            format!("[s [tw {}:]]", index).cstr().label(ui);
                            SFnShowRecursive::show_recursive(value, context, ui, depth + 1);
                        }
                        RecursiveField::IndexedAction(index, value) => {
                            format!("[s [tw {}:]]", index).cstr().label(ui);
                            SFnShowRecursive::show_recursive(value, context, ui, depth + 1);
                        }
                        RecursiveField::IndexedPainterAction(index, value) => {
                            format!("[s [tw {}:]]", index).cstr().label(ui);
                            SFnShowRecursive::show_recursive(value, context, ui, depth + 1);
                        }
                    });
                }
            });
        }
    }
}

/// Represents a field that can be displayed recursively
pub enum RecursiveField<'a> {
    /// Named expression field
    NamedExpr(&'static str, &'a Expression),
    /// Named action field
    NamedAction(&'static str, &'a Action),
    /// Named painter action field
    NamedPainterAction(&'static str, &'a PainterAction),
    /// Indexed expression field
    IndexedExpr(usize, &'a Expression),
    /// Indexed action field
    IndexedAction(usize, &'a Action),
    /// Indexed painter action field
    IndexedPainterAction(usize, &'a PainterAction),
}

/// Main trait for recursive showing - this is what gets called from the UI
pub trait SFnShowRecursive {
    fn show_recursive(&self, context: &Context, ui: &mut Ui, depth: usize);
}

// Blanket implementation for all types that implement RecursiveShow + SFnShow
impl<T> SFnShowRecursive for T
where
    T: RecursiveShow + SFnShow,
{
    fn show_recursive(&self, context: &Context, ui: &mut Ui, depth: usize) {
        RecursiveShow::show_recursive(self, context, ui, depth)
    }
}

// SFnShow implementations for types that need recursive showing
impl SFnShow for Action {
    fn show(&self, _context: &Context, ui: &mut Ui) {
        format!("{:?}", std::mem::discriminant(self))
            .cstr()
            .label(ui);
    }
}

impl SFnShow for PainterAction {
    fn show(&self, _context: &Context, ui: &mut Ui) {
        format!("{:?}", std::mem::discriminant(self))
            .cstr()
            .label(ui);
    }
}

// Implementations for primitive types (leaf nodes)
impl RecursiveShow for i32 {
    fn recursive_fields(&self) -> Vec<RecursiveField<'_>> {
        vec![]
    }
}

impl RecursiveShow for f32 {
    fn recursive_fields(&self) -> Vec<RecursiveField<'_>> {
        vec![]
    }
}

impl RecursiveShow for String {
    fn recursive_fields(&self) -> Vec<RecursiveField<'_>> {
        vec![]
    }
}

impl RecursiveShow for bool {
    fn recursive_fields(&self) -> Vec<RecursiveField<'_>> {
        vec![]
    }
}

impl RecursiveShow for VarName {
    fn recursive_fields(&self) -> Vec<RecursiveField<'_>> {
        vec![]
    }
}

impl RecursiveShow for VarValue {
    fn recursive_fields(&self) -> Vec<RecursiveField<'_>> {
        vec![]
    }
}

impl RecursiveShow for HexColor {
    fn recursive_fields(&self) -> Vec<RecursiveField<'_>> {
        vec![]
    }
}

// Container type implementations for Expression
impl RecursiveShow for Box<Expression> {
    fn recursive_fields(&self) -> Vec<RecursiveField<'_>> {
        self.as_ref().recursive_fields()
    }
}

impl RecursiveShow for Vec<Expression> {
    fn recursive_fields(&self) -> Vec<RecursiveField<'_>> {
        self.iter()
            .enumerate()
            .map(|(i, item)| RecursiveField::IndexedExpr(i, item))
            .collect()
    }
}

impl RecursiveShow for Vec<Action> {
    fn recursive_fields(&self) -> Vec<RecursiveField<'_>> {
        self.iter()
            .enumerate()
            .map(|(i, item)| RecursiveField::IndexedAction(i, item))
            .collect()
    }
}

impl RecursiveShow for Vec<Box<Action>> {
    fn recursive_fields(&self) -> Vec<RecursiveField<'_>> {
        self.iter()
            .enumerate()
            .map(|(i, item)| RecursiveField::IndexedAction(i, item.as_ref()))
            .collect()
    }
}

impl RecursiveShow for Vec<Box<PainterAction>> {
    fn recursive_fields(&self) -> Vec<RecursiveField<'_>> {
        self.iter()
            .enumerate()
            .map(|(i, item)| RecursiveField::IndexedPainterAction(i, item.as_ref()))
            .collect()
    }
}

impl RecursiveShow for Option<Expression> {
    fn recursive_fields(&self) -> Vec<RecursiveField<'_>> {
        match self {
            Some(value) => vec![RecursiveField::NamedExpr("value", value)],
            None => vec![], // None is a leaf node
        }
    }
}

// Expression implementation
impl RecursiveShow for Expression {
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

            // Simple value expressions (leaf nodes)
            Expression::var(_)
            | Expression::var_sum(_)
            | Expression::value(_)
            | Expression::string(_)
            | Expression::f32(_)
            | Expression::f32_slider(_)
            | Expression::i32(_)
            | Expression::bool(_)
            | Expression::vec2(_, _)
            | Expression::color(_)
            | Expression::lua_i32(_)
            | Expression::lua_f32(_) => vec![],

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
                vec![RecursiveField::NamedExpr("expr", expr.as_ref())]
            }

            // Special case: state_var has both an expression and a var
            Expression::state_var(expr, _var) => {
                vec![RecursiveField::NamedExpr("expr", expr.as_ref())]
                // Note: var is not recursive, it will be shown by the base show() method
            }

            // Binary expressions
            Expression::vec2_ee(a, b) => vec![
                RecursiveField::NamedExpr("x", a.as_ref()),
                RecursiveField::NamedExpr("y", b.as_ref()),
            ],
            Expression::str_macro(template, value) => vec![
                RecursiveField::NamedExpr("template", template.as_ref()),
                RecursiveField::NamedExpr("value", value.as_ref()),
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
                RecursiveField::NamedExpr("left", left.as_ref()),
                RecursiveField::NamedExpr("right", right.as_ref()),
            ],
            Expression::fallback(primary, fallback) => vec![
                RecursiveField::NamedExpr("primary", primary.as_ref()),
                RecursiveField::NamedExpr("fallback", fallback.as_ref()),
            ],

            // Ternary expressions
            Expression::r#if(condition, then_expr, else_expr) => vec![
                RecursiveField::NamedExpr("condition", condition.as_ref()),
                RecursiveField::NamedExpr("then", then_expr.as_ref()),
                RecursiveField::NamedExpr("else", else_expr.as_ref()),
            ],
            Expression::oklch(l, c, h) => vec![
                RecursiveField::NamedExpr("lightness", l.as_ref()),
                RecursiveField::NamedExpr("chroma", c.as_ref()),
                RecursiveField::NamedExpr("hue", h.as_ref()),
            ],
        }
    }
}

// Action implementation
impl RecursiveShow for Action {
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
                vec![RecursiveField::NamedExpr("expr", expr.as_ref())]
            }

            // Complex action: repeat
            Action::repeat(count_expr, actions) => {
                let mut fields = vec![RecursiveField::NamedExpr("count", count_expr.as_ref())];
                for (i, action) in actions.iter().enumerate() {
                    fields.push(RecursiveField::IndexedAction(i, action.as_ref()));
                }
                fields
            }
        }
    }
}

// PainterAction implementation
impl RecursiveShow for PainterAction {
    fn recursive_fields(&self) -> Vec<RecursiveField<'_>> {
        match self {
            // Leaf action
            PainterAction::paint => vec![],

            // Actions with one expression
            PainterAction::circle(radius) => {
                vec![RecursiveField::NamedExpr("radius", radius.as_ref())]
            }
            PainterAction::rectangle(size) => {
                vec![RecursiveField::NamedExpr("size", size.as_ref())]
            }
            PainterAction::text(content) => {
                vec![RecursiveField::NamedExpr("content", content.as_ref())]
            }
            PainterAction::hollow(thickness) => {
                vec![RecursiveField::NamedExpr("thickness", thickness.as_ref())]
            }
            PainterAction::translate(offset) => {
                vec![RecursiveField::NamedExpr("offset", offset.as_ref())]
            }
            PainterAction::rotate(angle) => {
                vec![RecursiveField::NamedExpr("angle", angle.as_ref())]
            }
            PainterAction::scale_mesh(factor) => {
                vec![RecursiveField::NamedExpr("factor", factor.as_ref())]
            }
            PainterAction::scale_rect(factor) => {
                vec![RecursiveField::NamedExpr("factor", factor.as_ref())]
            }
            PainterAction::color(color_expr) => {
                vec![RecursiveField::NamedExpr("color", color_expr.as_ref())]
            }
            PainterAction::alpha(alpha_expr) => {
                vec![RecursiveField::NamedExpr("alpha", alpha_expr.as_ref())]
            }
            PainterAction::feathering(amount) => {
                vec![RecursiveField::NamedExpr("amount", amount.as_ref())]
            }

            // Complex actions
            PainterAction::curve {
                thickness,
                curvature,
            } => vec![
                RecursiveField::NamedExpr("thickness", thickness.as_ref()),
                RecursiveField::NamedExpr("curvature", curvature.as_ref()),
            ],

            PainterAction::repeat(count, action) => vec![
                RecursiveField::NamedExpr("count", count.as_ref()),
                RecursiveField::NamedPainterAction("action", action.as_ref()),
            ],

            PainterAction::list(actions) => actions
                .iter()
                .enumerate()
                .map(|(i, action)| RecursiveField::IndexedPainterAction(i, action.as_ref()))
                .collect(),
        }
    }
}

// Reaction implementation (as an example of another complex type)
impl RecursiveShow for Reaction {
    fn recursive_fields(&self) -> Vec<RecursiveField<'_>> {
        self.actions
            .iter()
            .enumerate()
            .map(|(i, action)| RecursiveField::IndexedAction(i, action))
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
}
