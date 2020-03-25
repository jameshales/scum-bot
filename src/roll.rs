use rand::distributions::{Distribution, Uniform};
use rand::Rng;
use regex::Regex;
use std::error;
use std::fmt;

/// The maximum number of dice that may be rolled at one time.
pub const MAXIMUM_ROLLS: usize = 100;

/// The maximum number of individual dice rolls that will be displayed in full.
pub const MAXIMUM_ROLLS_DISPLAY: usize = 10;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum RollOperation {
    Min,
    Max
}

impl fmt::Display for RollOperation {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            RollOperation::Min => write!(f, "min"),
            RollOperation::Max => write!(f, "max"),
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum RollOutcome {
    CriticalSuccess,
    FullSuccess,
    PartialSuccess,
    BadOutcome,
}

impl RollOutcome {
    fn from_result(result: i32, critical: bool) -> RollOutcome {
        if critical {
            RollOutcome::CriticalSuccess
        } else if result >= 6 {
            RollOutcome::FullSuccess
        } else if result >= 4 {
            RollOutcome::PartialSuccess
        } else {
            RollOutcome::BadOutcome
        }
    }
}

impl fmt::Display for RollOutcome {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            RollOutcome::CriticalSuccess => write!(f, " — Critical Success 🤩"),
            RollOutcome::FullSuccess => write!(f, " — Full Success 😄"),
            RollOutcome::PartialSuccess => write!(f, " — Partial Success 😑"),
            RollOutcome::BadOutcome => write!(f, " — Bad Outcome 😰"),
        }
    }
}

/// A dice roll that might occur in Scum and Villainy.
///
/// A dice roll involves rolling a number of six-sided dice. The highest value die rolled
/// determines the outcome of the roll.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct Roll {
    rolls: usize,
}

/// The detailed result of a dice roll.
///
/// In addition to the numerical result itself, it includes the individual die values, and whether
/// the roll was a critical success, so that this information can be presented to the user.
#[derive(Debug, Eq, PartialEq)]
pub struct RollResult {
    result: i32,
    operation: RollOperation,
    dice: Vec<i32>,
    outcome: RollOutcome,
}

impl fmt::Display for RollResult {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "**{}**", self.result).and({
            if self.dice.len() > 1 {
                let mut iter = self.dice.iter().take(MAXIMUM_ROLLS_DISPLAY);
                iter.next().map_or(Ok(()), |head| {
                    iter.fold(write!(f, " = {}({}", self.operation, head), |result, die| {
                        result.and(write!(f, ", {}", die))
                    })
                    .and(if self.dice.len() > MAXIMUM_ROLLS_DISPLAY {
                        write!(f, ", …")
                    } else {
                        Ok(())
                    })
                    .and(write!(f, ")"))
                })
            } else {
                Ok(())
            }
        })
        .and(self.outcome.fmt(f))
    }
}

/// Represents an error that might occur when creating a roll.
///
/// A roll must have involve a positive number of rolls of dice.
/// The number of rolls must not be more than 100.
#[derive(Debug, Eq, PartialEq)]
pub enum Error {
    RollsTooGreat,
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Error::RollsTooGreat => write!(f, "Must roll no more than 100 dice."),
        }
    }
}

impl error::Error for Error {
    fn source(&self) -> Option<&(dyn error::Error + 'static)> {
        None
    }
}

/// Represents an error that might occur when parsing a roll from a String.
#[derive(Debug, Eq, PartialEq)]
pub enum ParserError {
    InvalidSyntax,
    InvalidValue(Error),
}

impl fmt::Display for ParserError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ParserError::InvalidSyntax => write!(f, "Invalid syntax."),
            ParserError::InvalidValue(e) => e.fmt(f),
        }
    }
}

impl Roll {
    /// Create a roll, validating that the number of dice being rolled are no more than the maximum
    /// allowed value.
    pub fn new(rolls: usize) -> Result<Roll, Error> {
        if rolls > MAXIMUM_ROLLS {
            Err(Error::RollsTooGreat)
        } else {
            Ok(Roll::new_unsafe(rolls))
        }
    }

    pub const fn new_unsafe(rolls: usize) -> Roll {
        Roll {
            rolls,
        }
    }

    /// Parse a roll from a String using conventional Scum and Villainy syntax.
    pub fn parse(string: &str) -> Result<Roll, ParserError> {
        lazy_static! {
            static ref RE: Regex = Regex::new(r"^(\d+)d$").unwrap();
        }
        Roll::parse_regex(&RE, string)
    }

    fn parse_regex(regex: &Regex, string: &str) -> Result<Roll, ParserError> {
        regex
            .captures(string)
            .and_then(|captures| {
                captures
                    .get(1)
                    .and_then(|m| m.as_str().parse::<usize>().ok())
            })
            .ok_or(ParserError::InvalidSyntax)
            .and_then(|rolls| {
                Roll::new(rolls).map_err(ParserError::InvalidValue)
            })
    }

    pub fn roll<R: Rng + ?Sized>(&self, rng: &mut R) -> RollResult {
        if self.rolls > 0 {
            let dice = Roll::roll_once_component(self.rolls, rng);
            let result = *(dice.iter().max().unwrap_or(&1));
            let critical = dice.iter().filter(|r| **r == 6).count() > 1;
            let outcome = RollOutcome::from_result(result, critical);
            RollResult {
                result,
                operation: RollOperation::Max,
                dice,
                outcome,
            }
        } else {
            let dice = Roll::roll_once_component(2, rng);
            let result = *(dice.iter().min().unwrap_or(&1));
            let outcome = RollOutcome::from_result(result, false);
            RollResult {
                result,
                operation: RollOperation::Min,
                dice,
                outcome,
            }
        }
    }

    fn roll_once_component<R: Rng + ?Sized>(rolls: usize, rng: &mut R) -> Vec<i32> {
        Uniform::new_inclusive(1, 6)
            .sample_iter(rng)
            .take(rolls)
            .collect()
    }
}

impl fmt::Display for Roll {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}d", self.rolls)
    }
}
