use super::*;

pub trait RecursiveFields {
    fn recursive_fields(&self) -> Vec<RecursiveField<'_>>;
}

pub trait RecursiveFieldsMut {
    fn recursive_fields_mut(&mut self) -> Vec<RecursiveFieldMut<'_>>;
}

pub struct RecursiveField<'a> {
    pub name: String,
    pub value: RecursiveValue<'a>,
}

#[derive(Copy, Clone)]
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
    Reaction(&'a Reaction),
}

#[derive(Debug)]
pub struct RecursiveFieldMut<'a> {
    pub name: String,
    pub value: RecursiveValueMut<'a>,
}

#[derive(Debug)]
pub enum RecursiveValueMut<'a> {
    Expr(&'a mut Expression),
    Action(&'a mut Action),
    PainterAction(&'a mut PainterAction),
    Var(&'a mut VarName),
    VarValue(&'a mut VarValue),
    HexColor(&'a mut HexColor),
    String(&'a mut String),
    I32(&'a mut i32),
    F32(&'a mut f32),
    Bool(&'a mut bool),
    Vec2(&'a mut Vec2),
    Reaction(&'a mut Reaction),
}

pub trait SFnRecursive {
    /// Renders root value and all nested fields recursively with a custom closure.
    ///
    /// This function converts the root value to a `RecursiveField` and uses a single
    /// recursive render function that:
    /// 1. Calls the closure for the current field (in horizontal layout)
    /// 2. Gets nested fields from the current field's value
    /// 3. Recursively calls itself for each nested field (in vertical layout)
    ///
    /// The closure uses `FnMut` so you can modify variables outside the closure.
    /// The root value is automatically converted using `ToRecursiveValue`.
    ///
    /// Usage:
    /// ```
    /// let mut field_names = Vec::new();
    /// expression.see(context).recursive(ui, |ui, context, field| {
    ///     if field.name.is_empty() {
    ///         ui.label("ROOT"); // Root value has empty name
    ///     } else {
    ///         field.name.label(ui); // Shows names of all nested fields
    ///         field_names.push(field.name.clone()); // Modify external variable
    ///     }
    /// });
    /// ```
    ///
    /// The closure will be called for every field in the recursive tree structure.
    fn recursive<F>(&self, context: &Context, ui: &mut Ui, f: &mut F)
    where
        F: FnMut(&mut Ui, &Context, RecursiveField<'_>);
}

pub trait SFnRecursiveMut {
    /// Renders root value and all nested fields recursively with a custom closure that can modify values.
    ///
    /// This function converts the root value to a `RecursiveFieldMut` and uses a single
    /// recursive render function that:
    /// 1. Calls the closure for the current field (in horizontal layout)
    /// 2. Gets nested fields from the current field's value
    /// 3. Recursively calls itself for each nested field (in vertical layout)
    ///
    /// The closure uses `FnMut` so you can modify variables outside the closure and the
    /// values within the recursive structure. The root value is automatically converted
    /// using `ToRecursiveValueMut`.
    ///
    /// Usage:
    /// ```
    /// let mut expression = Expression::f32(42.0);
    /// expression.see_mut(context).recursive(ui, |ui, context, mut field| {
    ///     if field.name.is_empty() {
    ///         ui.label("ROOT"); // Root value has empty name
    ///     } else {
    ///         field.name.label(ui); // Shows names of all nested fields
    ///         // Can modify mutable fields here
    ///         call_on_recursive_value_mut!(field, show_mut, context, ui);
    ///     }
    /// });
    /// ```
    ///
    /// The closure will be called for every field in the recursive tree structure.
    fn recursive<F>(&mut self, context: &Context, ui: &mut Ui, f: &mut F)
    where
        F: FnMut(&mut Ui, &Context, RecursiveFieldMut<'_>);
}

impl<T> SFnRecursive for T
where
    T: RecursiveFields + ToRecursiveValue,
{
    fn recursive<F>(&self, context: &Context, ui: &mut Ui, f: &mut F)
    where
        F: FnMut(&mut Ui, &Context, RecursiveField<'_>),
    {
        let root_field = RecursiveField {
            name: String::new(),
            value: self.to_recursive_value(),
        };
        render_field_recursive(root_field, context, ui, f);
    }
}

impl<T> SFnRecursiveMut for T
where
    T: RecursiveFieldsMut + ToRecursiveValueMut,
{
    fn recursive<F>(&mut self, context: &Context, ui: &mut Ui, f: &mut F)
    where
        F: FnMut(&mut Ui, &Context, RecursiveFieldMut<'_>),
    {
        let root_field = RecursiveFieldMut {
            name: String::new(),
            value: self.to_recursive_value_mut(),
        };
        render_field_recursive_mut(root_field, context, ui, f);
    }
}

/// Calls a method on the value inside a RecursiveField.
///
/// This macro takes a RecursiveField, extracts the value, and calls the specified
/// method on it with the provided arguments.
///
/// # Usage
/// ```
/// call_on_recursive_value!(field, show, context, ui);
/// call_on_recursive_value!(field, some_method, arg1, arg2, arg3);
/// ```
///
/// The macro will match on the RecursiveValue variant and call the method
/// on the underlying value with all the provided arguments.
#[macro_export]
macro_rules! call_on_recursive_value {
    ($field:expr, $func:ident $(, $arg:expr)*) => {
        match $field.value {
            RecursiveValue::Expr(v) => v.$func($($arg),*),
            RecursiveValue::Action(v) => v.$func($($arg),*),
            RecursiveValue::PainterAction(v) => v.$func($($arg),*),
            RecursiveValue::Var(v) => v.$func($($arg),*),
            RecursiveValue::VarValue(v) => v.$func($($arg),*),
            RecursiveValue::HexColor(v) => v.$func($($arg),*),
            RecursiveValue::String(v) => v.$func($($arg),*),
            RecursiveValue::I32(v) => v.$func($($arg),*),
            RecursiveValue::F32(v) => v.$func($($arg),*),
            RecursiveValue::Bool(v) => v.$func($($arg),*),
            RecursiveValue::Vec2(v) => v.$func($($arg),*),
            RecursiveValue::Reaction(v) => v.$func($($arg),*),
        }
    };
}

#[macro_export]
macro_rules! call_pass_recursive_value {
    ($field:expr, $func:ident $(, $arg:expr)*) => {
        match $field.value {
            RecursiveValue::Expr(v) => $func(v, $($arg),*),
            RecursiveValue::Action(v) => $func(v, $($arg),*),
            RecursiveValue::PainterAction(v) => $func(v, $($arg),*),
            RecursiveValue::Var(v) => $func(v, $($arg),*),
            RecursiveValue::VarValue(v) => $func(v, $($arg),*),
            RecursiveValue::HexColor(v) => $func(v, $($arg),*),
            RecursiveValue::String(v) => $func(v, $($arg),*),
            RecursiveValue::I32(v) => $func(v, $($arg),*),
            RecursiveValue::F32(v) => $func(v, $($arg),*),
            RecursiveValue::Bool(v) => $func(v, $($arg),*),
            RecursiveValue::Vec2(v) => $func(v, $($arg),*),
            RecursiveValue::Reaction(v) => $func(v, $($arg),*),
        }
    };
}

#[macro_export]
macro_rules! call_on_recursive_value_mut {
    ($field:expr, $func:ident $(, $arg:expr)*) => {
        match $field.value {
            RecursiveValueMut::Expr(v) => v.$func($($arg),*),
            RecursiveValueMut::Action(v) => v.$func($($arg),*),
            RecursiveValueMut::PainterAction(v) => v.$func($($arg),*),
            RecursiveValueMut::Var(v) => v.$func($($arg),*),
            RecursiveValueMut::VarValue(v) => v.$func($($arg),*),
            RecursiveValueMut::HexColor(v) => v.$func($($arg),*),
            RecursiveValueMut::String(v) => v.$func($($arg),*),
            RecursiveValueMut::I32(v) => v.$func($($arg),*),
            RecursiveValueMut::F32(v) => v.$func($($arg),*),
            RecursiveValueMut::Bool(v) => v.$func($($arg),*),
            RecursiveValueMut::Vec2(v) => v.$func($($arg),*),
            RecursiveValueMut::Reaction(v) => v.$func($($arg),*),
        }
    };
}

#[macro_export]
macro_rules! call_pass_recursive_value_mut {
    ($field:expr, $func:ident $(, $arg:expr)*) => {
        match $field.value {
            RecursiveValueMut::Expr(v) => $func(v, $($arg),*),
            RecursiveValueMut::Action(v) => $func(v, $($arg),*),
            RecursiveValueMut::PainterAction(v) => $func(v, $($arg),*),
            RecursiveValueMut::Var(v) => $func(v, $($arg),*),
            RecursiveValueMut::VarValue(v) => $func(v, $($arg),*),
            RecursiveValueMut::HexColor(v) => $func(v, $($arg),*),
            RecursiveValueMut::String(v) => $func(v, $($arg),*),
            RecursiveValueMut::I32(v) => $func(v, $($arg),*),
            RecursiveValueMut::F32(v) => $func(v, $($arg),*),
            RecursiveValueMut::Bool(v) => $func(v, $($arg),*),
            RecursiveValueMut::Vec2(v) => $func(v, $($arg),*),
            RecursiveValueMut::Reaction(v) => $func(v, $($arg),*),
        }
    };
}

pub trait ToRecursiveValue {
    fn to_recursive_value(&self) -> RecursiveValue<'_>;
}

pub trait ToRecursiveValueMut {
    fn to_recursive_value_mut(&mut self) -> RecursiveValueMut<'_>;
}

impl ToRecursiveValue for Expression {
    fn to_recursive_value(&self) -> RecursiveValue<'_> {
        RecursiveValue::Expr(self)
    }
}

impl ToRecursiveValue for Action {
    fn to_recursive_value(&self) -> RecursiveValue<'_> {
        RecursiveValue::Action(self)
    }
}

impl ToRecursiveValue for PainterAction {
    fn to_recursive_value(&self) -> RecursiveValue<'_> {
        RecursiveValue::PainterAction(self)
    }
}

impl ToRecursiveValue for VarName {
    fn to_recursive_value(&self) -> RecursiveValue<'_> {
        RecursiveValue::Var(self)
    }
}

impl ToRecursiveValue for VarValue {
    fn to_recursive_value(&self) -> RecursiveValue<'_> {
        RecursiveValue::VarValue(self)
    }
}

impl ToRecursiveValue for HexColor {
    fn to_recursive_value(&self) -> RecursiveValue<'_> {
        RecursiveValue::HexColor(self)
    }
}

impl ToRecursiveValue for String {
    fn to_recursive_value(&self) -> RecursiveValue<'_> {
        RecursiveValue::String(self)
    }
}

impl ToRecursiveValue for i32 {
    fn to_recursive_value(&self) -> RecursiveValue<'_> {
        RecursiveValue::I32(self)
    }
}

impl ToRecursiveValue for f32 {
    fn to_recursive_value(&self) -> RecursiveValue<'_> {
        RecursiveValue::F32(self)
    }
}

impl ToRecursiveValue for bool {
    fn to_recursive_value(&self) -> RecursiveValue<'_> {
        RecursiveValue::Bool(self)
    }
}

impl ToRecursiveValue for Vec2 {
    fn to_recursive_value(&self) -> RecursiveValue<'_> {
        RecursiveValue::Vec2(self)
    }
}

impl ToRecursiveValue for Reaction {
    fn to_recursive_value(&self) -> RecursiveValue<'_> {
        RecursiveValue::Reaction(self)
    }
}

impl ToRecursiveValueMut for Expression {
    fn to_recursive_value_mut(&mut self) -> RecursiveValueMut<'_> {
        RecursiveValueMut::Expr(self)
    }
}

impl ToRecursiveValueMut for Action {
    fn to_recursive_value_mut(&mut self) -> RecursiveValueMut<'_> {
        RecursiveValueMut::Action(self)
    }
}

impl ToRecursiveValueMut for PainterAction {
    fn to_recursive_value_mut(&mut self) -> RecursiveValueMut<'_> {
        RecursiveValueMut::PainterAction(self)
    }
}

