use crate::channel::Channel;
use crate::character::Character;
use crate::character_roll::CharacterRoll;
use crate::command;
use crate::command::{Command, CommandResult};
use crate::error::Error;
use crate::intent_logger::log_intent_result;
use crate::response::Response;
use crate::roll::Roll;
use log::{error, info};
use r2d2::Pool;
use r2d2_sqlite::SqliteConnectionManager;
use snips_nlu_lib::SnipsNluEngine;
use snips_nlu_ontology::IntentParserResult;
use std::convert::identity;
use std::sync::RwLock;
use symspell::{SymSpell, UnicodeStringStrategy};

use serenity::{
    model::{
        channel::Message,
        gateway::Ready,
        id::{ChannelId, UserId},
    },
    prelude::*,
};

const CHARACTER_NOT_FOUND_WARNING_TEXT: &str =
    "Couldn't find any entry for character.";

// TODO Remove this, attributes are not optional
const ATTRIBUTE_NOT_SET_WARNING_TEXT: &str =
    "Couldn't find required attribute ratings for character.";

enum Action {
    IgnoreChannelDisabled,
    IgnoreCommandMissing,
    IgnoreOwnMessage,
    Respond(Response),
}

pub struct Handler {
    pub bot_id: RwLock<Option<String>>,
    pub engine: SnipsNluEngine,
    pub pool: Pool<SqliteConnectionManager>,
    pub symspell: SymSpell<UnicodeStringStrategy>,
}

impl Handler {
    fn get_command(
        &self,
        engine: &SnipsNluEngine,
        symspell: &SymSpell<UnicodeStringStrategy>,
        message: &Message,
        dice_only: bool,
    ) -> Option<Result<CommandResult, command::Error>> {
        let content = &message.content.trim();
        self.bot_id
            .try_read()
            .ok()
            .and_then(|bot_id| {
                bot_id.as_ref().map(|bot_id| {
                    Command::parse(engine, symspell, content, Some(&bot_id), dice_only)
                })
            })
            .unwrap_or_else(|| Command::parse(engine, symspell, content, None, dice_only))
    }

    fn get_action(
        &self,
        command_result: Option<Result<CommandResult, command::Error>>,
        channel: &Channel,
        message: &Message,
        is_admin: bool,
        is_private: bool,
    ) -> Action {
        command_result.map_or(Action::IgnoreCommandMissing, |command_result| {
            command_result
                .map(|command_result| {
                    let command = match command_result {
                        CommandResult::Shorthand(command) => command,
                        CommandResult::NaturalLanguage(command, intent_result, corrected) => {
                            self.log_intent_result(&message, &intent_result, corrected.as_deref());
                            command
                        }
                    };
                    match command {
                        Ok(command) => {
                            if !is_admin && !channel.enabled {
                                Action::IgnoreChannelDisabled
                            } else if is_private && !command.is_private() {
                                Action::Respond(Response::Warning(format!("It looks like you're trying to {}. You can't do that in a private message.", command.description())))
                            } else {
                                Action::Respond(self.run_command(
                                    command,
                                    message.channel_id,
                                    message.author.id,
                                ))
                            }
                        }
                        Err(error) => Action::Respond(error.into_response()),
                    }
                })
                .unwrap_or_else(|error| Action::Respond(error.into_response()))
        })
    }

    fn run_command(&self, command: Command, channel_id: ChannelId, author_id: UserId) -> Response {
        match command {
            Command::CharacterRoll(roll) => self.character_roll(&roll, channel_id, author_id),
            Command::Help => Handler::help(),
            Command::Roll(roll) => Handler::roll(roll),
        }
    }

    fn log_intent_result(
        &self,
        message: &Message,
        intent_result: &IntentParserResult,
        corrected: Option<&str>,
    ) {
        self.pool
            .get()
            .map_err(|error| error!(target: "scum-bot", "Error obtaining database connection. Message ID: {}; Error: {}", message.id, error))
            .and_then(|mut connection| {
                log_intent_result(&mut connection, message, intent_result, corrected)
                    .map_err(|error|
                        error!(target: "scum-bot", "Error logging intent result. Message ID: {}; Error: {}", message.id, error)
                    )
            })
            .unwrap_or(())
    }

    fn character_roll(
        &self,
        character_roll: &CharacterRoll,
        channel_id: ChannelId,
        author_id: UserId,
    ) -> Response {
        self.pool
            .get()
            .map_err(|error| Response::Error(Error::R2D2Error(error)))
            .and_then(|connection| {
                Character::get(&connection, channel_id, author_id)
                    .map_err(|error| Response::Error(Error::RusqliteError(error)))
            })
            .and_then(|character| {
                character
                    .ok_or_else(|| Response::Warning(CHARACTER_NOT_FOUND_WARNING_TEXT.to_owned()))
            })
            .and_then(|character| {
                character_roll
                    .to_roll(&character)
                    .ok_or_else(|| Response::Warning(ATTRIBUTE_NOT_SET_WARNING_TEXT.to_owned()))
            })
            .map(|roll| {
                let mut rng = rand::thread_rng();
                let result = roll.roll(&mut rng);
                Response::DiceRoll(format!(
                    "rolled {} ({}) = {}",
                    character_roll.check, roll, result
                ))
            })
            .unwrap_or_else(identity)
    }

