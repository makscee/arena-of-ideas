use super::*;

pub struct Game {
    world: legion::World,
    resources: Resources,
    systems: Vec<Box<dyn System>>,
}

impl Game {
    pub fn new(world: legion::World, mut resources: Resources) -> Self {
        let mut fws = FileWatcherSystem::new();
        resources.load(&mut fws);

        let mut systems: Vec<Box<dyn System>> = Vec::default();
        systems.push(Box::new(GameStateSystem::new(GameState::MainMenu)));
        systems.push(Box::new(ShaderSystem::new()));
        systems.push(Box::new(fws));

        Self {
            world,
            resources,
            systems,
        }
    }
}

impl State for Game {
    fn update(&mut self, delta_time: f64) {
        self.resources.delta_time = delta_time as Time;
        self.resources.game_time += self.resources.delta_time;

        self.systems
            .iter_mut()
            .for_each(|s| s.update(&mut self.world, &mut self.resources));
        self.resources.down_key = None;
    }

    fn handle_event(&mut self, event: Event) {
        match event {
            Event::KeyDown { key } => {
                self.resources.down_key = Some(key);
            }
            _ => {}
        }
    }

    fn ui<'a>(&'a mut self, cx: &'a ui::Controller) -> Box<dyn ui::Widget + 'a> {
        #![allow(unused_variables)]
        Box::new(ui::Void)
    }

    fn draw(&mut self, framebuffer: &mut ugli::Framebuffer) {
        self.systems
            .iter()
            .for_each(|s| s.draw(&self.world, &self.resources, framebuffer));
    }
}
