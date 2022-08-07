use crate::http::ICLIENT;
use regex::Regex;
use serde::{Deserialize, Serialize};
use std::num::NonZeroU64;
use twilight_model::application::command::*;
use twilight_validate::command::CommandValidationError;

lazy_static! {
    pub static ref RE_NAME_PATTERN: Regex =
        Regex::new(r"^[-_\p{L}\p{N}\p{sc=Deva}\p{sc=Thai}]{1,32}$").unwrap();
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]

pub struct Tag {
    pub guild_id: String,
    pub author_id: String,
    pub command_id: Option<String>,

    pub name: String,
    pub description: Option<String>,

    pub tagscript: String,
    #[serde(default)]
    pub options: Vec<TagOption>,
    #[serde(default)]
    pub uses: u32,
}

impl Tag {
    pub async fn create(&mut self) -> Result<Command, CommandValidationError> {
        let command = ICLIENT
            .create_guild_command(
                NonZeroU64::new(self.guild_id.parse().unwrap())
                    .unwrap()
                    .into(),
            )
            .chat_input(&self.name, self.description.as_ref().unwrap())?
            .command_options(
                self.options
                    .iter()
                    .map::<CommandOption, _>(|x| x.into())
                    .collect::<Vec<_>>()
                    .as_slice(),
            )?
            .exec()
            .await
            .map_err(|e| {
                eprintln!("{:?}", e);
                CommandValidationError::COMMAND_COUNT_INVALID
            })?
            .model()
            .await
            .unwrap();

        self.command_id = Some(command.id.unwrap().to_string());
        Ok(command)
    }

    pub async fn delete(&self) -> Result<(), CommandValidationError> {
        ICLIENT
            .delete_guild_command(
                NonZeroU64::new(self.guild_id.parse().unwrap())
                    .unwrap()
                    .into(),
                NonZeroU64::new(self.command_id.as_ref().unwrap().parse().unwrap())
                    .unwrap()
                    .into(),
            )
            .exec()
            .await
            .map_err(|e| {
                eprintln!("{:?}", e);
                CommandValidationError::COMMAND_COUNT_INVALID
            })?;
        Ok(())
    }

    pub async fn update(&self, new: &Tag) -> Result<(), CommandValidationError> {
        let mut update = ICLIENT.update_guild_command(
            NonZeroU64::new(self.guild_id.parse().unwrap())
                .unwrap()
                .into(),
            NonZeroU64::new(self.command_id.as_ref().unwrap().parse().unwrap())
                .unwrap()
                .into(),
        );

        let mut updated = false;

        if new.name != self.name {
            updated = true;
            update = update.name(&new.name);
        }

        if new.description != self.description {
            updated = true;
            update = update.description(new.description.as_ref().unwrap());
        }

        let new_options = new
            .options
            .iter()
            .map::<CommandOption, _>(|x| x.into())
            .collect::<Vec<_>>();
        if new.options != self.options {
            updated = true;

            update = update.command_options(new_options.as_slice());
        }

        if updated {
            update.exec().await.map_err(|e| {
                eprintln!("{:?}", e);
                CommandValidationError::COMMAND_COUNT_INVALID
            })?;
        }
        Ok(())
    }
    pub fn validate(&self) -> bool {
        if self.guild_id.len() < 8
            || self.guild_id.parse::<u64>().is_err()
            || self.author_id.len() < 8
            || self.author_id.parse::<u64>().is_err()
            || self.command_id.is_some()
        {
            return false;
        }

        if !RE_NAME_PATTERN.is_match(&self.name) {
            return false;
        }

        if let Some(description) = &self.description {
            if description.len() > 100 || description.len() < 1 {
                return false;
            }
        }

        if self.tagscript.len() < 1 || self.tagscript.len() > 10_000 {
            return false;
        }

        if self.options.len() > 10 {
            return false;
        }

        if self.options.iter().any(|x| x.validate() == false) {
            return false;
        }
        true
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct TagOption {
    pub name: String,
    pub description: String,
    pub required: bool,
    #[serde(rename = "type")]
    pub ty: u8,
}

impl TagOption {
    pub fn validate(&self) -> bool {
        if !RE_NAME_PATTERN.is_match(&self.name) {
            return false;
        }

        if self.description.len() > 100 || self.description.len() < 1 {
            return false;
        }

        if self.ty < 3 || self.ty > 10 {
            return false;
        }

        true
    }
}
impl Into<CommandOption> for &TagOption {
    fn into(self) -> CommandOption {
        match self.ty {
            3 => CommandOption::String(ChoiceCommandOptionData {
                name: self.name.clone(),
                description: self.description.clone(),
                required: self.required,
                ..Default::default()
            }),
            4 => CommandOption::Integer(NumberCommandOptionData {
                name: self.name.clone(),
                description: self.description.clone(),
                required: self.required,
                ..Default::default()
            }),
            5 => CommandOption::Boolean(BaseCommandOptionData {
                name: self.name.clone(),
                description: self.description.clone(),
                required: self.required,
                ..Default::default()
            }),
            6 => CommandOption::User(BaseCommandOptionData {
                name: self.name.clone(),
                description: self.description.clone(),
                required: self.required,
                ..Default::default()
            }),
            7 => CommandOption::Channel(ChannelCommandOptionData {
                name: self.name.clone(),
                description: self.description.clone(),
                required: self.required,
                ..Default::default()
            }),
            8 => CommandOption::Role(BaseCommandOptionData {
                name: self.name.clone(),
                description: self.description.clone(),
                required: self.required,
                ..Default::default()
            }),
            9 => CommandOption::Mentionable(BaseCommandOptionData {
                name: self.name.clone(),
                description: self.description.clone(),
                required: self.required,
                ..Default::default()
            }),
            10 => CommandOption::Number(NumberCommandOptionData {
                name: self.name.clone(),
                description: self.description.clone(),
                required: self.required,
                ..Default::default()
            }),
            11 => CommandOption::Attachment(BaseCommandOptionData {
                name: self.name.clone(),
                description: self.description.clone(),
                required: self.required,
                ..Default::default()
            }),
            _ => CommandOption::String(ChoiceCommandOptionData {
                name: self.name.clone(),
                description: self.description.clone(),
                required: self.required,
                ..Default::default()
            }),
        }
    }
}
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct TagUpdate {
    pub new: Tag,
    pub old: Tag,
}
