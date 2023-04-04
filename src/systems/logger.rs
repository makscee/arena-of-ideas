use super::*;

#[derive(Default, Debug)]
pub struct Logger {
    enabled: bool,
    enabled_contexts: HashSet<LogContext>,
}

#[derive(Clone, Copy, PartialEq, Eq, Ord, PartialOrd, Debug, Deserialize, Hash)]
pub enum LogContext {
    Action,
    ActionFail,
    Effect,
    Condition,
    Trigger,
    Event,
    UnitCreation,
    Measurement,
    Contexts,
    Test,
}

impl Logger {
    pub fn load(&mut self, options: &Options) {
        self.enabled = true;
        self.enabled_contexts = HashSet::from_iter(options.log.iter().filter_map(
            |(context, value)| match value {
                true => Some(*context),
                false => None,
            },
        ));
        debug!("Load logger {:?}", self);
    }

    pub fn log(&self, text: &str, context: &LogContext) {
        if self.is_context_enabled(context) {
            println!("{:?}: {}", context, text);
        }
    }

    pub fn is_context_enabled(&self, context: &LogContext) -> bool {
        self.enabled && self.enabled_contexts.contains(context)
    }

    pub fn set_enabled(&mut self, value: bool) {
        self.enabled = value;
    }
}
