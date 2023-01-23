use geng::{prelude::*, *};

mod components;
mod game;
mod systems;

use components::*;
use game::*;
use legion::Read;
use legion::Write;
use legion::*;
use systems::*;
use ugli::*;

type Id = i64;
type Time = f32;

fn setup_geng() -> Geng {
    geng::setup_panic_handler();
    let geng = Geng::new_with(geng::ContextOptions {
        title: "Arena of Ideas".to_owned(),
        shader_prefix: Some((
            include_str!("vertex_prefix.glsl").to_owned(),
            include_str!("fragment_prefix.glsl").to_owned(),
        )),
        target_ui_resolution: Some(vec2(1920.0, 1080.0)),
        ..default()
    });
    geng
}

fn main() {
    logger::init().unwrap();
    geng::setup_panic_handler();
    let geng = setup_geng();

    // geng::run(
    //     &geng,
    //     ShaderEditState::new(&geng, assets.clone(), Rc::new(view)),
    // );
    let mut world = World::default();
    // world.push((
    //     EcsPosition { x: 0.0, y: 0.0 },
    //     EcsShaderProgram {
    //         shader: SystemShader::Unit,
    //         parameters: default(),
    //         vertices: default(),
    //         instances: default(),
    //     },
    // ));

    // #[system(for_each)]
    // fn draw_units(
    //     pos: &EcsPosition,
    //     shader: &EcsShaderProgram,
    //     #[resource] system_shaders: &SystemShaders,
    // ) {
    //     debug!("draw_units called");
    // }

    // let schedule = Schedule::builder()
    //     .add_thread_local(draw_units_system())
    //     .build();

    let mut resources = Resources::default();
    world.push((GameState::MainMenu,));
    // resources.insert(assets.system_shaders.clone());

    // game.state_manager.push(Box::new(MainMenu {
    //     assets: game.assets.clone(),
    //     transition: false,
    // }));
    let game = Game::new(geng.clone(), world, resources);
    geng::run(&geng, game);
}
