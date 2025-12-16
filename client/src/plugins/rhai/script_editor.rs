use super::*;
use crate::ui::render::features::FEdit;
use egui_code_editor::{CodeEditor, ColorTheme, Syntax};
use schema::RhaiScript;

#[derive(Clone, Resource)]
pub struct ScriptEditorState {
    pub code: String,
    pub theme: ColorTheme,
    pub show_help: bool,
}

impl Default for ScriptEditorState {
    fn default() -> Self {
        Self {
            code: String::new(),
            theme: ColorTheme::GRUVBOX,
            show_help: false,
        }
    }
}

pub fn pane_rhai_script_editor(ui: &mut egui::Ui, world: &mut World) -> NodeResult<()> {
    let mut editor_state = world
        .get_resource_or_insert_with(ScriptEditorState::default)
        .clone();

    ui.horizontal(|ui| {
        ui.heading("Rhai Script Editor");
        ui.separator();
        if !editor_state.code.is_empty() {
            ui.colored_label(egui::Color32::GREEN, "✓ Script");
        }
    });

    ui.separator();
    ui.checkbox(&mut editor_state.show_help, "Show Help");

    let available_height = ui.available_height();
    let editor_height = if editor_state.show_help {
        available_height * 0.6
    } else {
        available_height
    };

    ui.group(|ui| {
        ui.set_min_height(editor_height);
        let syntax = create_rhai_syntax();

        CodeEditor::default()
            .id_source("rhai_editor")
            .with_rows((editor_height / 14.0) as usize)
            .with_fontsize(14.0)
            .with_theme(editor_state.theme.clone())
            .with_syntax(syntax)
            .with_numlines(true)
            .show(ui, &mut editor_state.code);
    });

    if editor_state.show_help {
        ui.separator();
        egui::ScrollArea::vertical()
            .max_height(available_height * 0.35)
            .show(ui, |ui| {
                ui.label("Rhai Script Help");
                ui.separator();
                ui.vertical(|ui| {
                    ui.label("Unit Actions:");
                    ui.code("unit_actions.use_ability(ability_name, target_id)");
                    ui.code("unit_actions.apply_status(status_name, target_id, stacks)");
                    ui.separator();
                    ui.label("Status Actions:");
                    ui.code("status_actions.deal_damage(target_id, amount)");
                    ui.code("status_actions.heal_damage(target_id, amount)");
                    ui.code("status_actions.use_ability(ability_name, target_id)");
                    ui.code("status_actions.modify_stacks(delta)");
                    ui.separator();
                    ui.label("Ability Actions:");
                    ui.code("ability_actions.deal_damage(target_id, amount)");
                    ui.code("ability_actions.heal_damage(target_id, amount)");
                    ui.code("ability_actions.change_status(status_name, target_id, delta)");
                });
            });
    }
    world.insert_resource(editor_state);

    Ok(())
}

fn create_rhai_syntax() -> Syntax {
    let mut syntax = Syntax::rust();
    syntax.keywords.clear();

    syntax.types.insert("Unit");
    syntax.types.insert("Status");
    syntax.types.insert("Ability");

    syntax.special.insert("owner");
    syntax.special.insert("target");
    syntax.special.insert("status");
    syntax.special.insert("ability");
    syntax.special.insert("unit_actions");
    syntax.special.insert("status_actions");
    syntax.special.insert("ability_actions");
    syntax.special.insert("painter");
    syntax.special.insert("x");

    syntax
}

pub fn show_rhai_script_editor<T: schema::ScriptAction>(
    script: &mut RhaiScript<T>,
    ui: &mut egui::Ui,
) -> Response {
    ui.vertical(|ui| {
        ui.horizontal(|ui| {
            ui.label("Description:");
            ui.text_edit_singleline(&mut script.description);
        });

        ui.label("Script Code:");
        let syntax = create_rhai_syntax();
        let editor_height = 300.0;

        ui.group(|ui| {
            ui.set_min_height(editor_height);
            CodeEditor::default()
                .id_source("rhai_script_editor")
                .with_rows((editor_height / 14.0) as usize)
                .with_fontsize(13.0)
                .with_theme(ColorTheme::GRUVBOX)
                .with_syntax(syntax)
                .with_numlines(true)
                .show(ui, &mut script.code);
        });

        if !script.code.is_empty() {
            ui.colored_label(egui::Color32::GREEN, "✓ Script compiled");
        } else {
            ui.colored_label(egui::Color32::YELLOW, "○ Empty script");
        }
    })
    .response
}

impl<T: schema::ScriptAction> FEdit for RhaiScript<T> {
    fn edit(&mut self, ui: &mut Ui, _ctx: &ClientContext) -> Response {
        show_rhai_script_editor(self, ui)
    }
}
