use super::*;
use ::rhai::Engine;

pub fn register_common_functions(engine: &mut Engine) {
    // Math functions
    engine
        .register_fn("abs", |x: i64| x.abs())
        .register_fn("min", |a: i64, b: i64| a.min(b))
        .register_fn("max", |a: i64, b: i64| a.max(b))
        .register_fn("clamp", |x: i64, min: i64, max: i64| x.clamp(min, max))
        .register_fn("lerp", |a: f64, b: f64, t: f64| a + (b - a) * t);

    // Random functions
    engine
        .register_fn("random", || {
            use rand::Rng;
            rand::rng().random_range(0i64..100i64)
        })
        .register_fn("random_range", |min: i64, max: i64| {
            use rand::Rng;
            rand::rng().random_range(min..max)
        })
        .register_fn("random_float", || {
            use rand::Rng;
            rand::rng().random_range(0.0f64..1.0f64)
        });

    // Color helpers
    engine
        .register_fn("rgb", |r: i64, g: i64, b: i64| vec![r, g, b, 255i64])
        .register_fn("rgba", |r: i64, g: i64, b: i64, a: i64| vec![r, g, b, a])
        .register_fn("hsv_to_rgb", |h: f64, s: f64, v: f64| {
            let c = v * s;
            let x = c * (1.0 - ((h / 60.0) % 2.0 - 1.0).abs());
            let m = v - c;

            let (r, g, b) = if h < 60.0 {
                (c, x, 0.0)
            } else if h < 120.0 {
                (x, c, 0.0)
            } else if h < 180.0 {
                (0.0, c, x)
            } else if h < 240.0 {
                (0.0, x, c)
            } else if h < 300.0 {
                (x, 0.0, c)
            } else {
                (c, 0.0, x)
            };

            vec![
                ((r + m) * 255.0) as i64,
                ((g + m) * 255.0) as i64,
                ((b + m) * 255.0) as i64,
                255i64,
            ]
        });

    // Vector math helpers
    engine
        .register_fn("vec2", |x: f64, y: f64| vec![x, y])
        .register_fn("vec3", |x: f64, y: f64, z: f64| vec![x, y, z])
        .register_fn("distance", |x1: f64, y1: f64, x2: f64, y2: f64| {
            ((x2 - x1).powi(2) + (y2 - y1).powi(2)).sqrt()
        })
        .register_fn("normalize", |x: f64, y: f64| {
            let len = (x * x + y * y).sqrt();
            if len > 0.0 {
                vec![x / len, y / len]
            } else {
                vec![0.0, 0.0]
            }
        })
        .register_fn("dot", |x1: f64, y1: f64, x2: f64, y2: f64| {
            x1 * x2 + y1 * y2
        });

    // Game-specific constants
    engine.register_fn("player_id", || player_id());

    // Logging functions
    engine
        .register_fn("log", |msg: &str| {
            log::info!("Rhai Script: {}", msg);
        })
        .register_fn("debug", |msg: &str| {
            log::debug!("Rhai Script: {}", msg);
        });
}

