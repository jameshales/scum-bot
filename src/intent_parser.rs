use crate::character::{AttributeName, ActionName};
use crate::character_roll::{CharacterRoll, Check};
use crate::command::{Command, Error};
use crate::roll::Roll;
use snips_nlu_ontology::{IntentParserResult, Slot, SlotValue};
use std::convert::TryFrom;

pub fn parse_intent_result(result: &IntentParserResult) -> Result<Command, Error> {
    let IntentParserResult { intent, slots, .. } = result;
    intent
        .intent_name
        .as_ref()
        .ok_or(Error::NoIntent)
        .and_then(|intent_name| match intent_name.as_ref() {
            "rollAction" => parse_roll_action(&slots),
            "rollDice" => parse_roll_dice(&slots),
            "rollResistance" => parse_roll_resistance(&slots),
            "showHelp" => Ok(Command::Help),
            intent_name => Err(Error::UnknownIntent(intent_name.to_owned())),
        })
}

fn parse_roll_dice(slots: &[Slot]) -> Result<Command, Error> {
    let rolls = extract_usize_slot_value(slots, "rolls").unwrap_or(1);
    Roll::new(rolls)
        .map(Command::Roll)
        .map_err(|error| Error::RollDiceInvalid(error, rolls))
}

fn parse_roll_resistance(slots: &[Slot]) -> Result<Command, Error> {
    let attribute = extract_attribute_slot(slots);
    attribute
        .ok_or(Error::RollResistanceMissingAttribute)
        .map(|attribute| {
            let roll = CharacterRoll {
                check: Check::Attribute(attribute),
            };
            Command::CharacterRoll(roll)
        })
}

fn parse_roll_action(slots: &[Slot]) -> Result<Command, Error> {
    let action = extract_action_slot(slots);
    action.ok_or(Error::RollActionMissingAction).map(|action| {
        let roll = CharacterRoll {
            check: Check::Action(action),
        };
        Command::CharacterRoll(roll)
    })
}

fn extract_attribute_slot(slots: &[Slot]) -> Option<AttributeName> {
    extract_custom_slot_value(slots, "attribute").and_then(|value| AttributeName::parse(value.as_ref()))
}

fn extract_custom_slot_value<'a>(slots: &'a [Slot], slot_name: &str) -> Option<&'a String> {
    find_slot_by_name(slots, slot_name).and_then(|slot| match &slot.value {
        SlotValue::Custom(string_value) => Some(&string_value.value),
        _ => None,
    })
}

fn extract_usize_slot_value<'a>(slots: &'a [Slot], slot_name: &str) -> Option<usize> {
    extract_f64_slot_value(slots, slot_name).and_then(|v| usize::try_from(v as i64).ok())
}

fn extract_f64_slot_value<'a>(slots: &'a [Slot], slot_name: &str) -> Option<f64> {
    slots
        .iter()
        .find(|slot| slot.slot_name == slot_name)
        .and_then(|slot| match &slot.value {
            SlotValue::Number(number_value) => Some(number_value.value),
            _ => None,
        })
}

fn extract_action_slot(slots: &[Slot]) -> Option<ActionName> {
    extract_custom_slot_value(slots, "action").and_then(|value| ActionName::parse(value.as_ref()))
}

fn find_slot_by_name<'a>(slots: &'a [Slot], slot_name: &str) -> Option<&'a Slot> {
    slots.iter().find(|slot| slot.slot_name == slot_name)
}