impl ToRecursiveValueMut for VarName {
    fn to_recursive_value_mut(&mut self) -> RecursiveValueMut<'_> {
        RecursiveValueMut::Var(self)
    }
}

impl ToRecursiveValueMut for VarValue {
    fn to_recursive_value_mut(&mut self) -> RecursiveValueMut<'_> {
        RecursiveValueMut::VarValue(self)
    }
}

impl ToRecursiveValueMut for HexColor {
    fn to_recursive_value_mut(&mut self) -> RecursiveValueMut<'_> {
        RecursiveValueMut::HexColor(self)
    }
}

impl ToRecursiveValueMut for String {
    fn to_recursive_value_mut(&mut self) -> RecursiveValueMut<'_> {
        RecursiveValueMut::String(self)
    }
}

impl ToRecursiveValueMut for i32 {
    fn to_recursive_value_mut(&mut self) -> RecursiveValueMut<'_> {
        RecursiveValueMut::I32(self)
    }
}

impl ToRecursiveValueMut for f32 {
    fn to_recursive_value_mut(&mut self) -> RecursiveValueMut<'_> {
        RecursiveValueMut::F32(self)
    }
}

impl ToRecursiveValueMut for bool {
    fn to_recursive_value_mut(&mut self) -> RecursiveValueMut<'_> {
        RecursiveValueMut::Bool(self)
    }
}

