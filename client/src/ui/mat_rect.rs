use super::*;

/// MatRect widget for rendering materials with customizable clipping effects
///
/// Usage examples:
///
/// // Single material
/// MatRect::new(egui::Vec2::new(60.0, 60.0))
///     .add_mat(&material, owner_id)
///     .ui(ui, context);
///
/// // Multiple materials with different clipping effects
/// MatRect::new(egui::Vec2::new(60.0, 60.0))
///     .add_mat(&material1, owner_id1)
///     .add_mat_with_config(&material2, owner_id2, RenderConfig::default()
///         .clip_horizontal(0.0, 30.0)
///         .alpha(0.7))
///     .add_mat_with_config(&material3, owner_id3, RenderConfig::default()
///         .clip_circle(egui::Vec2::new(30.0, 30.0), 15.0)
///         .scale(0.8))
///     .render_unit_rep(true)
///     .ui(ui, context);
///
/// // Batch set materials with custom unit_rep configuration
/// MatRect::new(egui::Vec2::new(60.0, 60.0))
///     .materials(vec![
///         (&material1, owner_id1, RenderConfig::default()),
///         (&material2, owner_id2, RenderConfig::default().alpha(0.6))
///     ])
///     .unit_rep(fusion_id, RenderConfig::default()
///         .alpha(0.5)
///         .offset(egui::Vec2::new(5.0, 5.0)))
///     .ui(ui, context);

#[derive(Clone, Copy, Debug)]
pub enum ClipMode {
    None,
    Horizontal { offset: f32, width: f32 },
    Vertical { offset: f32, height: f32 },
    Circle { center: egui::Vec2, radius: f32 },
    Custom(Rect),
}

impl Default for ClipMode {
    fn default() -> Self {
        ClipMode::None
    }
}

#[derive(Clone, Debug)]
pub struct RenderConfig {
    pub clip_mode: ClipMode,
    pub alpha: f32,
    pub scale: f32,
    pub offset: egui::Vec2,
}

impl Default for RenderConfig {
    fn default() -> Self {
        Self {
            clip_mode: ClipMode::None,
            alpha: 1.0,
            scale: 1.0,
            offset: egui::Vec2::ZERO,
        }
    }
}

pub struct MatRect<'a> {
    size: egui::Vec2,
    materials: Vec<(&'a Material, u64, RenderConfig)>,
    unit_rep: Option<(u64, RenderConfig)>,
    enabled: bool,
    active: bool,
}

impl<'a> MatRect<'a> {
    pub fn new(size: egui::Vec2) -> Self {
        Self {
            size,
            materials: Vec::new(),
            unit_rep: None,
            enabled: true,
            active: false,
        }
    }