pub fn create_autocomplete_items() -> Vec<AutocompleteItem> {
    vec![
        // Unit variables and properties
        AutocompleteItem::new("owner", "Unit", "The owner unit"),
        AutocompleteItem::new("target", "Unit", "The target unit"),
        AutocompleteItem::new("unit_actions", "Vec<UnitAction>", "Unit action list"),
        AutocompleteItem::new(".unit_name", "String", "Unit's name"),
        AutocompleteItem::new(".id", "u64", "Unit's ID"),
        // Status variables and properties
        AutocompleteItem::new("status", "Status", "The status object"),
        AutocompleteItem::new("status_actions", "Vec<StatusAction>", "Status action list"),
        AutocompleteItem::new(".status_name", "String", "Status name"),
        AutocompleteItem::new(".id", "u64", "Status ID"),
        // Ability variables and properties
        AutocompleteItem::new("ability", "Ability", "The ability object"),
        AutocompleteItem::new(
            "ability_actions",
            "Vec<AbilityAction>",
            "Ability action list",
        ),
        AutocompleteItem::new(".ability_name", "String", "Ability name"),
        AutocompleteItem::new(".id", "u64", "Ability ID"),
        // Action functions
        AutocompleteItem::new("use_ability()", "void", "Add use ability action"),
        AutocompleteItem::new("apply_status()", "void", "Add apply status action"),
        AutocompleteItem::new("deal_damage()", "void", "Add deal damage action"),
        AutocompleteItem::new("heal_damage()", "void", "Add heal damage action"),
        AutocompleteItem::new("change_status()", "void", "Add change status action"),
        AutocompleteItem::new("modify_stacks()", "void", "Modify status stacks"),
        // Painter functions
        AutocompleteItem::new("painter.circle()", "void", "Draw circle"),
        AutocompleteItem::new("painter.rectangle()", "void", "Draw rectangle"),
        AutocompleteItem::new("painter.text()", "void", "Draw text"),
        AutocompleteItem::new("painter.translate()", "void", "Translate painter"),
        AutocompleteItem::new("painter.rotate()", "void", "Rotate painter"),
        AutocompleteItem::new("painter.scale_mesh()", "void", "Scale mesh"),
        AutocompleteItem::new("painter.scale_rect()", "void", "Scale rect"),
        AutocompleteItem::new("painter.color()", "void", "Set color"),
        AutocompleteItem::new("painter.alpha()", "void", "Set alpha"),
        AutocompleteItem::new("painter.hollow()", "void", "Set hollow"),
        // Common functions
        AutocompleteItem::new("random()", "u64", "Random number 0-99"),
        AutocompleteItem::new("random_range()", "u64", "Random number in range"),
        AutocompleteItem::new("random_float()", "f64", "Random float 0-1"),
        AutocompleteItem::new("abs()", "i64", "Absolute value"),
        AutocompleteItem::new("min()", "i64", "Minimum of two values"),
        AutocompleteItem::new("max()", "i64", "Maximum of two values"),
        AutocompleteItem::new("clamp()", "i64", "Clamp value between min and max"),
        AutocompleteItem::new("lerp()", "f64", "Linear interpolation"),
        AutocompleteItem::new("distance()", "f64", "Distance between points"),
        AutocompleteItem::new("normalize()", "Vec", "Normalize vector"),
        AutocompleteItem::new("dot()", "f64", "Dot product"),
        AutocompleteItem::new("vec2()", "Vec", "Create 2D vector"),
        AutocompleteItem::new("vec3()", "Vec", "Create 3D vector"),
        AutocompleteItem::new("rgb()", "Vec", "Create RGB color"),
        AutocompleteItem::new("rgba()", "Vec", "Create RGBA color"),
        AutocompleteItem::new("hsv_to_rgb()", "Vec", "Convert HSV to RGB"),
        AutocompleteItem::new("player_id()", "i64", "Current player's ID"),
        AutocompleteItem::new("log()", "void", "Log message"),
        AutocompleteItem::new("debug()", "void", "Debug message"),
        // Variables
        AutocompleteItem::new("value", "any", "The value to calculate/modify"),
        AutocompleteItem::new("x", "i64", "Power/stack modifier variable"),
        AutocompleteItem::new("actions", "Vec", "Actions to execute"),
        // Context methods
        AutocompleteItem::new("ctx.get_all_units()", "Vec<i64>", "Get all units in battle"),
        AutocompleteItem::new("ctx.get_enemies()", "Vec<i64>", "Get enemy unit IDs"),
        AutocompleteItem::new("ctx.get_allies()", "Vec<i64>", "Get ally unit IDs"),
        AutocompleteItem::new(
            "ctx.get_adjacent_left()",
            "Option<i64>",
            "Get left adjacent unit ID",
        ),
        AutocompleteItem::new(
            "ctx.get_adjacent_right()",
            "Option<i64>",
            "Get right adjacent unit ID",
        ),
        AutocompleteItem::new(
            "ctx.get_adjacent_allies()",
            "Vec<i64>",
            "Get adjacent ally IDs",
        ),
        AutocompleteItem::new("ctx.load_unit()", "Option<Unit>", "Load unit by ID"),
        AutocompleteItem::new("ctx.load_status()", "Option<Status>", "Load status by ID"),
        AutocompleteItem::new(
            "ctx.load_ability()",
            "Option<Ability>",
            "Load ability by ID",
        ),
        AutocompleteItem::new("ctx.load_house()", "Option<House>", "Load house by ID"),
        AutocompleteItem::new("vec.random()", "T?", "Get random element from vector"),
    ]
}

pub struct AutocompleteItem {
    pub name: String,
    pub type_hint: String,
    pub description: String,
}

impl AutocompleteItem {
    pub fn new(
        name: impl Into<String>,
        type_hint: impl Into<String>,
        description: impl Into<String>,
    ) -> Self {
        Self {
            name: name.into(),
            type_hint: type_hint.into(),
            description: description.into(),
        }
    }
}

pub fn register_vec_extensions(engine: &mut Engine) {
    engine
        .register_fn("random", |vec: Vec<i64>| -> Option<i64> {
            use rand::seq::SliceRandom;
            let mut v = vec;
            v.as_mut_slice().shuffle(&mut rand::rng());
            v.first().copied()
        })
        .register_fn("random", |vec: Vec<NUnit>| -> Option<NUnit> {
            use rand::seq::SliceRandom;
            let mut v = vec;
            v.as_mut_slice().shuffle(&mut rand::rng());
            v.first().cloned()
        })
        .register_fn("random", |vec: Vec<NStatusMagic>| -> Option<NStatusMagic> {
            use rand::seq::SliceRandom;
            let mut v = vec;
            v.as_mut_slice().shuffle(&mut rand::rng());
            v.first().cloned()
        })
        .register_fn(
            "random",
            |vec: Vec<NAbilityMagic>| -> Option<NAbilityMagic> {
                use rand::seq::SliceRandom;
                let mut v = vec;
                v.as_mut_slice().shuffle(&mut rand::rng());
                v.first().cloned()
            },
        );
}
