use super::*;

mod battle;
mod game;
mod main_menu;
mod shader_editor;
mod state_manager;

pub use battle::*;
pub use game::*;
pub use main_menu::*;
pub use shader_editor::*;
pub use state_manager::*;

pub enum Transition {
    /// Pops (removes) the current state from the state stack.
    Pop,
    /// Replaces the current state with another state.
    Switch(Box<dyn State>),
    /// Pushes a new state on the state stack.
    Push(Box<dyn State>),
}

pub trait State: 'static {
    fn init(&mut self, logic: &mut Logic, view: &mut View) {
        #![allow(unused_variables)]
    }
    fn update(&mut self, delta_time: Time, logic: &mut Logic, view: &mut View) {
        #![allow(unused_variables)]
    }
    fn draw(&mut self, framebuffer: &mut ugli::Framebuffer, view: &View, logic: &Logic) {
        #![allow(unused_variables)]
    }
    fn transition(&mut self, logic: &mut Logic) -> Option<Transition> {
        None
    }
    fn handle_event(&mut self, event: Event, logic: &mut Logic) {
        #![allow(unused_variables)]
    }
    fn ui<'a>(&'a mut self, cx: &'a ui::Controller) -> Box<dyn ui::Widget + 'a> {
        #![allow(unused_variables)]
        Box::new(ui::Void)
    }
}
