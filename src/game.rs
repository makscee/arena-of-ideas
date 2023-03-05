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

    pub fn init_world(resources: &mut Resources, world: &mut legion::World) {
        let world_entity = world.push((WorldComponent { global_time: 0.0 },));
        Self::init_field(resources, world, world_entity);
        let mut world_entry = world.entry(world_entity).unwrap();
        world_entry.add_component(EntityComponent {
            entity: world_entity,
        });
        let mut vars = Vars::default();
        vars.insert(VarName::FieldPosition, Var::Vec2(vec2(0.0, 0.0)));
        world_entry.add_component(Context {
            owner: world_entity,
            target: world_entity,
            parent: None,
            vars,
        });
    }

    fn init_field(
        resources: &mut Resources,
        world: &mut legion::World,
        world_entity: legion::Entity,
    ) {
        let shader = resources.options.field.clone();
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
}

impl geng::State for Game {
    fn update(&mut self, delta_time: f64) {
        self.resources.delta_time = delta_time as Time;
        self.resources.game_time += self.resources.delta_time;

        self.systems
            .iter_mut()
            .for_each(|s| s.update(&mut self.world, &mut self.resources));
        self.resources.down_keys.clear();
        self.resources.down_mouse_buttons.clear();
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

            geng::Event::MouseDown {
                position: _,
                button,
            } => {
                self.resources.down_mouse_buttons.insert(button);
                self.resources.pressed_mouse_buttons.insert(button);
            }

            geng::Event::MouseUp {
                position: _,
                button,
            } => {
                self.resources.pressed_mouse_buttons.remove(&button);
            }

            _ => {}
        }
    }

    fn ui<'a>(&'a mut self, cx: &'a ui::Controller) -> Box<dyn ui::Widget + 'a> {
        let widgets = self
            .systems
            .iter_mut()
            .map(|system| system.ui(cx, &mut self.world, &mut self.resources))
            .collect_vec();
        if widgets.is_empty() {
            return Box::new(ui::Void);
        }
        Box::new(geng::ui::stack(widgets))
    }

    fn draw(&mut self, framebuffer: &mut ugli::Framebuffer) {
        self.resources.mouse_pos = self.resources.camera.screen_to_world(
            framebuffer.size().map(|x| x as f32),
            self.resources.geng.window().mouse_pos().map(|x| x as f32),
        );
        ugli::clear(framebuffer, Some(Rgba::BLACK), None, None);
        self.systems
            .iter()
            .for_each(|s| s.draw(&self.world, &mut self.resources, framebuffer));
    }
}
