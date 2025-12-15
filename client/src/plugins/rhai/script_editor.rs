use super::*;
use egui_code_editor::{CodeEditor, ColorTheme, Syntax};
use egui_commonmark::{CommonMarkCache, CommonMarkViewer};

#[derive(Resource)]
pub struct RhaiScriptEditorState {
    pub current_script: Option<RhaiScript>,
    pub theme: ColorTheme,
    pub show_help: bool,
    pub markdown_cache: CommonMarkCache,
}

impl Default for RhaiScriptEditorState {
    fn default() -> Self {
        Self {
            current_script: None,
            theme: ColorTheme::GRUVBOX,
            show_help: true,
            markdown_cache: CommonMarkCache::default(),
        }
    }
}

impl Clone for RhaiScriptEditorState {
    fn clone(&self) -> Self {
        Self {
            current_script: self.current_script.clone(),
            theme: self.theme.clone(),
            show_help: self.show_help,
            markdown_cache: CommonMarkCache::default(),
        }
    }
}

pub fn pane_rhai_script_editor(ui: &mut egui::Ui, world: &mut World) -> NodeResult<()> {
    let mut state = world
        .get_resource_or_insert_with(RhaiScriptEditorState::default)
        .clone();
    let engine_res = world.resource::<RhaiEngineResource>();

    ui.horizontal(|ui| {
        ui.heading("Rhai Script Editor");
        ui.separator();

        if let Some(ref script) = state.current_script {
            if script.compiled.is_some() {
                ui.colored_label(egui::Color32::GREEN, "✓ Compiled");
            } else {
                ui.colored_label(egui::Color32::YELLOW, "⚠ Not Compiled");
            }
        }

        if ui.button("New Script").clicked() {
            state.current_script = Some(RhaiScript::new(
                next_id(),
                ScriptType::Value {
                    return_type: "i32".to_string(),
                },
            ));
        }

        ui.separator();

        ui.checkbox(&mut state.show_help, "Show Help");
    });

    ui.separator();

    if let Some(mut script) = state.current_script.clone() {
        ui.horizontal(|ui| {
            ui.label("Script Type:");
            egui::ComboBox::from_label("")
                .selected_text(format!("{:?}", script.script_type))
                .show_ui(ui, |ui| {
                    if ui
                        .selectable_label(
                            matches!(&script.script_type, ScriptType::Value { .. }),
                            "Value",
                        )
                        .clicked()
                    {
                        script.script_type = ScriptType::Value {
                            return_type: "i32".to_string(),
                        };
                        script.code = "value = 0".to_string();
                    }
                    if ui
                        .selectable_label(
                            matches!(&script.script_type, ScriptType::UnitAction),
                            "Unit Action",
                        )
                        .clicked()
                    {
                        script.script_type = ScriptType::UnitAction;
                        script.code = UNIT_ACTION_TEMPLATE.to_string();
                    }
                    if ui
                        .selectable_label(
                            matches!(&script.script_type, ScriptType::StatusAction),
                            "Status Action",
                        )
                        .clicked()
                    {
                        script.script_type = ScriptType::StatusAction;
                        script.code = STATUS_ACTION_TEMPLATE.to_string();
                    }
                    if ui
                        .selectable_label(
                            matches!(&script.script_type, ScriptType::AbilityAction),
                            "Ability Action",
                        )
                        .clicked()
                    {
                        script.script_type = ScriptType::AbilityAction;
                        script.code = ABILITY_ACTION_TEMPLATE.to_string();
                    }
                    if ui
                        .selectable_label(
                            matches!(&script.script_type, ScriptType::Painter),
                            "Painter",
                        )
                        .clicked()
                    {
                        script.script_type = ScriptType::Painter;
                        script.code = PAINTER_TEMPLATE.to_string();
                    }
                });

            ui.separator();

            if ui.button("Compile").clicked() {
                if let Ok(ast) = engine_res.compile_script(&script) {
                    let compiled = CompiledScript {
                        ast,
                        script_type: script.script_type.clone(),
                        last_compiled: std::time::Instant::now(),
                    };
                    engine_res.store_compiled(script.id, compiled.clone());
                    script.compiled = Some(compiled);
                }
            }
        });

        ui.separator();

        let available_height = ui.available_height();
        let editor_height = if state.show_help {
            available_height * 0.6
        } else {
            available_height
        };

        ui.group(|ui| {
            ui.set_min_height(editor_height);

            let syntax = create_rhai_syntax();

            let response = CodeEditor::default()
                .id_source(format!("rhai_editor_{}", script.id))
                .with_rows((editor_height / 14.0) as usize)
                .with_fontsize(14.0)
                .with_theme(state.theme)
                .with_syntax(syntax)
                .with_numlines(true)
                .show(ui, &mut script.code);

            if response.response.changed() {
                script.compiled = None;
            }
        });

        if state.show_help {
            ui.separator();

            egui::ScrollArea::vertical()
                .max_height(available_height * 0.35)
                .show(ui, |ui| {
                    let help_text = match &script.script_type {
                        ScriptType::Value { return_type } => {
                            VALUE_SCRIPT_HELP.replace("{TYPE}", return_type.as_str())
                        }
                        ScriptType::UnitAction => UNIT_ACTION_HELP.to_string(),
                        ScriptType::StatusAction => STATUS_ACTION_HELP.to_string(),
                        ScriptType::AbilityAction => ABILITY_ACTION_HELP.to_string(),
                        ScriptType::Painter => PAINTER_HELP.to_string(),
                    };

                    CommonMarkViewer::new().show(ui, &mut state.markdown_cache, &help_text);
                });
        }

        state.current_script = Some(script);
    } else {
        ui.centered_and_justified(|ui| {
            ui.label("No script loaded. Click 'New Script' to create one.");
        });
    }

    {
        let mut resource = world.get_resource_mut::<RhaiScriptEditorState>().unwrap();
        *resource = state;
    }

    Ok(())
}

