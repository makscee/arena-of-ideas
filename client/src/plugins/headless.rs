use std::time::Instant;

use super::*;
use crate::plugins::connect::ConnectPlugin;
use bevy::render::view::screenshot::{save_to_disk, Screenshot};

pub struct HeadlessPlugin;

impl Plugin for HeadlessPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(HeadlessState::default())
            .add_systems(Update, headless_system);
    }
}

#[derive(Clone, PartialEq, Eq, Debug)]
pub enum HeadlessMode {
    Screenshot,
    GameplayTest,
}

#[derive(Resource, Clone)]
pub struct HeadlessArgs {
    pub mode: HeadlessMode,
    pub wait_frames: u32,
    pub output: String,
    pub timeout_secs: u64,
}

#[derive(Resource)]
struct HeadlessState {
    frame: u32,
    screenshot_captured: bool,
    last_game_state: Option<GameState>,
    test_phase: TestPhase,
    start_time: Instant,
    screenshots_taken: Vec<String>,
    phase_frame: u32,
    auth_attempts: u32,
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum TestPhase {
    WaitingForTitle,
    StartingMatch,
    InShop,
    BuyingUnits { bought: u8 },
    StartingBattle,
    InBattle,
    MatchOver,
    Done,
    Failed(String),
}

impl Default for HeadlessState {
    fn default() -> Self {
        Self {
            frame: 0,
            screenshot_captured: false,
            last_game_state: None,
            test_phase: TestPhase::WaitingForTitle,
            start_time: Instant::now(),
            screenshots_taken: Vec::new(),
            phase_frame: 0,
            auth_attempts: 0,
        }
    }
}

fn headless_system(world: &mut World) {
    let frame = world.resource::<HeadlessState>().frame + 1;
    world.resource_mut::<HeadlessState>().frame = frame;

    let args = world.resource::<HeadlessArgs>().clone();
    let elapsed = world.resource::<HeadlessState>().start_time.elapsed();

    if elapsed.as_secs() > args.timeout_secs {
        let phase = world.resource::<HeadlessState>().test_phase.clone();
        finish_test(
            world,
            false,
            &format!("Timeout after {}s in phase {:?}", args.timeout_secs, phase),
        );
        return;
    }

    match args.mode {
        HeadlessMode::Screenshot => screenshot_mode(world, &args),
        HeadlessMode::GameplayTest => gameplay_test_mode(world, &args),
    }
}

fn screenshot_mode(world: &mut World, args: &HeadlessArgs) {
    let frame = world.resource::<HeadlessState>().frame;
    let captured = world.resource::<HeadlessState>().screenshot_captured;

    if captured {
        if frame > args.wait_frames + 10 {
            info!("Headless screenshot saved, exiting");
            std::process::exit(0);
        }
        return;
    }
    if frame >= args.wait_frames {
        take_screenshot(world, &args.output);
        world.resource_mut::<HeadlessState>().screenshot_captured = true;
    }
}

fn gameplay_test_mode(world: &mut World, args: &HeadlessArgs) {
    let current_state = cur_state(world);
    let last_state = world.resource::<HeadlessState>().last_game_state;
    let frame = world.resource::<HeadlessState>().frame;

    // Detect state transitions and screenshot them
    if last_state != Some(current_state) {
        info!(
            "[test-flow] State transition: {:?} -> {:?}",
            last_state, current_state
        );
        world.resource_mut::<HeadlessState>().last_game_state = Some(current_state);
        world.resource_mut::<HeadlessState>().phase_frame = 0;

        // Screenshot every state transition after a few frames for rendering to settle
        let output_dir = std::path::Path::new(&args.output)
            .parent()
            .unwrap_or(std::path::Path::new("screenshots"));
        let screenshot_path = output_dir
            .join(format!("state_{}.png", current_state.to_string().to_lowercase()))
            .to_string_lossy()
            .to_string();
        // Delay screenshot by a few frames so the UI renders
        schedule_screenshot(world, screenshot_path, 10);
    }

    let phase_frame = world.resource::<HeadlessState>().phase_frame + 1;
    world.resource_mut::<HeadlessState>().phase_frame = phase_frame;

    let phase = world.resource::<HeadlessState>().test_phase.clone();

    match phase {
        TestPhase::WaitingForTitle => {
            // Auto-handle auth: connect anonymously for test-flow
            if current_state == GameState::Auth && phase_frame > 5 {
                let auth_attempts = world.resource::<HeadlessState>().auth_attempts;
                if auth_attempts > 0 {
                    if phase_frame > 60 * 10 {
                        finish_test(
                            world,
                            false,
                            "Auth/connect timed out after anonymous connection attempt.",
                        );
                    }
                    return; // Wait for async connect callback
                }
                world.resource_mut::<HeadlessState>().auth_attempts += 1;
                info!("[test-flow] Connecting anonymously for test-flow...");
                // Set dummy token to satisfy AuthOption check
                world.insert_resource(AuthOption {
                    id_token: Some("anonymous".to_string()),
                });
                // Connect with None token — SDK will create/reuse anonymous identity
                ConnectPlugin::connect(None, |_, identity, token| {
                    info!("[test-flow] Connected anonymously as {identity}");
                    let token = token.to_owned();
                    save_player_identity(identity);
                    op(move |world| {
                        ConnectOption { identity, token }.save(world);
                        GameState::proceed(world);
                    });
                });
                if is_connected() {
                    let _ = cn().reducers.login_by_identity();
                }
            }
            // Auto-handle registration: if at Login and player doesn't exist, register
            if current_state == GameState::Login && phase_frame == 60 {
                if let Some(ld) = world.get_resource::<LoginData>() {
                    if !ld.user_exists {
                        let name = format!(
                            "bot{}",
                            std::time::SystemTime::now()
                                .duration_since(std::time::UNIX_EPOCH)
                                .unwrap()
                                .as_secs()
                                % 1000000
                        );
                        info!("[test-flow] Player not found, auto-registering as '{name}'...");
                        cn().reducers
                            .register_then(name, |_ctx, result| {
                                match result {
                                    Ok(Ok(())) => {
                                        info!("[test-flow] Registration successful, completing login...");
                                        LoginPlugin::complete();
                                    }
                                    Ok(Err(e)) => {
                                        error!("[test-flow] Registration error: {e}");
                                    }
                                    Err(e) => {
                                        error!("[test-flow] Registration internal error: {e:?}");
                                    }
                                }
                            })
                            .ok();
                    }
                }
            }
            if current_state == GameState::Title {
                info!("[test-flow] Reached Title, starting match...");
                world.resource_mut::<HeadlessState>().test_phase = TestPhase::StartingMatch;
            }
        }
        TestPhase::StartingMatch => {
            if phase_frame < 30 {
                return; // Wait for UI to settle
            }
            // First abandon any existing match, then create new one
            info!("[test-flow] Abandoning any existing match...");
            let _ = cn().reducers.match_abandon();
            info!("[test-flow] Calling match_insert_then...");
            match cn().reducers.match_insert_then(|_ctx, result| {
                match result {
                    Ok(Ok(())) => {
                        info!("[test-flow] match_insert succeeded, transitioning to Shop");
                        GameState::Shop.set_next_op();
                    }
                    Ok(Err(e)) => {
                        error!("[test-flow] match_insert server error: {e}");
                        // Might already have an active match, try going to Shop
                        GameState::Shop.set_next_op();
                    }
                    Err(e) => {
                        error!("[test-flow] match_insert internal error: {e:?}");
                    }
                }
            }) {
                Ok(()) => info!("[test-flow] match_insert_then called"),
                Err(e) => info!("[test-flow] match_insert_then error: {e:?}"),
            }
            world.resource_mut::<HeadlessState>().test_phase = TestPhase::InShop;
        }
        TestPhase::InShop => {
            if current_state == GameState::Shop {
                info!("[test-flow] Reached Shop, buying units...");
                world.resource_mut::<HeadlessState>().test_phase =
                    TestPhase::BuyingUnits { bought: 0 };
            }
        }
        TestPhase::BuyingUnits { bought } => {
            if current_state != GameState::Shop {
                return;
            }
            if phase_frame < 30 {
                return; // Wait for shop to load
            }
            if bought < 1 {
                info!("[test-flow] Buying unit at shop index {bought}...");
                match cn().reducers.match_shop_buy(bought) {
                    Ok(()) => info!("[test-flow] Bought unit {bought}"),
                    Err(e) => info!("[test-flow] Buy error: {e:?}"),
                }
                world.resource_mut::<HeadlessState>().test_phase =
                    TestPhase::BuyingUnits { bought: bought + 1 };
                world.resource_mut::<HeadlessState>().phase_frame = 0;
            } else {
                info!("[test-flow] Done buying, starting battle...");
                world.resource_mut::<HeadlessState>().test_phase = TestPhase::StartingBattle;
                world.resource_mut::<HeadlessState>().phase_frame = 0;
            }
        }
        TestPhase::StartingBattle => {
            if phase_frame < 30 {
                return;
            }
            info!("[test-flow] Calling match_start_battle...");
            match cn()
                .reducers
                .match_start_battle_then(|_ctx, result| match result {
                    Ok(Ok(())) => {
                        info!("[test-flow] Battle started on server, transitioning to Battle");
                        GameState::Battle.set_next_op();
                    }
                    Ok(Err(e)) => {
                        error!("[test-flow] start_battle server error: {e}");
                    }
                    Err(e) => {
                        error!("[test-flow] start_battle internal error: {e:?}");
                    }
                }) {
                Ok(()) => info!("[test-flow] match_start_battle_then called"),
                Err(e) => info!("[test-flow] start_battle call error: {e:?}"),
            }
            world.resource_mut::<HeadlessState>().test_phase = TestPhase::InBattle;
        }
        TestPhase::InBattle => {
            if current_state == GameState::Battle {
                // Wait for battle to finish
                if phase_frame > 60 * 10 {
                    info!("[test-flow] Battle taking too long, checking state...");
                }
            }
            if current_state == GameState::Shop {
                info!("[test-flow] Battle complete, back to shop. Test PASSED!");
                world.resource_mut::<HeadlessState>().test_phase = TestPhase::Done;
            }
            if current_state == GameState::MatchOver {
                info!("[test-flow] Match over. Test PASSED!");
                world.resource_mut::<HeadlessState>().test_phase = TestPhase::MatchOver;
            }
        }
        TestPhase::MatchOver => {
            if phase_frame > 30 {
                finish_test(world, true, "Full gameplay flow completed successfully");
            }
        }
        TestPhase::Done => {
            if phase_frame > 30 {
                finish_test(
                    world,
                    true,
                    "Battle completed, returned to shop successfully",
                );
            }
        }
        TestPhase::Failed(ref reason) => {
            finish_test(world, false, reason);
        }
    }
}

fn take_screenshot(world: &mut World, path: &str) {
    info!("[headless] Capturing screenshot: {path}");
    world
        .commands()
        .spawn(Screenshot::primary_window())
        .observe(save_to_disk(path.to_string()));
    world
        .resource_mut::<HeadlessState>()
        .screenshots_taken
        .push(path.to_string());
}

fn schedule_screenshot(world: &mut World, path: String, _delay_frames: u32) {
    // Take screenshot immediately - Bevy's screenshot system handles the timing
    take_screenshot(world, &path);
}

fn finish_test(world: &mut World, success: bool, message: &str) {
    let screenshots = world
        .resource::<HeadlessState>()
        .screenshots_taken
        .clone();
    let elapsed = world
        .resource::<HeadlessState>()
        .start_time
        .elapsed()
        .as_secs_f32();

    if success {
        info!("========================================");
        info!("[test-flow] PASSED: {message}");
        info!("[test-flow] Time: {elapsed:.1}s");
        info!("[test-flow] Screenshots: {}", screenshots.len());
        for s in &screenshots {
            info!("  - {s}");
        }
        info!("========================================");
    } else {
        error!("========================================");
        error!("[test-flow] FAILED: {message}");
        error!("[test-flow] Time: {elapsed:.1}s");
        error!("[test-flow] Screenshots: {}", screenshots.len());
        for s in &screenshots {
            error!("  - {s}");
        }
        error!("========================================");
    }

    // Take a final screenshot before exiting
    take_screenshot(world, "screenshots/final.png");

    // Exit with appropriate code
    std::process::exit(if success { 0 } else { 1 });
}
