use colored::Colorize;
use strum_macros::AsRefStr;

use super::*;

#[derive(Default, Debug)]
pub struct Logger {
    enabled: bool,
    enabled_contexts: HashSet<LogContext>,
}

#[derive(Clone, Copy, PartialEq, Eq, Ord, PartialOrd, Debug, Deserialize, Hash, AsRefStr)]
pub enum LogContext {
    Action,
    ActionFail,
    Effect,
    Condition,
    Expression,
    Trigger,
    Event,
    UnitCreation,
    Measurement,
    Contexts,
    Test,
}

impl Display for LogContext {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let text = {
            let text = self.as_ref();
            match self {
                LogContext::Action => text.bright_red(),
                LogContext::ActionFail => text.red(),
                LogContext::Effect => text.cyan(),
                LogContext::Condition => text.blue(),
                LogContext::Expression => text.blue(),
                LogContext::Trigger => text.on_green(),
                LogContext::Event => text.green(),
                LogContext::UnitCreation => text.purple(),
                LogContext::Measurement => text.bright_cyan(),
                LogContext::Contexts => text.bright_blue(),
                LogContext::Test => text.bright_purple(),
            }
            .bold()
        };
        write!(f, "{text}")
    }
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

    pub fn log<S>(&self, text: S, context: &LogContext)
    where
        S: FnOnce() -> String,
    {
        if self.is_context_enabled(context) {
            println!("{}: {}", context, text());
        }
    }

    pub fn is_context_enabled(&self, context: &LogContext) -> bool {
        self.enabled && self.enabled_contexts.contains(context)
    }

    pub fn set_enabled(&mut self, value: bool) {
        self.enabled = value;
    }
}
