use super::*;

#[derive(Default)]
struct Scope {
    time: f64,
    calls: usize,
    children: HashMap<&'static str, Scope>,
}

impl Display for Scope {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fn format(scope: &Scope, ident: usize, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            let mut names: Vec<_> = scope.children.keys().collect();
            names.sort_by_key(|&name| r64(scope.children[name].time));
            names.reverse();
            for name in names {
                for _ in 0..ident {
                    write!(f, " ")?;
                }
                let child = &scope.children[name];
                writeln!(
                    f,
                    "- {}: {:.2?} ({}%), {} calls, {:.2?} avg",
                    name,
                    std::time::Duration::from_secs_f64(child.time),
                    (child.time / scope.time * 100.0) as i32,
                    child.calls,
                    std::time::Duration::from_secs_f64(child.time / child.calls as f64),
                )?;
                format(child, ident + 2, f)?;
            }
            Ok(())
        }
        format(self, 0, f)
    }
}

#[derive(Default)]
struct State {
    current_path: Vec<&'static str>,
    root: Scope,
}

impl State {
    fn current_scope_mut(&mut self) -> &mut Scope {
        let mut scope = &mut self.root;
        for &name in &self.current_path {
            scope = scope.children.get_mut(name).unwrap();
        }
        scope
    }
    fn start_scope(&mut self, name: &'static str) {
        self.current_scope_mut()
            .children
            .entry(name)
            .or_default()
            .calls += 1;
        self.current_path.push(name);
    }
    fn end_scope(&mut self, time: f64) {
        self.current_scope_mut().time += time;
        self.current_path.pop();
    }
}

pub struct TimeTracker {
    state: Arc<Mutex<State>>,
}

pub struct TimeTrackerScope<'a> {
    timer: Timer,
    state: Arc<Mutex<State>>,
    phantom_data: PhantomData<&'a ()>,
}

impl TimeTracker {
    pub fn new() -> Self {
        Self { state: default() }
    }
    pub fn track(&self, name: &'static str) -> TimeTrackerScope {
        self.state.lock().unwrap().start_scope(name);
        TimeTrackerScope {
            timer: Timer::new(),
            state: self.state.clone(),
            phantom_data: PhantomData,
        }
    }
}

impl Drop for TimeTrackerScope<'_> {
    fn drop(&mut self) {
        let mut state = self.state.lock().unwrap();
        state.end_scope(self.timer.elapsed());
    }
}

impl Drop for TimeTracker {
    fn drop(&mut self) {
        let mut state = self.state.lock().unwrap();
        assert!(state.current_path.is_empty());
        state.root.time = state.root.children.values().map(|child| child.time).sum();
        debug!("Timings: {:.2}\n{}", state.root.time, state.root);
    }
}
