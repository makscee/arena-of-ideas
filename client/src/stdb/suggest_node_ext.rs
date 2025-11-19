use super::*;

pub trait ContentSuggestNodeExt {
    fn content_suggest_node(&self, kind: String, name: String) -> spacetimedb_sdk::Result<()>;
}

impl ContentSuggestNodeExt for RemoteReducers {
    fn content_suggest_node(&self, kind: String, name: String) -> spacetimedb_sdk::Result<()> {
        match kind.as_str() {
            "NUnit" => self.content_suggest_unit_name(name),
            "NHouse" => self.content_suggest_house_name(name),
            "NAbilityMagic" => self.content_suggest_house_name(name),
            "NStatusMagic" => self.content_suggest_house_name(name),
            _ => Err(spacetimedb_sdk::Error::Internal(
                format!("Unknown node kind: {}", kind).into(),
            )),
        }
    }
}