impl ToRecursiveValueMut for Vec2 {
    fn to_recursive_value_mut(&mut self) -> RecursiveValueMut<'_> {
        RecursiveValueMut::Vec2(self)
    }
}

impl ToRecursiveValueMut for Reaction {
    fn to_recursive_value_mut(&mut self) -> RecursiveValueMut<'_> {
        RecursiveValueMut::Reaction(self)
    }
}

macro_rules! replace_and_move_fields {
    ($value:expr, $new_value:expr) => {{
        std::mem::swap($value, &mut $new_value);
        let mut old_fields = $new_value.recursive_fields_mut();
        let mut new_fields = $value.recursive_fields_mut();
        RecursiveValueMut::swap_matching_fields(&mut old_fields, &mut new_fields);
    }};
}

impl<'a> RecursiveValueMut<'a> {
    /// Replace expression and move matching fields
    pub fn replace_expr_and_move_fields(expr: &mut Expression, mut new_expr: Expression) {
        replace_and_move_fields!(expr, new_expr);
    }

    /// Replace action and move matching fields
    pub fn replace_action_and_move_fields(action: &mut Action, mut new_action: Action) {
        replace_and_move_fields!(action, new_action);
    }

    /// Replace painter action and move matching fields
    pub fn replace_painter_action_and_move_fields(
        painter_action: &mut PainterAction,
        mut new_painter_action: PainterAction,
    ) {
        replace_and_move_fields!(painter_action, new_painter_action);
    }

