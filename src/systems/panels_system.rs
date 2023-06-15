use super::*;

pub struct PanelsSystem {}

impl System for PanelsSystem {
    fn update(&mut self, _: &mut legion::World, resources: &mut Resources) {
        for panel in resources
            .panels_data
            .alert
            .iter_mut()
            .chain(resources.panels_data.push.iter_mut())
        {
            panel.update(resources.delta_time)
        }

        resources.panels_data.alert.retain(|x| !x.is_closed());
        resources.panels_data.push.retain(|x| !x.is_closed());

        resources.frame_shaders.extend(
            resources
                .panels_data
                .alert
                .iter()
                .chain(resources.panels_data.push.iter())
                .map(|x| x.shader.clone()),
        );
    }
}

impl PanelsSystem {
    pub fn new() -> Self {
        Self {}
    }

    pub fn add_alert(title: &str, text: &str, resources: &mut Resources) {
        let mut panel = Panel::new(title, PanelType::Alert, &resources.options);
        panel.add_text(text, &resources.options);
        panel.add_close_button("Close", &resources.options);
        panel
            .shader
            .set_int_ref("u_index", resources.panels_data.alert.len() as i32);
        resources.panels_data.alert.push(panel);
    }

    pub fn close_alert(entity: legion::Entity, resources: &mut Resources) {
        for panel in resources.panels_data.alert.iter_mut() {
            if panel.shader.entity == Some(entity) || panel.shader.parent == Some(entity) {
                panel.state = PanelState::Closed;
            }
        }
    }

    pub fn add_push(title: &str, text: &str, resources: &mut Resources) {
        let mut panel = Panel::new(title, PanelType::Push, &resources.options);
        panel.add_text(text, &resources.options);
        resources.panels_data.push.insert(0, panel);
        let corner = vec2(-resources.camera.aspect_ratio, -1.0) + vec2(0.1, 0.1);

        for (ind, panel) in resources.panels_data.push.iter_mut().enumerate() {
            panel.shader.parameters.r#box.pos = corner;
        }
    }

    pub fn clear(resources: &mut Resources) {
        resources.panels_data.alert.clear();
        resources.panels_data.push.clear();
    }
}

#[derive(Default)]
pub struct PanelsData {
    pub alert: Vec<Panel>,
    pub push: Vec<Panel>,
}

#[derive(Debug)]
pub struct Panel {
    pub shader: Shader,
    pub state: PanelState,
    pub r#type: PanelType,
    pub t: Time,
}

impl Panel {
    pub fn new(title: &str, r#type: PanelType, options: &Options) -> Self {
        let mut shader =
            options
                .shaders
                .panel
                .clone()
                .set_string("u_title_text", title.to_owned(), 1);
        match r#type {
            PanelType::Push => {
                shader.parameters.r#box.center = vec2(-1.0, -1.0);
                shader.parameters.r#box.size.x = 0.2;
            }
            PanelType::Alert => {}
            PanelType::Hint => {}
        }
        shader.entity = Some(new_entity());
        Self {
            shader,
            state: PanelState::Open,
            r#type,
            // t: 1.0,
            t: default(),
        }
    }

    pub fn add_text(&mut self, text: &str, options: &Options) {
        let mut shader =
            options
                .shaders
                .panel_text
                .clone()
                .set_string("u_text", text.to_owned(), 0);
        let lines = text.chars().map(|x| (x == '\n') as i32).sum::<i32>() + 1;
        let per_line = shader.parameters.r#box.size.y;
        shader.parameters.r#box.size.y = lines as f32 * per_line;
        self.shader.parameters.r#box.size.y = shader.parameters.r#box.size.y + 2.0 * per_line;
        self.shader.chain_after.push(shader);
    }

    pub fn update(&mut self, delta: Time) {
        match self.state {
            PanelState::Open => self.t = (self.t + delta * 2.0).min(1.0),
            PanelState::Closed => self.t = (self.t - delta * 2.0).max(0.0),
        }
        self.shader
            .set_float_ref("u_open", EasingType::QuadInOut.f(self.t));
    }

    pub fn is_closed(&self) -> bool {
        self.state == PanelState::Closed && self.t <= 0.0
    }

    pub fn add_close_button(&mut self, title: &str, options: &Options) {
        let mut button =
            options
                .shaders
                .panel_button
                .clone()
                .set_string("u_text", title.to_owned(), 1);
        button.parent = self.shader.entity;
        button.entity = Some(new_entity());
        ButtonSystem::add_button_handlers(&mut button);
        fn close_panel_handler(
            event: HandleEvent,
            _: legion::Entity,
            shader: &mut Shader,
            _: &mut legion::World,
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
        button.input_handlers.push(close_panel_handler);
        self.shader.chain_after.push(button);
        self.shader.set_float_ref("u_footer", 1.0);
    }
}

impl Shader {
    // pub fn wrap_in_panel(self, title: &str) -> Self {}
}

#[derive(Debug)]
pub enum PanelType {
    Push,
    Alert,
    Hint,
}

#[derive(Eq, PartialEq, Debug)]
pub enum PanelState {
    Open,
    Closed,
}
