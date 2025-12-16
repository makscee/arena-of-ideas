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
        .register_fn("gt", || gt().play_head())
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
