use crate::character::{AttributeName, Character, ActionName};
use crate::roll::Roll;
use std::fmt;

#[derive(Debug)]
pub struct CharacterRoll {
    pub check: Check,
}

impl CharacterRoll {
    pub fn parse(string: &str) -> Option<CharacterRoll> {
        let check = Check::parse(string)?;
        Some(CharacterRoll { check })
    }

    pub fn to_roll(&self, character: &Character) -> Option<Roll> {
        let rating = match self.check {
            Check::Attribute(name) => character.attribute(name)?.rating,
            Check::Action(name) => character.action(name)?.rating,
        };
        Some(Roll::new(rating).unwrap())
    }
}

#[derive(Debug)]
pub enum Check {
    Attribute(AttributeName),
    Action(ActionName),
}

impl Check {
    pub fn parse(string: &str) -> Option<Check> {
        AttributeName::parse(string)
            .map(Check::Attribute)
            .or_else(|| ActionName::parse(string).map(Check::Action))
    }
}

impl fmt::Display for Check {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Check::Attribute(name) => name.as_str().fmt(f),
            Check::Action(name) => name.as_str().fmt(f),
        }
    }
}
