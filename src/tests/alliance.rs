use super::*;

pub fn test_alliances(assets: &Assets) {
    println!("Testings alliance effects...");
    for (name, template) in assets.units.iter() {
        println!("--- Testing Unit {name} ---");
        let max_level = if template.alliances.is_empty() { 0 } else { 6 };
        for level in 0..=max_level {
            let mut unit = template.clone();
            for alliance in &template.alliances {
                alliance.apply(&mut unit, level);
            }

            println!("^- Unit: {name}, alliance level: {level}");
            let unit =
                serde_json::to_string_pretty(&unit).expect("Failed to serialize unit template");
            println!("{unit}\n");
        }
    }
}
