use super::*;

impl Logic {
    pub fn process_panels(&mut self) {
        for panel in &mut self.model.render_model.panels {
            panel.visible = true;
            panel.time_passed += self.delta_time;
        }
        self.model
            .render_model
            .panels
            .retain(|panel| panel.time_passed < panel.duration)
    }
}