    pub fn materials(mut self, materials: Vec<(&'a Material, u64, RenderConfig)>) -> Self {
        self.materials = materials;
        self
    }

    pub fn add_mat(mut self, material: &'a Material, owner_id: u64) -> Self {
        self.materials
            .push((material, owner_id, RenderConfig::default()));
        self
    }

    pub fn add_mat_with_config(
        mut self,
        material: &'a Material,
        owner_id: u64,
        config: RenderConfig,
    ) -> Self {
        self.materials.push((material, owner_id, config));
        self
    }

    pub fn unit_rep(mut self, owner_id: u64, config: RenderConfig) -> Self {
        self.unit_rep = Some((owner_id, config));
        self
    }

    pub fn unit_rep_with_default(mut self, owner_id: u64) -> Self {
        self.unit_rep = Some((owner_id, RenderConfig::default()));
        self
    }

    pub fn enabled(mut self, enabled: bool) -> Self {
        self.enabled = enabled;
        self
    }

    pub fn active(mut self, active: bool) -> Self {
        self.active = active;
        self
    }

    fn apply_clip_mode(&self, ui: &mut Ui, rect: Rect, clip_mode: ClipMode) -> Rect {
        match clip_mode {
            ClipMode::None => rect,
            ClipMode::Horizontal { offset, width } => {
                let clipped_rect = Rect::from_min_size(
                    rect.min + egui::vec2(offset, 0.0),
                    egui::vec2(width, rect.height()),
                );
                ui.set_clip_rect(clipped_rect.intersect(rect));
                clipped_rect
            }
            ClipMode::Vertical { offset, height } => {
                let clipped_rect = Rect::from_min_size(
                    rect.min + egui::vec2(0.0, offset),
                    egui::vec2(rect.width(), height),
                );
                ui.set_clip_rect(clipped_rect.intersect(rect));
                clipped_rect
            }
            ClipMode::Circle { center, radius } => {
                let circle_rect = Rect::from_center_size(
                    center.to_pos2(),
                    egui::vec2(radius * 2.0, radius * 2.0),
                );
                ui.set_clip_rect(circle_rect.intersect(rect));
                circle_rect
            }
            ClipMode::Custom(custom_rect) => {
                ui.set_clip_rect(custom_rect.intersect(rect));
                custom_rect
            }
        }
    }

    fn render_material(
        &self,
        material: &Material,
        owner_id: u64,
        rect: Rect,
        config: &RenderConfig,
        context: &ClientContext,
        ui: &mut Ui,
    ) -> NodeResult<()> {
        let scaled_rect = if config.scale != 1.0 {
            let size = rect.size() * config.scale;
            Rect::from_center_size(rect.center(), size)
        } else {
            rect
        };

        let offset_rect = scaled_rect.translate(config.offset);
        let clipped_rect = self.apply_clip_mode(ui, offset_rect, config.clip_mode);

        // Apply alpha if needed
        if config.alpha < 1.0 {
            let painter = ui.painter().with_clip_rect(clipped_rect);
            let color = Color32::from_white_alpha((config.alpha * 255.0) as u8);
            painter.rect_filled(clipped_rect, 0.0, color);
        }

        // Try to get entity from owner_id - could be any node type

        context.with_owner_ref(owner_id, |ctx| {
            RepresentationPlugin::paint_rect(clipped_rect, ctx, material, ui)
        })?;
        Ok(())
    }

    pub fn ui(self, ui: &mut Ui, context: &ClientContext) -> Response {
        let button = RectButton::new_size(self.size)
            .enabled(self.enabled)
            .active(self.active);

        button.ui(ui, |color, rect, _, ui| {
            corners_rounded_rect(rect, rect.width() * 0.1, color.stroke(), ui);
            let content_rect = rect.shrink(5.0);

            // Render all materials
            for (material, owner_id, config) in &self.materials {
                let _ =
                    self.render_material(material, *owner_id, content_rect, config, context, ui);
            }

            // Render unit_rep if configured
            if let Some((owner_id, config)) = &self.unit_rep {
                let unit_rep_rect = if config.scale != 1.0 {
                    let size = content_rect.size() * config.scale;
                    Rect::from_center_size(content_rect.center(), size)
                } else {
                    content_rect
                };

                let offset_rect = unit_rep_rect.translate(config.offset);
                let clipped_rect = self.apply_clip_mode(ui, offset_rect, config.clip_mode);

                // Apply alpha if needed
                if config.alpha < 1.0 {
                    let painter = ui.painter().with_clip_rect(clipped_rect);
                    let color = Color32::from_white_alpha((config.alpha * 255.0) as u8);
                    painter.rect_filled(clipped_rect, 0.0, color);
                }

                context
                    .with_owner_ref(*owner_id, |ctx| {
                        let r = unit_rep().material.paint(clipped_rect, ctx, ui);
                        match &r {
                            Ok(_) => {}
                            Err(e) => {
                                dbg!(e);
                                ctx.debug_layers();
                                panic!();
                            }
                        }
                        r
                    })
                    .ui(ui);
            }
        })
    }
}

impl RenderConfig {
    pub fn clip_horizontal(mut self, offset: f32, width: f32) -> Self {
        self.clip_mode = ClipMode::Horizontal { offset, width };
        self
    }

    pub fn clip_vertical(mut self, offset: f32, height: f32) -> Self {
        self.clip_mode = ClipMode::Vertical { offset, height };
        self
    }

    pub fn clip_circle(mut self, center: egui::Vec2, radius: f32) -> Self {
        self.clip_mode = ClipMode::Circle { center, radius };
        self
    }

    pub fn clip_custom(mut self, rect: Rect) -> Self {
        self.clip_mode = ClipMode::Custom(rect);
        self
    }

    pub fn alpha(mut self, alpha: f32) -> Self {
        self.alpha = alpha.clamp(0.0, 1.0);
        self
    }

    pub fn scale(mut self, scale: f32) -> Self {
        self.scale = scale.max(0.01);
        self
    }

    pub fn offset(mut self, offset: egui::Vec2) -> Self {
        self.offset = offset;
        self
    }
}
