use std::collections::VecDeque;

use geng::prelude::itertools::Itertools;

use super::*;
use geng::ui::*;

pub struct Game {
    world: legion::World,
    resources: Resources,
    systems: Vec<Box<dyn System>>,
    fps: VecDeque<f32>,
}

impl Game {
    pub fn new(world: legion::World, mut resources: Resources, watcher: FileWatcherSystem) -> Self {
        let mut systems = Game::create_systems(&mut resources);
        systems.push(Box::new(watcher));

        Self {
            world,
            resources,
            systems,
            fps: VecDeque::from_iter((0..30).map(|x| x as f32)),
        }
    }

    pub fn init_world(resources: &mut Resources, world: &mut legion::World) {
        let world_entity = WorldSystem::init_world_entity(world);
        Self::init_field(resources, world, world_entity);
    }

    fn init_field(
        resources: &mut Resources,
        world: &mut legion::World,
        world_entity: legion::Entity,
    ) {
        let shader = resources.options.shaders.field.clone();
        let entity = world.push((shader,));
        let mut entry = world.entry(entity).unwrap();
        entry.add_component(EntityComponent { entity });
        entry.add_component(Context {
            owner: entity,
            target: entity,
            parent: Some(world_entity),
            vars: default(),
        })
    }

    pub fn reset(world: &mut legion::World, resources: &mut Resources) {
        UnitSystem::clear_factions(
            world,
            resources,
            &hashset! {Faction::Light, Faction::Dark, Faction::Shop, Faction::Team, },
        );
        StatusPool::clear_all_active(resources);
        resources.floors.reset();
        resources.action_queue.clear();
        resources.factions_state = default();
    }

    pub fn restart(world: &mut legion::World, resources: &mut Resources) {
        Self::reset(world, resources);
        resources.current_state = GameState::MainMenu;
        resources.transition_state = GameState::Shop;
    }
}

impl geng::State for Game {
    fn update(&mut self, delta_time: f64) {
        self.resources.delta_time = delta_time as Time;

        self.systems
            .iter_mut()
            .for_each(|s| s.pre_update(&mut self.world, &mut self.resources));
        self.systems
            .iter_mut()
            .for_each(|s| s.update(&mut self.world, &mut self.resources));
        self.systems
            .iter_mut()
            .for_each(|s| s.post_update(&mut self.world, &mut self.resources));
        self.resources.input.down_keys.clear();
        self.resources.input.down_mouse_buttons.clear();
        self.resources.input.up_mouse_buttons.clear();
    }

    fn handle_event(&mut self, event: geng::Event) {
        match event {
            geng::Event::KeyDown { key } => {
                self.resources.input.down_keys.insert(key);
                self.resources.input.pressed_keys.insert(key);
            }

            geng::Event::KeyUp { key } => {
                self.resources.input.pressed_keys.remove(&key);
            }

            geng::Event::MouseDown {
                position: _,
                button,
            } => {
                self.resources.input.down_mouse_buttons.insert(button);
                self.resources.input.pressed_mouse_buttons.insert(button);
            }

            geng::Event::MouseUp {
                position: _,
                button,
            } => {
                self.resources.input.pressed_mouse_buttons.remove(&button);
                self.resources.input.up_mouse_buttons.insert(button);
            }

            _ => {}
        }
    }

    fn ui<'a>(&'a mut self, cx: &'a ui::Controller) -> Box<dyn ui::Widget + 'a> {
        let mut widgets = self
            .systems
            .iter_mut()
            .map(|system| system.ui(cx, &self.world, &self.resources))
            .collect_vec();
        self.fps.pop_front();
        self.fps.push_back(1.0 / self.resources.delta_time);
        let mut fps: f32 = self.fps.iter().sum();
        fps /= self.fps.len() as f32;
        let fps = Text::new(
            format!("{:.0}", fps),
            self.resources.fonts.get_font(1),
            32.0,
            Rgba::WHITE,
        )
        .fixed_size(vec2(60.0, 30.0))
        .background_color(Rgba::BLACK)
        .align(vec2(0.0, 0.0))
        .boxed();
        widgets.push(fps);
        Box::new(geng::ui::stack(widgets))
    }

    fn draw(&mut self, framebuffer: &mut ugli::Framebuffer) {
        self.resources.input.mouse_pos = self.resources.camera.camera.screen_to_world(
            framebuffer.size().map(|x| x as f32),
            self.resources
                .geng
                .as_ref()
                .unwrap()
                .window()
                .mouse_pos()
                .map(|x| x as f32),
        );
        ugli::clear(framebuffer, Some(Rgba::BLACK), None, None);
        self.systems
            .iter()
            .for_each(|s| s.draw(&self.world, &mut self.resources, framebuffer));
    }
}
