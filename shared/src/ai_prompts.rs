/// AI prompt building and response parsing logic.
/// Shared so both server and tests can use it.

use serde::{Deserialize, Serialize};

/// Parsed response from Claude for ability breeding.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AbilityGenResponse {
    pub name: String,
    pub description: String,
    pub target_type: String,
    pub effect_script: String,
}

/// Parsed response from Claude for unit generation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UnitGenResponse {
    pub name: String,
    pub painter_script: String,
}

/// Parse a JSON response from Claude into an AbilityGenResponse.
pub fn parse_ability_response(json: &str) -> Result<AbilityGenResponse, String> {
    // Strip markdown code fences if present
    let cleaned = strip_code_fences(json);

    let response: AbilityGenResponse =
        serde_json::from_str(&cleaned).map_err(|e| format!("Failed to parse ability response: {}", e))?;

    // Validate required fields
    if response.name.is_empty() {
        return Err("Ability name cannot be empty".to_string());
    }
    if response.effect_script.is_empty() {
        return Err("Effect script cannot be empty".to_string());
    }

    // Validate target_type
    let valid_targets = [
        "RandomEnemy", "AllEnemies", "RandomAlly", "AllAllies", "Owner", "All",
    ];
    if !valid_targets.contains(&response.target_type.as_str()) {
        return Err(format!("Invalid target_type: {}", response.target_type));
    }

    Ok(response)
}

/// Parse a JSON response from Claude into a UnitGenResponse.
pub fn parse_unit_response(json: &str) -> Result<UnitGenResponse, String> {
    let cleaned = strip_code_fences(json);

    let response: UnitGenResponse =
        serde_json::from_str(&cleaned).map_err(|e| format!("Failed to parse unit response: {}", e))?;

    if response.name.is_empty() {
        return Err("Unit name cannot be empty".to_string());
    }

    Ok(response)
}

/// Strip markdown code fences (```json ... ```) from Claude responses.
fn strip_code_fences(input: &str) -> String {
    let trimmed = input.trim();
    if trimmed.starts_with("```") {
        let without_start = if let Some(pos) = trimmed.find('\n') {
            &trimmed[pos + 1..]
        } else {
            trimmed
        };
        if let Some(pos) = without_start.rfind("```") {
            return without_start[..pos].trim().to_string();
        }
    }
    trimmed.to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_valid_ability_response() {
        let json = r#"{"name": "Ember Heist", "description": "Steals power while dealing fire damage", "target_type": "RandomEnemy", "effect_script": "deal_damage(target_id, X * level);"}"#;

        let result = parse_ability_response(json).unwrap();
        assert_eq!(result.name, "Ember Heist");
        assert_eq!(result.target_type, "RandomEnemy");
        assert!(result.effect_script.contains("deal_damage"));
    }

    #[test]
    fn parse_ability_with_code_fences() {
        let json = "```json\n{\"name\": \"Frostbite\", \"description\": \"Freezes enemy\", \"target_type\": \"RandomEnemy\", \"effect_script\": \"change_stat(target_id, pwr, -X);\"}\n```";

        let result = parse_ability_response(json).unwrap();
        assert_eq!(result.name, "Frostbite");
    }

    #[test]
    fn parse_ability_empty_name_fails() {
        let json = r#"{"name": "", "description": "x", "target_type": "RandomEnemy", "effect_script": "x"}"#;
        assert!(parse_ability_response(json).is_err());
    }

    #[test]
    fn parse_ability_empty_script_fails() {
        let json = r#"{"name": "Test", "description": "x", "target_type": "RandomEnemy", "effect_script": ""}"#;
        assert!(parse_ability_response(json).is_err());
    }

    #[test]
    fn parse_ability_invalid_target_fails() {
        let json = r#"{"name": "Test", "description": "x", "target_type": "InvalidTarget", "effect_script": "x"}"#;
        assert!(parse_ability_response(json).is_err());
    }

    #[test]
    fn parse_ability_invalid_json_fails() {
        assert!(parse_ability_response("not json at all").is_err());
    }

    #[test]
    fn parse_valid_unit_response() {
        let json = r##"{"name": "Cinderpaw", "painter_script": "painter.circle(25.0, \"#ff4444\");"}"##;

        let result = parse_unit_response(json).unwrap();
        assert_eq!(result.name, "Cinderpaw");
        assert!(result.painter_script.contains("circle"));
    }

    #[test]
    fn parse_unit_empty_name_fails() {
        let json = r#"{"name": "", "painter_script": "x"}"#;
        assert!(parse_unit_response(json).is_err());
    }

    #[test]
    fn strip_code_fences_works() {
        assert_eq!(strip_code_fences("```json\n{}\n```"), "{}");
        assert_eq!(strip_code_fences("```\n{}\n```"), "{}");
        assert_eq!(strip_code_fences("{}"), "{}");
        assert_eq!(strip_code_fences("  {}  "), "{}");
    }

    #[test]
    fn parse_all_valid_target_types() {
        for target in ["RandomEnemy", "AllEnemies", "RandomAlly", "AllAllies", "Owner", "All"] {
            let json = format!(
                r#"{{"name": "T", "description": "d", "target_type": "{}", "effect_script": "x"}}"#,
                target
            );
            assert!(parse_ability_response(&json).is_ok(), "Should accept target: {}", target);
        }
    }
}
