use std::sync::{Arc, RwLock};

use super::*;

static AI_PROMPT: &str = r#"You are a game script generator for Arena of Ideas, a tactical auto-battler.
Write Rhai scripts for unit behaviors. Available APIs:

UNIT BEHAVIOR SCRIPTS (variable: unit_actions):
- unit_actions.apply_status("House/Status", target_id, stacks) - Apply a status effect
- unit_actions.use_ability("House/Ability", target_id) - Use an ability

STATUS BEHAVIOR SCRIPTS (variable: status_actions):
- status_actions.deal_damage(amount) - Deal damage
- status_actions.heal_damage(amount) - Heal
- status_actions.set_stax(amount) - Set status stacks
- status_actions.use_ability("House/Ability", target_id) - Use ability

ABILITY EFFECT SCRIPTS (variable: ability_actions):
- ability_actions.deal_damage(amount) - Deal damage
- ability_actions.heal_damage(amount) - Heal
- ability_actions.change_status("House/Status", stacks_delta) - Modify status stacks

Available variables: owner (Unit), target (Unit), x (status stacks), value (modifiable damage)
Unit properties: .id, .hp, .pwr, .dmg
Context: ctx.get_enemies(unit_id), ctx.get_allies(unit_id), ctx.get_all_units()

Use placeholder names like "House/Status" or "House/Ability" for references.
Return ONLY the script code, no explanation, no markdown fences."#;

#[derive(Default)]
pub struct AiGenState {
    pub prompt: String,
    pub result: Arc<RwLock<Option<String>>>,
    pub loading: Arc<RwLock<bool>>,
    pub error: Arc<RwLock<Option<String>>>,
}

pub fn render_ai_gen(kind: ContentNodeKind, state: &mut AiGenState, ui: &mut Ui) -> Option<String> {
    let type_hint = match kind {
        ContentNodeKind::NUnitBehavior => "unit behavior",
        ContentNodeKind::NStatusBehavior => "status effect",
        ContentNodeKind::NAbilityEffect => "ability effect",
        _ => return None,
    };

    ui.horizontal(|ui| {
        ui.label("Describe:");
        ui.text_edit_singleline(&mut state.prompt);
    });

    let is_loading = *state.loading.read().unwrap();

    if is_loading {
        ui.spinner();
        ui.label("Generating...");
    } else if !state.prompt.is_empty()
        && ui.button(format!("Generate {type_hint} with AI")).clicked()
    {
        let api_key = std::env::var("ANTHROPIC_API_KEY").ok();

        if let Some(key) = api_key {
            *state.loading.write().unwrap() = true;
            *state.error.write().unwrap() = None;
            *state.result.write().unwrap() = None;

            let prompt = state.prompt.clone();
            let result = state.result.clone();
            let loading = state.loading.clone();
            let error = state.error.clone();
            let user_msg = format!(
                "{}\n\nGenerate a {} script for: {}",
                AI_PROMPT, type_hint, prompt
            );

            // Build JSON body manually to avoid serde_json dependency
            let body = format!(
                r#"{{"model":"claude-sonnet-4-20250514","max_tokens":512,"messages":[{{"role":"user","content":{}}}]}}"#,
                json_escape_string(&user_msg)
            );

            std::thread::spawn(move || {
                let request = ehttp::Request {
                    method: "POST".to_string(),
                    url: "https://api.anthropic.com/v1/messages".to_string(),
                    body: body.into_bytes(),
                    headers: ehttp::Headers::new(&[
                        ("content-type", "application/json"),
                        ("x-api-key", &key),
                        ("anthropic-version", "2023-06-01"),
                    ]),
                };

                ehttp::fetch(request, move |response| {
                    *loading.write().unwrap() = false;
                    match response {
                        Ok(resp) => {
                            if let Some(text) = resp.text() {
                                if let Some(content) = extract_json_text(text) {
                                    let code = content
                                        .trim()
                                        .trim_start_matches("```rhai")
                                        .trim_start_matches("```")
                                        .trim_end_matches("```")
                                        .trim()
                                        .to_string();
                                    *result.write().unwrap() = Some(code);
                                } else if let Some(err) = extract_json_error(text) {
                                    *error.write().unwrap() = Some(err);
                                } else {
                                    *error.write().unwrap() = Some(format!(
                                        "Unexpected response: {}",
                                        &text[..200.min(text.len())]
                                    ));
                                }
                            }
                        }
                        Err(e) => {
                            *error.write().unwrap() = Some(e);
                        }
                    }
                });
            });
        } else {
            *state.error.write().unwrap() =
                Some("Set ANTHROPIC_API_KEY env var to use AI generation".to_string());
        }
    }

    if let Some(ref err) = *state.error.read().unwrap() {
        ui.colored_label(egui::Color32::RED, format!("AI error: {err}"));
    }

    let result = state.result.write().unwrap().take();
    if result.is_some() {
        ui.colored_label(egui::Color32::GREEN, "AI script applied!");
    }
    ui.separator();

    result
}

fn json_escape_string(s: &str) -> String {
    let mut out = String::with_capacity(s.len() + 2);
    out.push('"');
    for c in s.chars() {
        match c {
            '"' => out.push_str("\\\""),
            '\\' => out.push_str("\\\\"),
            '\n' => out.push_str("\\n"),
            '\r' => out.push_str("\\r"),
            '\t' => out.push_str("\\t"),
            c => out.push(c),
        }
    }
    out.push('"');
    out
}

/// Extract the text content from Claude API response JSON
fn extract_json_text(json: &str) -> Option<String> {
    // Look for "text":"..." in the response
    let marker = "\"text\":\"";
    let start = json.find(marker)? + marker.len();
    let rest = &json[start..];
    // Find the closing quote, handling escapes
    let mut chars = rest.chars();
    let mut result = String::new();
    while let Some(c) = chars.next() {
        match c {
            '\\' => {
                if let Some(escaped) = chars.next() {
                    match escaped {
                        'n' => result.push('\n'),
                        'r' => result.push('\r'),
                        't' => result.push('\t'),
                        '"' => result.push('"'),
                        '\\' => result.push('\\'),
                        _ => {
                            result.push('\\');
                            result.push(escaped);
                        }
                    }
                }
            }
            '"' => return Some(result),
            _ => result.push(c),
        }
    }
    None
}

/// Extract error message from Claude API error response
fn extract_json_error(json: &str) -> Option<String> {
    let marker = "\"message\":\"";
    let start = json.find(marker)? + marker.len();
    let rest = &json[start..];
    let end = rest.find('"')?;
    Some(rest[..end].to_string())
}
