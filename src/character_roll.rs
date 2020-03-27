use crate::character::{AttributeName, Character, ActionName};
use crate::roll::Roll;
use regex::Regex;
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
            Check::Action(name, bonus) => character.action(name)?.rating + bonus,
        };
        Some(Roll::new(rating).unwrap())
    }
}

#[derive(Debug)]
pub enum Check {
    Attribute(AttributeName),
    Action(ActionName, usize),
}

impl Check {
    pub fn parse(string: &str) -> Option<Check> {
        lazy_static! {
            static ref RE: Regex =
                Regex::new(r"^(.*?)(?: with (\d+) bonus dice)?$").unwrap();
        }

        AttributeName::parse(string)
            .map(Check::Attribute)
            .or_else(|| 
                RE.captures(string).and_then(|captures| {
                    let action = ActionName::parse(captures.get(1)?.as_str())?;
                    let bonus = captures.get(2)?.as_str().parse::<usize>().ok()?;
                    Some(Check::Action(action, bonus))
                })
            )
    }
}

impl fmt::Display for Check {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Check::Attribute(name) => name.as_str().fmt(f),
            Check::Action(name, _) => name.as_str().fmt(f),
        }
    }
}
