use super::*;

pub struct PanelsSystem {}

impl System for PanelsSystem {
    fn update(&mut self, world: &mut legion::World, resources: &mut Resources) {
        for panel in resources.panels_data.center.iter_mut() {
            panel.update(resources.delta_time)
        }
        resources.panels_data.center.retain(|x| !x.is_closed());
        resources.frame_shaders.extend(
            resources
                .panels_data
                .center
                .iter()
                .map(|x| x.shader.clone()),
        );
    }
}

impl PanelsSystem {
    pub fn new() -> Self {
        Self {}
    }

    pub fn add_alert(title: &str, text: &str, resources: &mut Resources) {
        let panel = Panel::new(title, text, &resources.options);
        resources.panels_data.center.push(panel);
    }

    pub fn close_panel(entity: legion::Entity, resources: &mut Resources) {
        for panel in resources.panels_data.center.iter_mut() {
            if panel.shader.entity == Some(entity) || panel.shader.parent == Some(entity) {
                panel.state = PanelState::Closed;
            }
        }
    }

    pub fn clear(resources: &mut Resources) {
        resources.panels_data.center.clear();
    }
}

#[derive(Default)]
pub struct PanelsData {
    pub center: Vec<Panel>,
}

#[derive(Debug)]
pub struct Panel {
    pub shader: Shader,
    pub state: PanelState,
    pub r#type: PanelType,
    pub t: Time,
}

impl Panel {
    pub fn new(title: &str, text: &str, options: &Options) -> Self {
        Self {
            shader: options.shaders.panel.clone(),
            state: PanelState::Open,
            r#type: PanelType::Alert,
            t: 1.0,
            // t: default(),
        }
    }

    pub fn update(&mut self, delta: Time) {
        match self.state {
            PanelState::Open => self.t = (self.t + delta).min(1.0),
            PanelState::Closed => self.t = (self.t - delta).max(0.0),
        }
        self.shader.set_float_ref("u_open", self.t);
    }

    pub fn is_closed(&self) -> bool {
        self.state == PanelState::Closed && self.t <= 0.0
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
