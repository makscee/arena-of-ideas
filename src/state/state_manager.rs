use super::*;

pub struct StateManager {
    stack: Vec<Box<dyn State>>,
}

impl StateManager {
    #[allow(clippy::new_without_default)]
    pub fn new() -> Self {
        Self { stack: Vec::new() }
    }
    pub fn switch(&mut self, state: Box<dyn State>) {
        *self.stack.last_mut().unwrap() = state;
    }
    pub fn push(&mut self, state: Box<dyn State>) {
        self.stack.push(state);
    }
    pub fn pop(&mut self) {
        self.stack.pop();
    }
    pub fn current_state(&mut self) -> Option<&mut dyn State> {
        self.stack.last_mut().map(|state| state.deref_mut())
    }
}

impl State for StateManager {
    fn update(&mut self, delta_time: Time, logic: &mut Logic, view: &mut View) {
        if let Some(state) = self.current_state() {
            state.update(delta_time, logic, view);
            if let Some(transition) = state.transition(logic, view) {
                match transition {
                    Transition::Pop => self.pop(),
                    Transition::Push(state) => self.push(state),
                    Transition::Switch(state) => self.switch(state),
                }
            }
        }
    }
    fn handle_event(&mut self, event: Event, logic: &mut Logic) {
        if let Some(state) = self.current_state() {
            state.handle_event(event, logic);
        }
    }
    fn draw(&mut self, framebuffer: &mut ugli::Framebuffer, view: &View, logic: &Logic) {
        if let Some(state) = self.current_state() {
            state.draw(framebuffer, view, logic);
        }
    }
    fn ui<'a>(&'a mut self, cx: &'a ui::Controller) -> Box<dyn ui::Widget + 'a> {
        match self.current_state() {
            Some(state) => state.ui(cx),
            None => Box::new(ui::Void),
        }
    }
}
