use super::*;

pub struct Game {
    geng: Geng,
    world: World,
    resources: Resources,
}

impl Game {
    pub fn new(geng: Geng, world: World, resources: Resources) -> Self {
        Self {
            geng,
            world,
            resources,
        }
    }
}

impl State for Game {
    fn update(&mut self, delta_time: f64) {
        let delta_time = delta_time as Time;
        GameStateSystem::update(&mut self.world, delta_time);
    }

    fn handle_event(&mut self, event: Event) {
        GameStateSystem::handle_event(&mut self.world, event);
    }

    fn transition(&mut self) -> Option<Transition> {
        None
    }

    fn ui<'a>(&'a mut self, cx: &'a ui::Controller) -> Box<dyn ui::Widget + 'a> {
        #![allow(unused_variables)]
        Box::new(ui::Void)
    }

    fn draw(&mut self, framebuffer: &mut ugli::Framebuffer) {
        GameStateSystem::draw(&self.world, framebuffer);
    }
}
