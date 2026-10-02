use std::fmt;

/// Everything that can make `slp` exit unsuccessfully.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Failure {
    MissingOperand,
    InvalidInterval(String),
    InvalidTimeOfDay(String),
    InvalidOption(String),
    MissingUntilValue,
    DuplicateUntil,
    UntilWithInterval,
    LocalTimeUnavailable,
}

impl Failure {
    /// Usage mistakes get a pointer to `--help`; runtime failures do not.
    #[must_use]
    pub fn is_usage(&self) -> bool {
        !matches!(self, Self::LocalTimeUnavailable)
    }
}

impl fmt::Display for Failure {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::MissingOperand => write!(f, "missing operand"),
            Self::InvalidInterval(text) => write!(f, "invalid time interval '{text}'"),
            Self::InvalidTimeOfDay(text) => {
                write!(
                    f,
                    "invalid time of day '{text}' (expected HH:MM or HH:MM:SS)"
                )
            }
            Self::InvalidOption(text) => write!(f, "invalid option '{text}'"),
            Self::MissingUntilValue => write!(f, "option '--until' requires a time of day"),
            Self::DuplicateUntil => write!(f, "option '--until' given more than once"),
            Self::UntilWithInterval => {
                write!(f, "'--until' cannot be combined with a time interval")
            }
            Self::LocalTimeUnavailable => write!(f, "cannot resolve the local time zone"),
        }
    }
}
