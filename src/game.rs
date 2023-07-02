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
            fps: VecDeque::from_iter((0..30).map(|_| 0.0)),
        }
    }

    pub fn init_world(resources: &mut Resources, world: &mut legion::World) {
        let world_entity = WorldSystem::init_world_entity(world, &resources.options);
        Self::init_field(world_entity, resources, world);
        AbilityPool::init_world(world, resources);
    }

    fn init_field(parent: legion::Entity, resources: &mut Resources, world: &mut legion::World) {
        let shader = resources.options.shaders.field.clone();
        let entity = world.push((shader,));
        let mut entry = world.entry(entity).unwrap();
        entry.add_component(EntityComponent::new(entity));
        entry.add_component(ContextState::new("Field".to_owned(), Some(parent)));
    }

    pub fn reset(world: &mut legion::World, resources: &mut Resources) {
        UnitSystem::clear_factions(world, &Faction::all());
        resources.ladder.reset();
        resources.action_queue.clear();
        resources.prepared_shaders.clear();
        resources.battle_data.total_score = default();
        resources.transition_state = GameState::MainMenu;
        ShopData::load_pool_full(resources);
        world.clear();
        PanelsSystem::clear(resources);
    }

    pub fn restart(world: &mut legion::World, resources: &mut Resources) {
        Self::reset(world, resources);
        Self::init_world(resources, world);
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
        self.resources.input_data.down_keys.clear();
        self.resources.input_data.down_mouse_buttons.clear();
        self.resources.input_data.up_mouse_buttons.clear();
    }

    fn handle_event(&mut self, event: geng::Event) {
        match event {
            geng::Event::KeyDown { key } => {
                self.resources.input_data.down_keys.insert(key);
                self.resources.input_data.pressed_keys.insert(key);
            }

            geng::Event::KeyUp { key } => {
                self.resources.input_data.pressed_keys.remove(&key);
            }

            geng::Event::MouseDown {
                position: _,
                button,
            } => {
                self.resources.input_data.down_mouse_buttons.insert(button);
                self.resources
                    .input_data
                    .pressed_mouse_buttons
                    .insert(button);
            }

            geng::Event::MouseUp {
                position: _,
                button,
            } => {
                self.resources
                    .input_data
                    .pressed_mouse_buttons
                    .remove(&button);
                self.resources.input_data.up_mouse_buttons.insert(button);
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
            self.resources.fonts.get_font(0),
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
        self.resources.camera.framebuffer_size = framebuffer.size().map(|x| x as f32);
        self.resources.camera.aspect_ratio =
            self.resources.camera.framebuffer_size.x / self.resources.camera.framebuffer_size.y;
        let mouse_pos = self
            .resources
            .geng
            .as_ref()
            .unwrap()
            .window()
            .mouse_pos()
            .map(|x| x as f32);
        self.resources.input_data.mouse_screen_pos =
            (mouse_pos / self.resources.camera.framebuffer_size * 2.0 - vec2(1.0, 1.0))
                * vec2(self.resources.camera.aspect_ratio, 1.0);
        self.resources.input_data.mouse_world_pos = self
            .resources
            .camera
            .camera
            .screen_to_world(framebuffer.size().map(|x| x as f32), mouse_pos);
        ugli::clear(framebuffer, Some(Rgba::BLACK), None, None);
        self.systems
            .iter()
            .for_each(|s| s.draw(&self.world, &mut self.resources, framebuffer));
    }
}
