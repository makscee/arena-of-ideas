use super::*;
use ::rhai::{Array, Dynamic, Engine};

fn array_to_vec2(arr: &Array) -> Option<(f32, f32)> {
    if arr.len() < 2 {
        return None;
    }
    let x = arr[0].as_float().ok()?;
    let y = arr[1].as_float().ok()?;
    Some((x, y))
}

fn array_to_vec3(arr: &Array) -> Option<(f32, f32, f32)> {
    if arr.len() < 3 {
        return None;
    }
    let x = arr[0].as_float().ok()?;
    let y = arr[1].as_float().ok()?;
    let z = arr[2].as_float().ok()?;
    Some((x, y, z))
}

fn vec2_to_array(x: f32, y: f32) -> Array {
    vec![Dynamic::from(x), Dynamic::from(y)]
}

fn vec3_to_array(x: f32, y: f32, z: f32) -> Array {
    vec![Dynamic::from(x), Dynamic::from(y), Dynamic::from(z)]
}

trait ToArray {
    fn to_dynamic_vec(&self) -> Array;
}

impl ToArray for [f32; 2] {
    fn to_dynamic_vec(&self) -> Array {
        vec2_to_array(self[0], self[1])
    }
}

impl ToArray for [f32; 3] {
    fn to_dynamic_vec(&self) -> Array {
        vec3_to_array(self[0], self[1], self[2])
    }
}

pub fn register_common_functions(engine: &mut Engine) {
    // Math functions
    engine
        .register_fn("abs".register_completer(), |x: i32| x.abs())
        .register_fn("min".register_completer(), |a: i32, b: i32| a.min(b))
        .register_fn("max".register_completer(), |a: i32, b: i32| a.max(b))
        .register_fn(
            "clamp".register_completer(),
            |x: i32, min: i32, max: i32| x.clamp(min, max),
        )
        .register_fn("lerp".register_completer(), |a: f32, b: f32, t: f32| {
            a + (b - a) * t
        });

    // Random functions
    engine
        .register_fn("random_int".register_completer(), |seed: Dynamic| {
            rng_seeded_from_hash(&seed).random_range(0..100)
        })
        .register_fn(
            "random_range".register_completer(),
            |seed: Dynamic, min: i32, max: i32| rng_seeded_from_hash(&seed).random_range(min..max),
        )
        .register_fn(
            "random_range".register_completer(),
            |seed: Dynamic, min: i32, max: i32| rng_seeded_from_hash(&seed).random_range(min..max),
        )
        .register_fn("random_float".register_completer(), |seed: Dynamic| {
            rng_seeded_from_hash(&seed).random_range(0.0..1.0)
        });

    // Color helpers
    engine
        .register_fn("rgb".register_completer(), |r: i32, g: i32, b: i32| {
            vec![r, g, b, 255]
        })
        .register_fn(
            "rgba".register_completer(),
            |r: i32, g: i32, b: i32, a: i32| vec![r, g, b, a],
        )
        .register_fn(
            "hsv_to_rgb".register_completer(),
            |h: f32, s: f32, v: f32| {
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
                    ((r + m) * 255.0) as i32,
                    ((g + m) * 255.0) as i32,
                    ((b + m) * 255.0) as i32,
                    255,
                ]
            },
        );

    // Vector math helpers
    engine
        .register_fn("+", |a: Array, b: Array| {
            let mut result: Array = default();
            for i in 0..a.len().min(b.len()) {
                result.push(Dynamic::from_float(
                    a[i].as_float().unwrap_or_default() + b[i].as_float().unwrap_or_default(),
                ));
            }
            result
        })
        .register_fn("-", |a: Array, b: Array| {
            let mut result: Array = default();
            for i in 0..a.len().min(b.len()) {
                result.push(Dynamic::from_float(
                    a[i].as_float().unwrap_or_default() - b[i].as_float().unwrap_or_default(),
                ));
            }
            result
        })
        .register_fn("*", |a: Array, scalar: f32| {
            let mut result: Array = default();
            for val in a.iter() {
                result.push(Dynamic::from_float(
                    val.as_float().unwrap_or_default() * scalar,
                ));
            }
            result
        })
        .register_fn("*", |scalar: f32, a: Array| {
            let mut result: Array = default();
            for val in a.iter() {
                result.push(Dynamic::from_float(
                    val.as_float().unwrap_or_default() * scalar,
                ));
            }
            result
        })
        .register_fn("unit_vec".register_completer(), |x: f32| {
            [x.sin(), x.cos()].to_dynamic_vec()
        })
        .register_fn("vec2".register_completer(), |x: f32, y: f32| {
            vec2_to_array(x, y)
        })
        .register_fn("vec3".register_completer(), |x: f32, y: f32, z: f32| {
            vec3_to_array(x, y, z)
        })
        .register_fn("distance".register_completer(), |v1: Array, v2: Array| {
            if let (Some((x1, y1)), Some((x2, y2))) = (array_to_vec2(&v1), array_to_vec2(&v2)) {
                ((x2 - x1).powi(2) + (y2 - y1).powi(2)).sqrt()
            } else {
                0.0
            }
        })
        .register_fn("normalize".register_completer(), |v: Array| {
            if let Some((x, y)) = array_to_vec2(&v) {
                let len = (x * x + y * y).sqrt();
                if len > 0.0 {
                    vec2_to_array(x / len, y / len)
                } else {
                    vec2_to_array(0.0, 0.0)
                }
            } else {
                v
            }
        })
        .register_fn("dot".register_completer(), |v1: Array, v2: Array| {
            if let (Some((x1, y1)), Some((x2, y2))) = (array_to_vec2(&v1), array_to_vec2(&v2)) {
                x1 * x2 + y1 * y2
            } else {
                0.0
            }
        })
        .register_fn("magnitude".register_completer(), |v: Array| {
            if let Some((x, y)) = array_to_vec2(&v) {
                (x * x + y * y).sqrt()
            } else if let Some((x, y, z)) = array_to_vec3(&v) {
                (x * x + y * y + z * z).sqrt()
            } else {
                0.0
            }
        });

    // Game-specific constants
    engine
        .register_fn("gt".register_completer(), || gt().play_head())
        .register_fn("player_id".register_completer(), || player_id());

    // Logging functions
    engine
        .register_fn("log".register_completer(), |msg: &str| {
            log::info!("Rhai Script: {}", msg);
        })
        .register_fn("debug".register_completer(), |msg: &str| {
            log::debug!("Rhai Script: {}", msg);
        });
}

pub fn register_vec_extensions(engine: &mut Engine) {
    engine
        .register_fn(
            "random".register_completer(),
            |vec: Vec<i32>| -> Option<i32> {
                use rand::seq::SliceRandom;
                let mut v = vec;
                v.as_mut_slice().shuffle(&mut rand::rng());
                v.first().copied()
            },
        )
        .register_fn(
            "random".register_completer(),
            |vec: Vec<NUnit>| -> Option<NUnit> {
                use rand::seq::SliceRandom;
                let mut v = vec;
                v.as_mut_slice().shuffle(&mut rand::rng());
                v.first().cloned()
            },
        )
        .register_fn(
            "random".register_completer(),
            |vec: Vec<NStatusMagic>| -> Option<NStatusMagic> {
                use rand::seq::SliceRandom;
                let mut v = vec;
                v.as_mut_slice().shuffle(&mut rand::rng());
                v.first().cloned()
            },
        )
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