    fn help() -> Response {
        Response::Help(
            "Try typing the following:\n\
             • \"Roll three dice\"\n\
             • \"Do a hacking roll\"\n\
             • \"Perform an insight resistance roll\""
                .to_owned(),
        )
    }

    fn roll(roll: Roll) -> Response {
        let mut rng = rand::thread_rng();
        let result = roll.roll(&mut rng);
        Response::DiceRoll(format!("rolled {} = {}", roll, result))
    }

    fn get_channel(&self, channel_id: ChannelId) -> Channel {
        self.pool
            .get()
            .ok()
            .and_then(|connection|
                Channel::get(&connection, channel_id)
                    .map_err(|error| error!(target: "scum-bot", "Error retrieving channel: Channel ID: {}; Error: {}", channel_id.to_string(), error))
                    .ok()
                    .and_then(identity)
            )
            .unwrap_or(
                Channel {
                    enabled: false,
                    locked: false,
                    dice_only: false,
                }
            )
    }
}

impl EventHandler for Handler {
    fn message(&self, ctx: Context, message: Message) {
        info!(target: "scum-bot", "Received message. Message ID: {}; Content: {}", message.id, message.content.escape_debug());
        let action = if message.is_own(&ctx.cache) {
            // Don't respond to our own messages, this may cause an infinite loop
            Action::IgnoreOwnMessage
        } else {
            let channel = self.get_channel(message.channel_id);
            let is_admin = message.member(&ctx.cache).map_or(true, |member| {
                member
                    .permissions(&ctx.cache)
                    .ok()
                    .map_or(false, |permissions| permissions.administrator())
            });
            let is_private = message.is_private();
            let command_result = self.get_command(
                &self.engine,
                &self.symspell,
                &message,
                // Private channels are implicitly dice only, no need to @me
                channel.dice_only || is_private,
            );
            if let Some(command_result) = command_result.as_ref() {
                match command_result {
                    Ok(CommandResult::NaturalLanguage(Ok(command), _, corrected)) => {
                        info!(target: "scum-bot", "Parsed natural language command successfully. Message ID: {}; Command: {:?}; Corrected Message: {}", message.id, command, corrected.as_deref().unwrap_or(""))
                    }
                    Ok(CommandResult::NaturalLanguage(Err(error), _, corrected)) => {
                        info!(target: "scum-bot", "Error parsing natural language command. Message ID: {}; Corrected Message: {}; Error: {:}", message.id, corrected.as_deref().unwrap_or(""), error)
                    }
                    Ok(CommandResult::Shorthand(Err(error))) => {
                        info!(target: "scum-bot", "Error parsing shorthand command. Message ID: {}; Command: {:?}", message.id, error)
                    }
                    Ok(CommandResult::Shorthand(Ok(command))) => {
                        info!(target: "scum-bot", "Parsed shorthand command successfully. Message ID: {}; Command: {:?}", message.id, command)
                    }
                    Err(error) => {
                        info!(target: "scum-bot", "Error parsing command. Message ID: {}; Error: {}", message.id, error)
                    }
                }
            };
            self.get_action(command_result, &channel, &message, is_admin, is_private)
        };
        match action {
            Action::IgnoreChannelDisabled => {
                info!(target: "scum-bot", "Ignoring command because Scum Bot is disabled in current channel. Message ID: {}", message.id);
            }
            Action::IgnoreCommandMissing => {
                info!(target: "scum-bot", "Ignoring message because it contains no command. Message ID: {}", message.id);
            }
            Action::IgnoreOwnMessage => {
                info!(target: "scum-bot", "Ignoring message because it was sent by us. Message ID: {}", message.id);
            }
            Action::Respond(response) => {
                if let Response::Error(error) = &response {
                    error!(target: "scum-bot", "Error processing command. Message ID: {}; Error = {:?}", message.id, error);
                };
                let result = message
                    .channel_id
                    .say(&ctx.http, response.render(message.author.id, message.id));
                match result {
                    Ok(sent_message) => {
                        info!(target: "scum-bot", "Sent message. Message ID: {}; Sent Message ID: {}; Content: {}", message.id, sent_message.id, sent_message.content.escape_debug())
                    }
                    Err(error) => {
                        error!(target: "scum-bot", "Error sending message. Message ID: {}; Error: {:?}", message.id, error)
                    }
                }
            }
        };
    }

    fn ready(&self, _: Context, ready: Ready) {
        let mut bot_id = self
            .bot_id
            .write()
            .expect("RwLock for bot_id has been poisoned");
        *bot_id = Some(ready.user.id.to_string());
        info!(target: "scum-bot", "{} is connected!", ready.user.name);
    }
}