    fn swap_matching_fields<'b>(
        old_fields: &mut Vec<RecursiveFieldMut<'b>>,
        new_fields: &mut Vec<RecursiveFieldMut<'b>>,
    ) {
        for new_field in new_fields.iter_mut() {
            let mut found_index = None;
            for (i, old_field) in old_fields.iter_mut().enumerate() {
                if Self::swap_field_values(&mut old_field.value, &mut new_field.value) {
                    found_index = Some(i);
                    break;
                }
            }
            if let Some(i) = found_index {
                old_fields.remove(i);
            }
        }
    }

    /// Swap values between two matching field types, returns true if swap occurred
    fn swap_field_values<'b>(
        old: &mut RecursiveValueMut<'b>,
        new: &mut RecursiveValueMut<'b>,
    ) -> bool {
        match (old, new) {
            (RecursiveValueMut::Expr(old_e), RecursiveValueMut::Expr(new_e)) => {
                std::mem::swap(*old_e, *new_e);
                true
            }
            (RecursiveValueMut::Action(old_a), RecursiveValueMut::Action(new_a)) => {
                std::mem::swap(*old_a, *new_a);
                true
            }
            (RecursiveValueMut::PainterAction(old_p), RecursiveValueMut::PainterAction(new_p)) => {
                std::mem::swap(*old_p, *new_p);
                true
            }
            (RecursiveValueMut::Var(old_v), RecursiveValueMut::Var(new_v)) => {
                std::mem::swap(*old_v, *new_v);
                true
            }
            (RecursiveValueMut::VarValue(old_v), RecursiveValueMut::VarValue(new_v)) => {
                std::mem::swap(*old_v, *new_v);
                true
            }
            (RecursiveValueMut::HexColor(old_c), RecursiveValueMut::HexColor(new_c)) => {
                std::mem::swap(*old_c, *new_c);
                true
            }
            (RecursiveValueMut::String(old_s), RecursiveValueMut::String(new_s)) => {
                std::mem::swap(*old_s, *new_s);
                true
            }
            (RecursiveValueMut::I32(old_i), RecursiveValueMut::I32(new_i)) => {
                std::mem::swap(*old_i, *new_i);
                true
            }
            (RecursiveValueMut::F32(old_f), RecursiveValueMut::F32(new_f)) => {
                std::mem::swap(*old_f, *new_f);
                true
            }
            (RecursiveValueMut::Bool(old_b), RecursiveValueMut::Bool(new_b)) => {
                std::mem::swap(*old_b, *new_b);
                true
            }
            (RecursiveValueMut::Vec2(old_v), RecursiveValueMut::Vec2(new_v)) => {
                std::mem::swap(*old_v, *new_v);
                true
            }
            (RecursiveValueMut::Reaction(old_r), RecursiveValueMut::Reaction(new_r)) => {
                std::mem::swap(*old_r, *new_r);
                true
            }
            _ => false, // Types don't match, can't swap
        }
    }
}

