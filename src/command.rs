use crate::character_roll::CharacterRoll;
use crate::error;
use crate::intent_parser::parse_intent_result;
use crate::response::Response;
use crate::roll;
use crate::roll::Roll;
use crate::roll::Error as RollError;
use regex::Regex;
use snips_nlu_lib::SnipsNluEngine;
use snips_nlu_ontology::IntentParserResult;
use std::fmt;
use symspell::{SymSpell, UnicodeStringStrategy};

#[derive(Debug)]
pub enum Command {
    CharacterRoll(crate::character_roll::CharacterRoll),
    Help,
    Roll(crate::roll::Roll),
}

impl Command {
    pub fn description(&self) -> &str {
        match self {
            Command::CharacterRoll(_) => "perform a character roll",
            Command::Help => "ask for help",
            Command::Roll(_) => "perform a roll",
        }
    }
}

#[derive(Debug)]
pub enum Error {
    // Shorthand commands
    CharacterRollParserError,
    RollParserError(roll::ParserError),

    // Natural language commands
    IntentParserError(::failure::Error),
    NoIntent,
    RollDiceInvalid(RollError, usize),
    RollResistanceMissingAttribute,
    RollActionMissingAction,
    UnknownIntent(String),
}

impl Error {
    pub fn into_response(self) -> Response {
        match self {
            Error::IntentParserError(error) => {
                Response::Error(error::Error::IntentParserError(error))
            }
            Error::UnknownIntent(intent_name) => {
                Response::Error(error::Error::UnknownIntent(intent_name))
            }
            error => Response::Clarification(error.to_string()),
        }
    }
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Error::CharacterRollParserError => {
                write!(f, "It looks like you're trying to roll an action or resistance roll, but the syntax is invalid. Try typing `!help` for some examples.")
            }
            Error::RollParserError(error) => {
                write!(f, "It looks like you're trying to some dice, but the syntax is invalid. {} Try typing `!help` for some examples.", error)
            }
            Error::RollDiceInvalid(error, rolls) => match error {
                RollError::RollsTooGreat => {
                    write!(f, "It looks like you're trying to roll {} dice. That's too many dice! Try rolling 100 or fewer dice.", rolls)
                },
            }
            Error::RollResistanceMissingAttribute => {
                write!(f, "It looks like you're trying to roll a resistance roll, but I'm not sure what kind of resistance roll you want. Try \"Roll insight resistance roll\", \"Resolve resistance roll\", etc.")
            }
            Error::RollActionMissingAction => {
                write!(f, "It looks like you're trying to roll an action check, but I'm not sure what action you want. Try \"Roll command\", \"Hacking roll\", etc.")
            }
            Error::NoIntent => {
                write!(f, "I'm not sure what you mean. Try asking again with a different or simpler phrasing. Try asking for help to see some examples.")
            }
            Error::UnknownIntent(intent_name) => {
                write!(f, "An unknown intent name was returned by the NLP engine: {}", intent_name)
            },
            Error::IntentParserError(error) => {
                write!(f, "An unknown error was returned by the NLP engine: {}", error)
            }
        }
    }
}

type NaturalLanguageCommandResult =
    Option<Result<(Result<Command, Error>, IntentParserResult, Option<String>), Error>>;

impl Command {
    pub fn is_private(&self) -> bool {
        match self {
            Command::Help | Command::Roll(_) => true,
            _ => false,
        }
    }

    pub fn parse(
        engine: &SnipsNluEngine,
        symspell: &SymSpell<UnicodeStringStrategy>,
        content: &str,
        bot_id: Option<&str>,
        dice_only: bool,
    ) -> Option<Result<CommandResult, Error>> {
        Command::parse_shorthand(content)
            .map(CommandResult::Shorthand)
            .map(Ok)
            .or({
                Command::parse_natural_language(engine, symspell, content, bot_id, dice_only).map(
                    |result| {
                        result.map(|(command, intent_result, corrected)| {
                            CommandResult::NaturalLanguage(command, intent_result, corrected)
                        })
                    },
                )
            })
    }

    fn parse_natural_language(
        engine: &SnipsNluEngine,
        symspell: &SymSpell<UnicodeStringStrategy>,
        message: &str,
        bot_id: Option<&str>,
        dice_only: bool,
    ) -> NaturalLanguageCommandResult {
        Command::extract_at_message(message, bot_id, dice_only)
            .as_ref()
            .map(|at_message| {
                let corrected = Command::spelling_correction(symspell, at_message);
                let used = corrected.as_ref().unwrap_or(at_message).as_str();
                engine
                    .parse(used, None, None)
                    .map(|result| (parse_intent_result(&result), result, corrected))
                    .map_err(Error::IntentParserError)
            })
    }

    fn extract_at_message(message: &str, bot_id: Option<&str>, dice_only: bool) -> Option<String> {
        lazy_static! {
            static ref COMMAND_REGEX: Regex = Regex::new(r"^(?:<@!?(\d+)> *)?(.*)$").unwrap();
        }

        COMMAND_REGEX.captures(&message).and_then(|c| {
            let is_at_message = c
                .get(1)
                .map_or(false, |m| bot_id.iter().any(|bot_id| bot_id == &m.as_str()));
            if dice_only || is_at_message {
                c.get(2).map(|m| m.as_str().to_owned())
            } else {
                None
            }
        })
    }

    fn spelling_correction(
        symspell: &SymSpell<UnicodeStringStrategy>,
        message: &str,
    ) -> Option<String> {
        let trimmed = message.trim();
        let suggestions = symspell.lookup_compound(trimmed, 2);
        suggestions.into_iter().next().map(|s| s.term)
    }

    fn parse_shorthand(command: &str) -> Option<Result<Command, Error>> {
        lazy_static! {
            static ref ROLL_COMMAND_REGEX: Regex = Regex::new(r"^!(?:r|roll) +(.*)$").unwrap();
        }

        if command == "!help" {
            Some(Ok(Command::Help))
        } else if let Some(captures) = ROLL_COMMAND_REGEX.captures(&command) {
            let roll_command = captures.get(1).map_or("", |m| m.as_str()).to_owned();
            Some(
                Roll::parse(&roll_command)
                    .map(Command::Roll)
                    .map_err(Error::RollParserError)
                    .or_else(|_| {
                        CharacterRoll::parse(&roll_command)
                            .map(Command::CharacterRoll)
                            .ok_or(Error::CharacterRollParserError)
                    }),
            )
        } else {
            None
        }
    }
}

pub enum CommandResult {
    Shorthand(Result<Command, Error>),
    NaturalLanguage(Result<Command, Error>, IntentParserResult, Option<String>),
}
