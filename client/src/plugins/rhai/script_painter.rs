use super::*;

#[derive(Clone, Debug)]
pub struct RhaiPainter {
    painter_ptr: *const u8,
}

impl RhaiPainter {
    pub fn with_painter(painter: &Painter) -> Self {
        Self {
            painter_ptr: painter as *const _ as *const u8,
        }
    }

    unsafe fn get_painter_mut(&self) -> &mut Painter {
        &mut *(self.painter_ptr as *mut Painter)
    }

    pub fn circle(&self, radius: f64) {
        unsafe {
            let p = self.get_painter_mut();
            let r = p.rect;
            let up = r.width().min(r.height()) * 0.5;
            let radius = (radius as f32) * up;
            let shape = if let Some(width) = p.hollow {
                epaint::CircleShape::stroke(Default::default(), radius, epaint::Stroke::new(width, p.color))
            } else {
                epaint::CircleShape::filled(Default::default(), radius, p.color)
            };
            p.tesselator.tessellate_circle(shape, &mut p.mesh);
        }
    }

    pub fn translate(&self, x: f64, y: f64) {
        unsafe {
            let p = self.get_painter_mut();
            let dx = x as f32;
            let dy = y as f32;
            for vertex in &mut p.mesh.vertices {
                vertex.pos.x += dx;
                vertex.pos.y += dy;
            }
        }
    }

    pub fn rotate(&self, angle: f64) {
        unsafe {
            let p = self.get_painter_mut();
            let rot = emath::Rot2::from_angle(angle as f32);
            for vertex in &mut p.mesh.vertices {
                let v = vertex.pos - egui::Pos2::ZERO;
                let rotated = rot * v;
                vertex.pos = egui::Pos2::ZERO + rotated;
            }
        }
    }

    pub fn scale_mesh(&self, scale: f64) {
        unsafe {
            let p = self.get_painter_mut();
            let s = scale as f32;
            for vertex in &mut p.mesh.vertices {
                vertex.pos.x *= s;
                vertex.pos.y *= s;
            }
        }
    }

    pub fn color(&self, r: i64, g: i64, b: i64, a: i64) {
        unsafe {
            let p = self.get_painter_mut();
            p.color = epaint::Color32::from_rgba_unmultiplied(
                (r as u8).clamp(0, 255),
                (g as u8).clamp(0, 255),
                (b as u8).clamp(0, 255),
                (a as u8).clamp(0, 255),
            );
        }
    }

    pub fn alpha(&self, alpha: f64) {
        unsafe {
            let p = self.get_painter_mut();
            let a = ((alpha as f32).clamp(0.0, 1.0) * 255.0) as u8;
            let [r, g, b, _] = p.color.to_array();
            p.color = epaint::Color32::from_rgba_unmultiplied(r, g, b, a);
        }
    }

    pub fn hollow(&self, width: f64) {
        unsafe {
            let p = self.get_painter_mut();
            p.hollow = Some(width as f32);
        }
    }

    pub fn solid(&self) {
        unsafe {
            let p = self.get_painter_mut();
            p.hollow = None;
        }
    }
}

unsafe impl Send for RhaiPainter {}
unsafe impl Sync for RhaiPainter {}

pub fn register_painter_type(engine: &mut ::rhai::Engine) {
    engine
        .register_type_with_name::<RhaiPainter>("Painter")
        .register_fn("circle", |painter: &mut RhaiPainter, radius: f64| {
            painter.circle(radius);
        })
        .register_fn("translate", |painter: &mut RhaiPainter, x: f64, y: f64| {
            painter.translate(x, y);
        })
        .register_fn("rotate", |painter: &mut RhaiPainter, angle: f64| {
            painter.rotate(angle);
        })
        .register_fn("scale_mesh", |painter: &mut RhaiPainter, scale: f64| {
            painter.scale_mesh(scale);
        })
        .register_fn("color", |painter: &mut RhaiPainter, r: i64, g: i64, b: i64, a: i64| {
            painter.color(r, g, b, a);
        })
        .register_fn("alpha", |painter: &mut RhaiPainter, alpha: f64| {
            painter.alpha(alpha);
        })
        .register_fn("hollow", |painter: &mut RhaiPainter, width: f64| {
            painter.hollow(width);
        })
        .register_fn("solid", |painter: &mut RhaiPainter| {
            painter.solid();
        });
}
