use super::*;

pub struct PanelsSystem {}
const PADDING: f32 = 0.05;

impl System for PanelsSystem {
    fn update(&mut self, _: &mut legion::World, resources: &mut Resources) {
        let bot_left_corner = vec2(-resources.camera.aspect_ratio, -1.0) + vec2(PADDING, PADDING);
        let bot_right_corner = vec2(resources.camera.aspect_ratio, -1.0) + vec2(-PADDING, PADDING);
        let top_left_corner =
            vec2(-resources.camera.aspect_ratio, 1.0) + vec2(PADDING, -PADDING * 2.0);

        let mut offset = vec2::ZERO;
        for panel in resources.panels_data.push.iter_mut() {
            panel.need_pos = bot_left_corner + offset;
            if panel.t == 0.0 && panel.state == PanelState::Open {
                panel.shader.parameters.r#box.pos = panel.need_pos;
            }
            offset += vec2(0.0, PADDING + panel.shader.parameters.r#box.size.y * 2.0);
        }
        let mut offset = vec2::ZERO;
        for panel in resources.panels_data.hint.iter_mut() {
            panel.need_pos = bot_right_corner + offset;
            if panel.t == 0.0 && panel.state == PanelState::Open {
                panel.shader.parameters.r#box.pos = panel.need_pos;
            }
            offset += vec2(0.0, PADDING + panel.shader.parameters.r#box.size.y * 2.0);
        }

        if let Some(panel) = resources.panels_data.stats.as_mut() {
            panel.need_pos = top_left_corner;
        }

        let delta_time = resources.delta_time;
        let global_time = resources.global_time;
        for panel in resources
            .panels_data
            .alert
            .iter_mut()
            .chain(resources.panels_data.push.iter_mut())
            .chain(resources.panels_data.stats.iter_mut())
            .chain(resources.panels_data.hint.iter_mut())
        {
            panel.update(delta_time, global_time)
        }

        resources.panels_data.alert.retain(|x| !x.is_closed());
        resources.panels_data.push.retain(|x| !x.is_closed());
        resources.panels_data.hint.retain(|x| !x.is_closed());

        resources.frame_shaders.extend(
            resources
                .panels_data
                .alert
                .iter()
                .chain(resources.panels_data.push.iter())
                .chain(resources.panels_data.stats.iter())
                .chain(resources.panels_data.hint.iter())
                .map(|x| x.shader.clone()),
        );
    }
}

impl PanelsSystem {
    pub fn new() -> Self {
        Self {}
    }

    pub fn add_alert(
        title: &str,
        text: &str,
        pos: vec2<f32>,
        footer: bool,
        resources: &mut Resources,
    ) {
        let mut panel = Self::generate_text_shader(text, &resources.options)
            .wrap_panel_body(&resources.options)
            .wrap_panel_header(title, &resources.options);
        if footer {
            panel = panel.wrap_panel_footer(PanelFooterButton::Close, &resources.options);
        }
        let mut panel = panel.panel(PanelType::Alert, resources);
        panel.need_pos = pos;
        panel.shader.set_int_ref(
            "u_index".to_owned(),
            resources.panels_data.alert.len() as i32,
        );
        resources.panels_data.alert.push(panel);
    }

    pub fn add_push(title: &str, text: &str, resources: &mut Resources) {
        let panel = Self::generate_text_shader(text, &resources.options)
            .wrap_panel_body(&resources.options)
            .wrap_panel_header(title, &resources.options);
        resources
            .panels_data
            .push
            .insert(0, panel.panel(PanelType::Push, resources));
    }

    pub fn add_hint(title: &str, text: &str, resources: &mut Resources) {
        let panel = Self::generate_text_shader(&text, &resources.options)
            .wrap_panel_body(&resources.options)
            .wrap_panel_header(&title, &resources.options)
            .panel(PanelType::Hint, resources);
        resources.panels_data.hint.push(panel);
    }

    pub fn add_stats(world: &legion::World, resources: &mut Resources) {
        let text = Self::get_stats_text(world, resources);
        let panel = Self::generate_text_shader(&text, &resources.options)
            .wrap_panel_body(&resources.options)
            .wrap_panel_header("Stats", &resources.options)
            .panel(PanelType::Stats, resources);
        resources.panels_data.stats = Some(panel);
    }

    pub fn get_stats_text(world: &legion::World, resources: &mut Resources) -> String {
        let g = ShopSystem::get_g(world);
        let level = resources.ladder.current_ind();
        format!("g: {g}\nlevel: {level}")
    }

    pub fn refresh_stats(world: &legion::World, resources: &mut Resources) {
        if resources.panels_data.stats.is_some() {
            let text = Self::get_stats_text(world, resources);
            let panel = resources.panels_data.stats.as_mut().unwrap();
            panel
                .shader
                .set_string_ref("u_panel_text".to_owned(), text, 0);
        }
    }

    pub fn close_alert(entity: legion::Entity, resources: &mut Resources) {
        for panel in resources.panels_data.alert.iter_mut() {
            if panel.shader.entity == Some(entity) || panel.shader.parent == Some(entity) {
                panel.state = PanelState::Closed;
            }
        }
    }

    pub fn close_hints(resources: &mut Resources) {
        for panel in resources.panels_data.hint.iter_mut() {
            panel.state = PanelState::Closed;
        }
    }

    pub fn clear(resources: &mut Resources) {
        resources.panels_data.alert.clear();
        resources.panels_data.push.clear();
        resources.panels_data.hint.clear();
        resources.panels_data.stats = None;
    }

    pub fn generate_text_shader(text: &str, options: &Options) -> Shader {
        let mut shader =
            options
                .shaders
                .panel_text
                .clone()
                .set_string("u_text".to_owned(), text.to_owned(), 0);
        let lines = text.chars().map(|x| (x == '\n') as i32).sum::<i32>() + 1;
        let per_line = shader.parameters.r#box.size.y;
        shader.parameters.r#box.size.y = lines as f32 * per_line;
        shader
    }
}

#[derive(Default)]
pub struct PanelsData {
    pub alert: Vec<Panel>,
    pub push: Vec<Panel>,
    pub hint: Vec<Panel>,
    pub stats: Option<Panel>,
}

#[derive(Debug)]
pub struct Panel {
    pub shader: Shader,
    pub need_pos: vec2<f32>,
    pub state: PanelState,
    pub r#type: PanelType,
    pub t: Time,
    pub ts: Time,
}

impl Panel {
    pub fn update(&mut self, delta_time: Time, global_time: Time) {
        const SPEED: f32 = 10.0;

        self.shader.parameters.r#box.pos +=
            (self.need_pos - self.shader.parameters.r#box.pos) * SPEED * delta_time;
        if self.state == PanelState::Open {
            let duration = self.r#type.duration();
            if duration > 0.0 && self.ts + duration < global_time {
                self.state = PanelState::Closed;
            }
        }
        match self.state {
            PanelState::Open => self.t = (self.t + delta_time * 1.5).min(1.0),
            PanelState::Closed => self.t = (self.t - delta_time * 1.5).max(0.0),
        }
        self.shader
            .set_float_ref("u_open".to_owned(), EasingType::QuartInOut.f(self.t));
    }

    pub fn is_closed(&self) -> bool {
        self.state == PanelState::Closed && self.t <= 0.0
    }
}

impl Shader {
    pub fn wrap_panel_body(self, options: &Options) -> Self {
        let mut shader = options.shaders.panel_body.clone();
        shader.parameters.r#box = self.parameters.r#box;
        let padding = options.floats.panel_body_padding;
        shader.parameters.r#box.size += vec2(padding, padding);
        shader.chain_after.push(self);
        shader.entity = Some(new_entity());
        shader
    }

    pub fn wrap_panel_header(mut self, title: &str, options: &Options) -> Self {
        let mut shader = options.shaders.panel_header.clone().set_string(
            "u_title_text".to_owned(),
            title.to_owned(),
            1,
        );
        shader.parameters.r#box.size.x = self.parameters.r#box.size.x;
        for child in shader.chain_after.iter_mut() {
            child.parameters.r#box.size.x = shader.parameters.r#box.size.x;
        }
        self.chain_after.push(shader);
        self
    }

    pub fn wrap_panel_footer(mut self, button: PanelFooterButton, options: &Options) -> Self {
        let mut shader = options.shaders.panel_footer.clone();
        shader.parameters.r#box.size.x = self.parameters.r#box.size.x;
        for child in shader.chain_after.iter_mut() {
            child.parameters.r#box.size.x = shader.parameters.r#box.size.x;
        }
        let mut button_shader = options.shaders.panel_button.clone().set_string(
            "u_text".to_owned(),
            button.get_text().to_owned(),
            1,
        );
        button_shader.parent = self.entity;
        button_shader.entity = Some(new_entity());
        ButtonSystem::add_button_handlers(&mut button_shader);
        button_shader.input_handlers.push(button.get_handler());
        shader.chain_after.push(button_shader);
        self.chain_after.push(shader);
        self
    }

    pub fn panel(mut self, r#type: PanelType, resources: &Resources) -> Panel {
        match r#type {
            PanelType::Push => {
                self.parameters.r#box.center = vec2(-1.0, -1.0);
            }
            PanelType::Alert => {}
            PanelType::Hint => {
                self.parameters.r#box.center = vec2(1.0, -1.0);
            }
            PanelType::Stats => {
                self.parameters.r#box.center = vec2(-1.0, 1.0);
            }
        }
        Panel {
            need_pos: self.parameters.r#box.pos,
            shader: self,
            state: PanelState::Open,
            r#type,
            t: default(),
            ts: resources.global_time,
        }
    }
}

#[derive(Debug, Deserialize, PartialEq, Eq, Hash)]
pub enum PanelType {
    Push,
    Alert,
    Hint,
    Stats,
}

impl PanelType {
    pub fn duration(&self) -> Time {
        match self {
            PanelType::Push => 5.0,
            _ => 0.0,
        }
    }
}

#[derive(Eq, PartialEq, Debug)]
pub enum PanelState {
    Open,
    Closed,
}
pub enum PanelFooterButton {
    Close,
}

impl PanelFooterButton {
    pub fn get_text(&self) -> &str {
        match self {
            PanelFooterButton::Close => "Close",
        }
    }

    pub fn get_handler(&self) -> Handler {
        match self {
            PanelFooterButton::Close => {
                fn input_handler(
                    event: HandleEvent,
                    entity: legion::Entity,
                    shader: &mut Shader,
                    world: &mut legion::World,
                    resources: &mut Resources,
                ) {
                    match event {
                        HandleEvent::Click => {
                            if let Some(entity) = shader.parent {
                                PanelsSystem::close_alert(entity, resources);
                            }
                        }
                        _ => {}
                    }
                }
                input_handler
            }
        }
    }
}
