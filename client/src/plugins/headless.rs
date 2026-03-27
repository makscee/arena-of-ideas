use super::*;
use bevy::render::view::screenshot::{save_to_disk, Screenshot};

pub struct HeadlessPlugin;

impl Plugin for HeadlessPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(HeadlessState::default())
            .add_systems(Update, headless_screenshot_system);
    }
}

#[derive(Resource)]
struct HeadlessState {
    frame: u32,
    captured: bool,
}

impl Default for HeadlessState {
    fn default() -> Self {
        Self {
            frame: 0,
            captured: false,
        }
    }
}

fn headless_screenshot_system(
    mut commands: Commands,
    mut state: ResMut<HeadlessState>,
    args: Res<HeadlessArgs>,
) {
    state.frame += 1;
    if state.captured {
        if state.frame > args.wait_frames + 10 {
            info!("Headless screenshot saved, exiting");
            std::process::exit(0);
        }
        return;
    }
    if state.frame >= args.wait_frames {
        let output = args.output.clone();
        info!("Capturing screenshot to {output} at frame {}", state.frame);
        commands
            .spawn(Screenshot::primary_window())
            .observe(save_to_disk(output));
        state.captured = true;
    }
}

#[derive(Resource, Clone)]
pub struct HeadlessArgs {
    pub wait_frames: u32,
    pub output: String,
}