fn render_field_recursive<F>(field: RecursiveField<'_>, context: &Context, ui: &mut Ui, f: &mut F)
where
    F: FnMut(&mut Ui, &Context, RecursiveField<'_>),
{
    ui.horizontal(|ui| {
        f(
            ui,
            context,
            RecursiveField {
                name: field.name.clone(),
                value: field.value,
            },
        );
        ui.vertical(|ui| {
            for nested_field in call_on_recursive_value!(field, recursive_fields) {
                render_field_recursive(nested_field, context, ui, f);
            }
        });
    });
}

fn render_field_recursive_mut<F>(
    mut field: RecursiveFieldMut<'_>,
    context: &Context,
    ui: &mut Ui,
    f: &mut F,
) where
    F: FnMut(&mut Ui, &Context, RecursiveFieldMut<'_>),
{
    ui.horizontal(|ui| {
        // Call the function first to allow modifications
        f(
            ui,
            context,
            RecursiveFieldMut {
                name: field.name.clone(),
                value: match &mut field.value {
                    RecursiveValueMut::Expr(v) => RecursiveValueMut::Expr(v),
                    RecursiveValueMut::Action(v) => RecursiveValueMut::Action(v),
                    RecursiveValueMut::PainterAction(v) => RecursiveValueMut::PainterAction(v),
                    RecursiveValueMut::Var(v) => RecursiveValueMut::Var(v),
                    RecursiveValueMut::VarValue(v) => RecursiveValueMut::VarValue(v),
                    RecursiveValueMut::HexColor(v) => RecursiveValueMut::HexColor(v),
                    RecursiveValueMut::String(v) => RecursiveValueMut::String(v),
                    RecursiveValueMut::I32(v) => RecursiveValueMut::I32(v),
                    RecursiveValueMut::F32(v) => RecursiveValueMut::F32(v),
                    RecursiveValueMut::Bool(v) => RecursiveValueMut::Bool(v),
                    RecursiveValueMut::Vec2(v) => RecursiveValueMut::Vec2(v),
                    RecursiveValueMut::Reaction(v) => RecursiveValueMut::Reaction(v),
                },
            },
        );

        ui.vertical(|ui| {
            // Get nested fields after the function call
            for nested_field in call_on_recursive_value_mut!(field, recursive_fields_mut) {
                render_field_recursive_mut(nested_field, context, ui, f);
            }
        });
    });
}

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

impl<'a> RecursiveFieldMut<'a> {
    fn named(name: &str, value: RecursiveValueMut<'a>) -> Self {
        Self {
            name: name.to_string(),
            value,
        }
    }

    fn indexed(index: usize, value: RecursiveValueMut<'a>) -> Self {
        Self {
            name: index.to_string(),
            value,
        }
    }
}

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

impl RecursiveFieldsMut for i32 {
    fn recursive_fields_mut(&mut self) -> Vec<RecursiveFieldMut<'_>> {
        vec![]
    }
}

impl RecursiveFieldsMut for f32 {
    fn recursive_fields_mut(&mut self) -> Vec<RecursiveFieldMut<'_>> {
        vec![]
    }
}

impl RecursiveFieldsMut for String {
    fn recursive_fields_mut(&mut self) -> Vec<RecursiveFieldMut<'_>> {
        vec![]
    }
}

