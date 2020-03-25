use rusqlite::Result as RusqliteResult;
use rusqlite::{Connection, OptionalExtension, Row};
use serenity::model::id::{ChannelId, UserId};
use std::convert::TryInto;

/// A character in a Scum and Villainy campaign.
///
/// The character has a number of action ratings, from which action and resistance rolls are
/// calculated.
#[derive(Debug, Eq, PartialEq)]
pub struct Character {
    // Action ratings
    attune: i32,
    command: i32,
    consort: i32,
    doctor: i32,
    hack: i32,
    helm: i32,
    rig: i32,
    scramble: i32,
    scrap: i32,
    skulk: i32,
    study: i32,
    sway: i32,
}

impl Character {
    pub fn get(
        connection: &Connection,
        channel_id: ChannelId,
        user_id: UserId,
    ) -> RusqliteResult<Option<Character>> {
        connection
            .query_row(
                "SELECT \
                 attune, \
                 command, \
                 consort, \
                 doctor, \
                 hack, \
                 helm, \
                 rig, \
                 scramble, \
                 scrap, \
                 skulk, \
                 study, \
                 sway \
                 FROM characters \
                 WHERE channel_id = $1 \
                 AND user_id = $2",
                &[&channel_id.to_string(), &user_id.to_string()],
                Character::from_row,
            )
            .optional()
    }

    pub fn from_row(row: &Row) -> RusqliteResult<Character> {
        Ok(Character {
            attune: row.get("attune")?,
            command: row.get("command")?,
            consort: row.get("consort")?,
            doctor: row.get("doctor")?,
            hack: row.get("hack")?,
            helm: row.get("helm")?,
            rig: row.get("rig")?,
            scramble: row.get("scramble")?,
            scrap: row.get("scrap")?,
            skulk: row.get("skulk")?,
            study: row.get("study")?,
            sway: row.get("sway")?,
        })
    }

    // Attributes

    pub fn attribute(&self, name: AttributeName) -> Option<AttributeRating> {
        match name {
            AttributeName::Insight => self.insight(),
            AttributeName::Prowess => self.prowess(),
            AttributeName::Resolve => self.resolve(),
        }
    }

    pub fn insight(&self) -> Option<AttributeRating> {
        Character::make_attribute(&[self.doctor, self.hack, self.rig, self.study])
    }

    pub fn prowess(&self) -> Option<AttributeRating> {
        Character::make_attribute(&[self.helm, self.scramble, self.scrap, self.skulk])
    }

    pub fn resolve(&self) -> Option<AttributeRating> {
        Character::make_attribute(&[self.attune, self.command, self.consort, self.sway])
    }

    fn make_attribute(ratings: &[i32]) -> Option<AttributeRating> {
        Some(AttributeRating {
            rating: ratings.iter().filter(|r| **r > 0).count(),
        })
    }

    // Actions

    pub fn action(&self, name: ActionName) -> Option<ActionRating> {
        match name {
            ActionName::Attune => self.attune(),
            ActionName::Command => self.command(),
            ActionName::Consort => self.consort(),
            ActionName::Doctor => self.doctor(),
            ActionName::Hack => self.hack(),
            ActionName::Helm => self.helm(),
            ActionName::Rig => self.rig(),
            ActionName::Scramble => self.scramble(),
            ActionName::Scrap => self.scrap(),
            ActionName::Skulk => self.skulk(),
            ActionName::Study => self.study(),
            ActionName::Sway => self.sway(),
        }
    }

    pub fn attune(&self) -> Option<ActionRating> {
        self.make_action(self.attune)
    }

    pub fn command(&self) -> Option<ActionRating> {
        self.make_action(self.command)
    }

    pub fn consort(&self) -> Option<ActionRating> {
        self.make_action(self.consort)
    }

    pub fn doctor(&self) -> Option<ActionRating> {
        self.make_action(self.doctor)
    }

    pub fn hack(&self) -> Option<ActionRating> {
        self.make_action(self.hack)
    }

    pub fn helm(&self) -> Option<ActionRating> {
        self.make_action(self.helm)
    }

    pub fn rig(&self) -> Option<ActionRating> {
        self.make_action(self.rig)
    }

    pub fn scramble(&self) -> Option<ActionRating> {
        self.make_action(self.scramble)
    }

    pub fn scrap(&self) -> Option<ActionRating> {
        self.make_action(self.scrap)
    }

    pub fn skulk(&self) -> Option<ActionRating> {
        self.make_action(self.skulk)
    }

    pub fn study(&self) -> Option<ActionRating> {
        self.make_action(self.study)
    }

    pub fn sway(&self) -> Option<ActionRating> {
        self.make_action(self.sway)
    }

    fn make_action(&self, rating: i32) -> Option<ActionRating> {
        Some(ActionRating {
            rating: rating.try_into().unwrap_or(0),
        })
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct AttributeRating {
    pub rating: usize,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ActionRating {
    pub rating: usize,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum AttributeName {
    Insight,
    Prowess,
    Resolve,
}

impl AttributeName {
    pub fn parse(string: &str) -> Option<AttributeName> {
        match string.to_lowercase().as_ref() {
            "insight" => Some(AttributeName::Insight),
            "prowess" => Some(AttributeName::Prowess),
            "resolve" => Some(AttributeName::Resolve),
            _ => None,
        }
    }

    pub fn as_str(&self) -> &str {
        match self {
            AttributeName::Insight => "Insight",
            AttributeName::Prowess => "Prowess",
            AttributeName::Resolve => "Resolve",
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ActionName {
    Attune,
    Command,
    Consort,
    Doctor,
    Hack,
    Helm,
    Rig,
    Scramble,
    Scrap,
    Skulk,
    Study,
    Sway,
}

impl ActionName {
    pub fn parse(string: &str) -> Option<ActionName> {
        match string.to_lowercase().as_ref() {
            "attune" => Some(ActionName::Attune),
            "command" => Some(ActionName::Command),
            "consort" => Some(ActionName::Consort),
            "doctor" => Some(ActionName::Doctor),
            "hack" => Some(ActionName::Hack),
            "helm" => Some(ActionName::Helm),
            "rig" => Some(ActionName::Rig),
            "scramble" => Some(ActionName::Scramble),
            "scrap" => Some(ActionName::Scrap),
            "skulk" => Some(ActionName::Skulk),
            "study" => Some(ActionName::Study),
            "sway" => Some(ActionName::Sway),
            _ => None,
        }
    }

    pub fn as_str(&self) -> &str {
        match self {
            ActionName::Attune => "Attune",
            ActionName::Command => "Command",
            ActionName::Consort => "Consort",
            ActionName::Doctor => "Doctor",
            ActionName::Hack => "Hack",
            ActionName::Helm => "Helm",
            ActionName::Rig => "Rig",
            ActionName::Scramble => "Scramble",
            ActionName::Scrap => "Scrap",
            ActionName::Skulk => "Skulk",
            ActionName::Study => "Study",
            ActionName::Sway => "Sway",
        }
    }
}
