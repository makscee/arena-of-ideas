use super::*;
use crate::ui::render::features::FEdit;
use egui_code_editor::{CodeEditor, ColorTheme, Completer, Syntax};
use schema::RhaiScript;

#[derive(Clone, Resource)]
pub struct ScriptEditorState {
    pub code: String,
    pub theme: ColorTheme,
    pub show_help: bool,
    pub auto_compile: bool,
    pub compile_error: Option<String>,
}

impl Default for ScriptEditorState {
    fn default() -> Self {
        Self {
            code: String::new(),
            theme: ColorTheme::GRUVBOX,
            show_help: false,
            auto_compile: false,
            compile_error: None,
        }
    }
}

static COMPLETER: OnceCell<Mutex<Completer>> = OnceCell::new();
pub fn init_completer() {
    COMPLETER
        .set(Mutex::new(
            Completer::new_with_syntax(&rhai_syntax()).with_user_words(),
        ))
        .unwrap();
}

pub trait CompleterHelper {
    fn push_completer(&self) -> &Self;
}

impl CompleterHelper for str {
    fn push_completer(&self) -> &Self {
        rhai_completer().push_word(self);
        self
    }
}

pub fn rhai_completer() -> MutexGuard<'static, Completer> {
    COMPLETER
        .get_or_init(|| Mutex::new(Completer::new_with_syntax(&rhai_syntax()).with_user_words()))
        .lock()
}

fn rhai_syntax() -> Syntax {
    static SYNTAX: OnceCell<Syntax> = OnceCell::new();

    SYNTAX
        .get_or_init(|| {
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
        })
        .clone()
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
        let syntax = rhai_syntax();
        let editor_height = 300.0;
        let mut saved = false;
        let response = ui
            .horizontal(|ui| {
                if ui.button("Save & Compile (Cmd+S)").clicked()
                    || ui.input(|i| i.key_pressed(egui::Key::S) && i.modifiers.command)
                {
                    script.clear_compiled();
                    script.compile_error.write().unwrap().take();
                    saved = true;
                }

                if let Some(ref error) = *script.compile_error.read().unwrap() {
                    ui.colored_label(egui::Color32::RED, "❌ Compilation Error:");
                    ui.label(RichText::new(error).monospace());
                } else if let Some(ref error) = *script.run_error.read().unwrap() {
                    ui.colored_label(egui::Color32::RED, "❌ Runtime Error:");
                    ui.label(RichText::new(error).monospace());
                } else {
                    ui.colored_label(egui::Color32::GREEN, "✅ Script compiled");
                }
            })
            .response;

        let mut editor_response = ui
            .group(|ui| {
                ui.set_min_height(editor_height);
                CodeEditor::default()
                    .id_source("rhai_script_editor")
                    .with_rows((editor_height / 14.0) as usize)
                    .with_fontsize(13.0)
                    .with_theme(ColorTheme::SONOKAI)
                    .with_syntax(syntax)
                    .with_numlines(true)
                    .show_with_completer(ui, &mut script.code, &mut rhai_completer())
                    .response
            })
            .inner;
        if saved {
            editor_response.mark_changed();
            editor_response
        } else {
            response
        }
    })
    .inner
}

impl<T: schema::ScriptAction> FEdit for RhaiScript<T> {
    fn edit(&mut self, ui: &mut Ui, _ctx: &ClientContext) -> Response {
        show_rhai_script_editor(self, ui)
    }
}
