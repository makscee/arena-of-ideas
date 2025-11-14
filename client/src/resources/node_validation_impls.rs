use super::*;

/// Trait for node validation functionality
pub trait NodeValidation {
    fn validate(&self) -> bool;
}

impl NodeValidation for NUnitDescription {
    fn validate(&self) -> bool {
        validate_unit_description(self)
    }
}

impl NodeValidation for NUnitBehavior {
    fn validate(&self) -> bool {
        validate_unit_behavior(self)
    }
}

impl NUnitDescription {
    /// Gets the processed description with macros replaced
    pub fn get_processed_description(&self, context: &ClientContext) -> Result<String, NodeError> {
        replace_description_macros(&self.description, self.magic_type, self.trigger, context)
    }
}

/// Validates that unit descriptions have proper macro usage based on magic type
pub fn validate_unit_description(description: &NUnitDescription) -> bool {
    let text = &description.description;
    match description.magic_type {
        MagicType::Ability => {
            // For ability magic, description should have %trigger and %ability macros
            text.contains("%trigger") && text.contains("%ability")
        }
        MagicType::Status => {
            // For status magic, description should have %trigger and %status macros
            text.contains("%trigger") && text.contains("%status")
        }
    }
}

/// Validates that unit behavior has appropriate actions for its magic type
pub fn validate_unit_behavior(behavior: &NUnitBehavior) -> bool {
    let has_use_ability = behavior
        .reaction
        .actions
        .iter()
        .any(|action| matches!(action, Action::use_ability));

    let has_apply_status = behavior
        .reaction
        .actions
        .iter()
        .any(|action| matches!(action, Action::apply_status));

    match behavior.magic_type {
        MagicType::Ability => {
            // For ability magic, should have use_ability action and no apply_status
            has_use_ability && !has_apply_status
        }
        MagicType::Status => {
            // For status magic, should have apply_status action and no use_ability
            has_apply_status && !has_use_ability
        }
    }
}

/// Replaces macros in description text with actual names from the context
pub fn replace_description_macros(
    description: &str,
    magic_type: MagicType,
    trigger: Trigger,
    context: &ClientContext,
) -> Result<String, NodeError> {
    let mut result = description.to_string();

    // Replace %trigger macro with trigger name
    if result.contains("%trigger") {
        let trigger_name = trigger.as_ref();
        result = result.replace("%trigger", trigger_name);
    }

    // Replace magic-specific macros
    match magic_type {
        MagicType::Ability => {
            if result.contains("%ability") {
                // Try to get the ability name from the context
                if let Ok(owner_id) = context.owner() {
                    if let Ok(house) = context.load_first_parent_recursive_ref::<NHouse>(owner_id) {
                        if let Ok(ability) = house.ability_ref(context) {
                            result = result.replace("%ability", &ability.ability_name);
                        } else {
                            result = result.replace("%ability", "[Unknown Ability]");
                        }
                    } else {
                        result = result.replace("%ability", "[No House]");
                    }
                } else {
                    result = result.replace("%ability", "[No Owner]");
                }
            }
        }
        MagicType::Status => {
            if result.contains("%status") {
                // Try to get the status name from the context
                if let Ok(owner_id) = context.owner() {
                    if let Ok(house) = context.load_first_parent_recursive_ref::<NHouse>(owner_id) {
                        if let Ok(status) = house.status_ref(context) {
                            result = result.replace("%status", &status.status_name);
                        } else {
                            result = result.replace("%status", "[Unknown Status]");
                        }
                    } else {
                        result = result.replace("%status", "[No House]");
                    }
                } else {
                    result = result.replace("%status", "[No Owner]");
                }
            }
        }
    }

    Ok(result)
}