fn create_rhai_syntax() -> Syntax {
    let mut syntax = Syntax::rust();

    // Clear default Rust keywords and add only Rhai-specific ones
    syntax.keywords.clear();

    // Add our custom types
    syntax.types.insert("Unit");
    syntax.types.insert("Status");
    syntax.types.insert("Ability");
    syntax.types.insert("Painter");
    syntax.types.insert("UnitState");
    syntax.types.insert("UnitStats");
    syntax.types.insert("StatusState");

    // Add our special variables
    syntax.special.insert("owner");
    syntax.special.insert("target");
    syntax.special.insert("status");
    syntax.special.insert("ability");
    syntax.special.insert("painter");
    syntax.special.insert("value");
    syntax.special.insert("x");
    syntax.special.insert("actions");

    // Note: egui_code_editor Syntax doesn't have literals field
    // The syntax highlighting will use the types and special fields we've already set

    syntax
}

// Script templates
const UNIT_ACTION_TEMPLATE: &str = r#"// Available: owner, target, x, unit_actions
// owner and target are Units
// x is the power/modifier value
// unit_actions is a vector to store actions

if owner.id > 0 {
    use_ability(unit_actions, "Shield", target.id);
}

if target.id > 0 {
    apply_status(unit_actions, "Poison", target.id, 2);
}"#;

const STATUS_ACTION_TEMPLATE: &str = r#"// Available: status, x, status_actions
// x is the stack count modifier
// status_actions is a vector to store actions

deal_damage(status_actions, 0, x * 2);

if x > 5 {
    heal_damage(status_actions, 0, x);
}"#;

const ABILITY_ACTION_TEMPLATE: &str = r#"// Available: ability, ability_actions
// ability is the current ability being used
// ability_actions is a vector to store actions

deal_damage(ability_actions, 0, 10);
change_status(ability_actions, "Shield", 0, 1);"#;

const PAINTER_TEMPLATE: &str = r#"// Available: painter
// Use painter to draw shapes

// Create painter actions
let actions = [];
actions.push(painter_circle(20.0));
actions.push(painter_color(255, 0, 0));
actions.push(painter_paint());

actions.push(painter_translate(50.0, 0.0));
actions.push(painter_rectangle(30.0, 30.0));
actions.push(painter_hollow(2.0));
actions.push(painter_color(0, 255, 0));
actions.push(painter_paint());"#;

// Help documentation
const VALUE_SCRIPT_HELP: &str = r#"# Value Script

Calculates and returns a value of type **{TYPE}**.

## Available Variables
- `value` - The initial value to modify (type: {TYPE})

## Example
```rhai
// Simple calculation
value = value * 2 + 1
```

## Common Functions
- `abs(x)` - Absolute value
- `min(a, b)` - Minimum value
- `max(a, b)` - Maximum value
- `clamp(x, min, max)` - Clamp value in range
- `random()` - Random number 0-99
- `random_range(min, max)` - Random in range
- `player_id()` - Current player's ID
"#;

const UNIT_ACTION_HELP: &str = r#"# Unit Action Script

Defines actions for a unit to perform during gameplay.

## Available Variables
- `owner` - The unit performing the action (type: Unit)
- `target` - The target unit (type: Unit)
- `x` - Power/modifier value (type: i64)
- `actions` - Vector to store actions