impl RecursiveFieldsMut for bool {
    fn recursive_fields_mut(&mut self) -> Vec<RecursiveFieldMut<'_>> {
        vec![]
    }
}

impl RecursiveFieldsMut for VarName {
    fn recursive_fields_mut(&mut self) -> Vec<RecursiveFieldMut<'_>> {
        vec![]
    }
}

impl RecursiveFieldsMut for VarValue {
    fn recursive_fields_mut(&mut self) -> Vec<RecursiveFieldMut<'_>> {
        vec![]
    }
}

impl RecursiveFieldsMut for HexColor {
    fn recursive_fields_mut(&mut self) -> Vec<RecursiveFieldMut<'_>> {
        vec![]
    }
}

impl RecursiveFieldsMut for Vec2 {
    fn recursive_fields_mut(&mut self) -> Vec<RecursiveFieldMut<'_>> {
        vec![]
    }
}

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

impl RecursiveFields for Expression {
    fn recursive_fields(&self) -> Vec<RecursiveField<'_>> {
        match self {
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

            Expression::vec2(x, y) => vec![
                RecursiveField::named("x", RecursiveValue::F32(x)),
                RecursiveField::named("y", RecursiveValue::F32(y)),
            ],

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

            Expression::state_var(expr, var) => vec![
                RecursiveField::named("expr", RecursiveValue::Expr(expr.as_ref())),
                RecursiveField::named("var", RecursiveValue::Var(var)),
            ],

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

impl RecursiveFields for Action {
    fn recursive_fields(&self) -> Vec<RecursiveField<'_>> {
        match self {
            Action::noop
            | Action::deal_damage
            | Action::heal_damage
            | Action::use_ability
            | Action::apply_status => vec![],

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

impl RecursiveFields for PainterAction {
    fn recursive_fields(&self) -> Vec<RecursiveField<'_>> {
        match self {
            PainterAction::paint => vec![],

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

impl RecursiveFields for Reaction {
    fn recursive_fields(&self) -> Vec<RecursiveField<'_>> {
        self.actions
            .iter()
            .enumerate()
            .map(|(i, action)| RecursiveField::indexed(i, RecursiveValue::Action(action)))
            .collect()
    }
}

impl RecursiveFieldsMut for Expression {
    fn recursive_fields_mut(&mut self) -> Vec<RecursiveFieldMut<'_>> {
        match self {
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

            Expression::var(var) => {
                vec![RecursiveFieldMut::named("var", RecursiveValueMut::Var(var))]
            }
            Expression::var_sum(var) => {
                vec![RecursiveFieldMut::named("var", RecursiveValueMut::Var(var))]
            }
            Expression::value(val) => vec![RecursiveFieldMut::named(
                "value",
                RecursiveValueMut::VarValue(val),
            )],
            Expression::string(s) => {
                vec![RecursiveFieldMut::named(
                    "string",
                    RecursiveValueMut::String(s),
                )]
            }
            Expression::f32(f) => vec![RecursiveFieldMut::named("f32", RecursiveValueMut::F32(f))],
            Expression::f32_slider(f) => {
                vec![RecursiveFieldMut::named(
                    "f32_slider",
                    RecursiveValueMut::F32(f),
                )]
            }
            Expression::i32(i) => vec![RecursiveFieldMut::named("i32", RecursiveValueMut::I32(i))],
            Expression::bool(b) => {
                vec![RecursiveFieldMut::named("bool", RecursiveValueMut::Bool(b))]
            }
            Expression::color(c) => {
                vec![RecursiveFieldMut::named(
                    "color",
                    RecursiveValueMut::HexColor(c),
                )]
            }
            Expression::lua_i32(i) => {
                vec![RecursiveFieldMut::named(
                    "lua_i32",
                    RecursiveValueMut::String(i),
                )]
            }
            Expression::lua_f32(f) => {
                vec![RecursiveFieldMut::named(
                    "lua_f32",
                    RecursiveValueMut::String(f),
                )]
            }

            Expression::vec2(x, y) => vec![
                RecursiveFieldMut::named("x", RecursiveValueMut::F32(x)),
                RecursiveFieldMut::named("y", RecursiveValueMut::F32(y)),
            ],

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
                vec![RecursiveFieldMut::named(
                    "expr",
                    RecursiveValueMut::Expr(expr.as_mut()),
                )]
            }

            Expression::state_var(expr, var) => vec![
                RecursiveFieldMut::named("expr", RecursiveValueMut::Expr(expr.as_mut())),
                RecursiveFieldMut::named("var", RecursiveValueMut::Var(var)),
            ],

            Expression::vec2_ee(a, b) => vec![
                RecursiveFieldMut::named("x", RecursiveValueMut::Expr(a.as_mut())),
                RecursiveFieldMut::named("y", RecursiveValueMut::Expr(b.as_mut())),
            ],
            Expression::str_macro(template, value) => vec![
                RecursiveFieldMut::named("template", RecursiveValueMut::Expr(template.as_mut())),
                RecursiveFieldMut::named("value", RecursiveValueMut::Expr(value.as_mut())),
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
                RecursiveFieldMut::named("left", RecursiveValueMut::Expr(left.as_mut())),
                RecursiveFieldMut::named("right", RecursiveValueMut::Expr(right.as_mut())),
            ],
            Expression::fallback(primary, fallback) => vec![
                RecursiveFieldMut::named("primary", RecursiveValueMut::Expr(primary.as_mut())),
                RecursiveFieldMut::named("fallback", RecursiveValueMut::Expr(fallback.as_mut())),
            ],

            Expression::r#if(condition, then_expr, else_expr) => vec![
                RecursiveFieldMut::named("condition", RecursiveValueMut::Expr(condition.as_mut())),
                RecursiveFieldMut::named("then", RecursiveValueMut::Expr(then_expr.as_mut())),
                RecursiveFieldMut::named("else", RecursiveValueMut::Expr(else_expr.as_mut())),
            ],
            Expression::oklch(l, c, h) => vec![
                RecursiveFieldMut::named("lightness", RecursiveValueMut::Expr(l.as_mut())),
                RecursiveFieldMut::named("chroma", RecursiveValueMut::Expr(c.as_mut())),
                RecursiveFieldMut::named("hue", RecursiveValueMut::Expr(h.as_mut())),
            ],
        }
    }
}

impl RecursiveFieldsMut for Action {
    fn recursive_fields_mut(&mut self) -> Vec<RecursiveFieldMut<'_>> {
        match self {
            Action::noop
            | Action::deal_damage
            | Action::heal_damage
            | Action::use_ability
            | Action::apply_status => vec![],

            Action::debug(expr)
            | Action::set_value(expr)
            | Action::add_value(expr)
            | Action::subtract_value(expr)
            | Action::add_target(expr) => {
                vec![RecursiveFieldMut::named(
                    "expr",
                    RecursiveValueMut::Expr(expr.as_mut()),
                )]
            }

            Action::repeat(count_expr, actions) => {
                let mut fields = vec![RecursiveFieldMut::named(
                    "count",
                    RecursiveValueMut::Expr(count_expr.as_mut()),
                )];
                for (i, action) in actions.iter_mut().enumerate() {
                    fields.push(RecursiveFieldMut::indexed(
                        i,
                        RecursiveValueMut::Action(action.as_mut()),
                    ));
                }
                fields
            }
        }
    }
}

impl RecursiveFieldsMut for PainterAction {
    fn recursive_fields_mut(&mut self) -> Vec<RecursiveFieldMut<'_>> {
        match self {
            PainterAction::paint => vec![],

            PainterAction::circle(radius) => {
                vec![RecursiveFieldMut::named(
                    "radius",
                    RecursiveValueMut::Expr(radius.as_mut()),
                )]
            }
            PainterAction::rectangle(size) => {
                vec![RecursiveFieldMut::named(
                    "size",
                    RecursiveValueMut::Expr(size.as_mut()),
                )]
            }
            PainterAction::text(content) => {
                vec![RecursiveFieldMut::named(
                    "content",
                    RecursiveValueMut::Expr(content.as_mut()),
                )]
            }
            PainterAction::hollow(thickness) => {
                vec![RecursiveFieldMut::named(
                    "thickness",
                    RecursiveValueMut::Expr(thickness.as_mut()),
                )]
            }
            PainterAction::translate(offset) => {
                vec![RecursiveFieldMut::named(
                    "offset",
                    RecursiveValueMut::Expr(offset.as_mut()),
                )]
            }
            PainterAction::rotate(angle) => {
                vec![RecursiveFieldMut::named(
                    "angle",
                    RecursiveValueMut::Expr(angle.as_mut()),
                )]
            }
            PainterAction::scale_mesh(factor) => {
                vec![RecursiveFieldMut::named(
                    "factor",
                    RecursiveValueMut::Expr(factor.as_mut()),
                )]
            }
            PainterAction::scale_rect(factor) => {
                vec![RecursiveFieldMut::named(
                    "factor",
                    RecursiveValueMut::Expr(factor.as_mut()),
                )]
            }
            PainterAction::color(color_expr) => {
                vec![RecursiveFieldMut::named(
                    "color",
                    RecursiveValueMut::Expr(color_expr.as_mut()),
                )]
            }
            PainterAction::alpha(alpha_expr) => {
                vec![RecursiveFieldMut::named(
                    "alpha",
                    RecursiveValueMut::Expr(alpha_expr.as_mut()),
                )]
            }
            PainterAction::feathering(amount) => {
                vec![RecursiveFieldMut::named(
                    "amount",
                    RecursiveValueMut::Expr(amount.as_mut()),
                )]
            }

            PainterAction::curve {
                thickness,
                curvature,
            } => vec![
                RecursiveFieldMut::named("thickness", RecursiveValueMut::Expr(thickness.as_mut())),
                RecursiveFieldMut::named("curvature", RecursiveValueMut::Expr(curvature.as_mut())),
            ],

            PainterAction::repeat(count, action) => vec![
                RecursiveFieldMut::named("count", RecursiveValueMut::Expr(count.as_mut())),
                RecursiveFieldMut::named(
                    "action",
                    RecursiveValueMut::PainterAction(action.as_mut()),
                ),
            ],

            PainterAction::list(actions) => actions
                .iter_mut()
                .enumerate()
                .map(|(i, action)| {
                    RecursiveFieldMut::indexed(i, RecursiveValueMut::PainterAction(action.as_mut()))
                })
                .collect(),
        }
    }
}

