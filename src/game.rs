use geng::prelude::itertools::Itertools;

use super::*;

pub struct Game {
    world: legion::World,
    resources: Resources,
    systems: Vec<Box<dyn System>>,
}

impl Game {
    pub fn new(mut world: legion::World, mut resources: Resources) -> Self {
        let systems = Game::create_systems(&mut resources);
        Game::init_world(&mut resources, &mut world);

        Self {
            world,
            resources,
            systems,
        }
    }
}

impl geng::State for Game {
    fn update(&mut self, delta_time: f64) {
        self.resources.delta_time = delta_time as Time;
        self.resources.game_time += self.resources.delta_time;

        self.systems
            .iter_mut()
            .for_each(|s| s.update(&mut self.world, &mut self.resources));
        self.resources.down_keys.clear();
    }

    fn handle_event(&mut self, event: geng::Event) {
        match event {
            geng::Event::KeyDown { key } => {
                self.resources.down_keys.insert(key);
                self.resources.pressed_keys.insert(key);
            }

            geng::Event::KeyUp { key } => {
                self.resources.pressed_keys.remove(&key);
            }

            _ => {}
        }
    }

    fn ui<'a>(&'a mut self, cx: &'a ui::Controller) -> Box<dyn ui::Widget + 'a> {
        let widgets = self
            .systems
            .iter_mut()
            .map(|system| system.ui(cx, &self.resources))
            .collect_vec();
        if widgets.is_empty() {
            return Box::new(ui::Void);
        }
        Box::new(geng::ui::stack(widgets))
    }

    fn draw(&mut self, framebuffer: &mut ugli::Framebuffer) {
        ugli::clear(framebuffer, Some(Rgba::BLACK), None, None);
        self.systems
            .iter()
            .for_each(|s| s.draw(&self.world, &self.resources, framebuffer));
    }
}