## Unit Properties
- `owner.id` - Unit's ID
- `owner.unit_name` - Unit's name
- `owner.state.dmg` - Damage taken
- `owner.state.stax` - Stack count
- `owner.stats.hp` - Max health
- `owner.stats.pwr` - Power

## Action Functions
- `use_ability(unit_actions, name, target_id)` - Add use ability action
- `apply_status(unit_actions, name, target_id, stacks)` - Add apply status action

## Example
```rhai
if owner.id > 0 {
    use_ability(unit_actions, "Heal", target.id);
}

if target.id > 0 {
    apply_status(unit_actions, "Weakness", target.id, 3);
}
```
"#;

const STATUS_ACTION_HELP: &str = r#"# Status Action Script

Defines actions for a status effect when triggered.

## Available Variables
- `status` - The status effect (type: Status)
- `x` - Stack count modifier (type: i64)
- `actions` - Vector to store actions

## Status Properties
- `status.id` - Status ID
- `status.status_name` - Status name
- `status.state.stax` - Current stacks

## Action Functions
- `deal_damage(status_actions, target_id, amount)` - Add deal damage action
- `heal_damage(status_actions, target_id, amount)` - Add heal damage action
- `use_ability(status_actions, name, target_id)` - Add use ability action
- `modify_stacks(status_actions, delta)` - Modify stacks

## Example
```rhai
// Deal damage based on stacks
deal_damage(status_actions, target_id, x * 3);

// Heal if stacks are high
if x > 5 {
    heal_damage(status_actions, owner_id, x * 2);
}

// Chain another ability
use_ability(status_actions, "Spread", target.id);
```
"#;

const ABILITY_ACTION_HELP: &str = r#"# Ability Action Script

Defines the effects of an ability when used.

## Available Variables
- `ability` - The ability being used (type: Ability)
- `actions` - Vector to store actions

## Ability Properties
- `ability.id` - Ability ID
- `ability.ability_name` - Ability name

## Action Functions
- `deal_damage(ability_actions, target_id, amount)` - Add deal damage action
- `heal_damage(ability_actions, target_id, amount)` - Add heal damage action
- `change_status(ability_actions, name, target_id, delta)` - Add change status action

## Example
```rhai
// Basic damage ability
deal_damage(ability_actions, target.id, 15);

// Apply shield to self
change_status(ability_actions, "Shield", 0, 2);

// Complex combo
deal_damage(ability_actions, target.id, 10);
heal_damage(ability_actions, 0, 5);
change_status(ability_actions, "Burn", target.id, 3);
```
"#;

const PAINTER_HELP: &str = r#"# Painter Script

Draw custom graphics and visual effects.

## Available Variables
- `painter` - The painter object for drawing

## Painter Functions

### Shape Drawing
- `painter_circle(radius)` - Create circle action
- `painter_rectangle(width, height)` - Create rectangle action
- `painter_text(string)` - Create text action

### Style Modifiers
- `painter_hollow(thickness)` - Create hollow modifier
- `painter_color(r, g, b)` - Create color modifier (0-255)
- `painter_alpha(value)` - Create alpha modifier (0.0-1.0)

### Transformations
- `painter_translate(x, y)` - Create translate action
- `painter_rotate(angle)` - Create rotate action (radians)
- `painter_scale(factor)` - Create scale action

### Execution
- `painter_paint()` - Create paint execution action

## Color Helpers
- `rgb(r, g, b)` - Create RGB color array
- `rgba(r, g, b, a)` - Create RGBA color array
- `hsv_to_rgb(h, s, v)` - Convert HSV to RGB

## Example
```rhai
// Draw a red circle
painter_circle(painter_actions, 25.0);
painter_color(painter_actions, 255, 0, 0);
painter_paint(painter_actions);

// Draw hollow green square
painter_translate(painter_actions, 50.0, 0.0);
painter_rectangle(painter_actions, 40.0, 40.0);
painter_hollow(painter_actions, 3.0);
painter_color(painter_actions, 0, 255, 0);
painter_alpha(painter_actions, 0.7);
painter_paint(painter_actions);

// Draw rotated text
painter_translate(painter_actions, 100.0, 50.0);
painter_rotate(painter_actions, 0.785);
painter_text(painter_actions, "Hello!");
painter_color(painter_actions, 255, 255, 255);
painter_paint(painter_actions);
```

## Math Helpers
- `vec2(x, y)` - Create 2D vector
- `distance(x1, y1, x2, y2)` - Calculate distance
- `normalize(x, y)` - Normalize vector
- `lerp(a, b, t)` - Linear interpolation
"#;