impl RecursiveFieldsMut for Reaction {
    fn recursive_fields_mut(&mut self) -> Vec<RecursiveFieldMut<'_>> {
        self.actions
            .iter_mut()
            .enumerate()
            .map(|(i, action)| RecursiveFieldMut::indexed(i, RecursiveValueMut::Action(action)))
            .collect()
    }
}

impl RecursiveFieldsMut for Box<Expression> {
    fn recursive_fields_mut(&mut self) -> Vec<RecursiveFieldMut<'_>> {
        self.as_mut().recursive_fields_mut()
    }
}

impl RecursiveFieldsMut for Vec<Expression> {
    fn recursive_fields_mut(&mut self) -> Vec<RecursiveFieldMut<'_>> {
        self.iter_mut()
            .enumerate()
            .map(|(i, item)| RecursiveFieldMut::indexed(i, RecursiveValueMut::Expr(item)))
            .collect()
    }
}

impl RecursiveFieldsMut for Vec<Action> {
    fn recursive_fields_mut(&mut self) -> Vec<RecursiveFieldMut<'_>> {
        self.iter_mut()
            .enumerate()
            .map(|(i, item)| RecursiveFieldMut::indexed(i, RecursiveValueMut::Action(item)))
            .collect()
    }
}

impl RecursiveFieldsMut for Vec<Box<Action>> {
    fn recursive_fields_mut(&mut self) -> Vec<RecursiveFieldMut<'_>> {
        self.iter_mut()
            .enumerate()
            .map(|(i, item)| {
                RecursiveFieldMut::indexed(i, RecursiveValueMut::Action(item.as_mut()))
            })
            .collect()
    }
}

impl RecursiveFieldsMut for Vec<Box<PainterAction>> {
    fn recursive_fields_mut(&mut self) -> Vec<RecursiveFieldMut<'_>> {
        self.iter_mut()
            .enumerate()
            .map(|(i, item)| {
                RecursiveFieldMut::indexed(i, RecursiveValueMut::PainterAction(item.as_mut()))
            })
            .collect()
    }
}

impl RecursiveFieldsMut for Option<Expression> {
    fn recursive_fields_mut(&mut self) -> Vec<RecursiveFieldMut<'_>> {
        match self {
            Some(expr) => vec![RecursiveFieldMut::named(
                "value",
                RecursiveValueMut::Expr(expr),
            )],
            None => vec![],
        }
    }
}
