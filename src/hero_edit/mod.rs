use std::env;

use sciter::{dispatch_script_call, make_args};

use super::*;

#[derive(clap::Args)]
pub struct HeroEditor {}

impl HeroEditor {
    pub fn run(self) {
        println!("Editor run");

        sciter::set_options(sciter::RuntimeOptions::ScriptFeatures(
            sciter::SCRIPT_RUNTIME_FEATURES::ALLOW_SYSINFO as u8
                | sciter::SCRIPT_RUNTIME_FEATURES::ALLOW_FILE_IO as u8
                | sciter::SCRIPT_RUNTIME_FEATURES::ALLOW_SOCKET_IO as u8
                | sciter::SCRIPT_RUNTIME_FEATURES::ALLOW_EVAL as u8,
        ))
        .unwrap();
        sciter::set_options(sciter::RuntimeOptions::DebugMode(true)).unwrap();

        let dir = env::current_dir().unwrap().as_path().display().to_string();
        let filename = format!("{}/{}", dir, "resources/index.htm");
        println!("Full filename with path of index.htm: {}", filename);

        let mut frame = sciter::Window::new();
        frame
            .set_options(sciter::window::Options::DebugMode(true))
            .unwrap();
        frame.event_handler(Handler);
        frame.load_file(&filename);
        let hwnd = frame.get_hwnd();
        frame.run_app();

        use sciter::{Element, Value};

        let root = Element::from_window(hwnd).unwrap();
        let panel = root.find_first("panel").expect("panel not found").unwrap();
        let value = panel.call_method("getValue", &make_args!()).unwrap();

        debug!("value = {}", value);

        // let result: Value = root
        //     .call_function("namespace.name", &make_args!(1, "2", 3))
        //     .unwrap();
    }
}

struct Handler;

impl Handler {
    fn calc_sum(&self, a: i32, b: i32) -> i32 {
        a + b
    }
}

impl sciter::EventHandler for Handler {
    dispatch_script_call! {
      fn calc_sum(i32, i32);
    }
}
