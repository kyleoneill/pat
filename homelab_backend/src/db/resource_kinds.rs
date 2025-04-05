use std::fmt::{Display, Formatter, Result};

#[derive(Debug)]
pub enum ResourceKind {
    Reminder,
    ReminderCategory,
    User,
    ConnectionsGame,
    Log,
}

impl Display for ResourceKind {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        let thing = match self {
            ResourceKind::Reminder => "Reminder",
            ResourceKind::ReminderCategory => "Reminder Category",
            ResourceKind::User => "User",
            ResourceKind::ConnectionsGame => "Connections Game",
            ResourceKind::Log => "Log",
        };
        write!(f, "{}", thing)
    }
}
