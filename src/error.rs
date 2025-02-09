use std::fmt;
use chrono::ParseError;
use reqwest::StatusCode;
use serenity::client::Context;
use serenity::model::interactions::{InteractionResponseType};
use serenity::model::prelude::{Interaction, InteractionApplicationCommandCallbackDataFlags};
use serenity::prelude::SerenityError;
use crate::settings::SETTINGS;

#[derive(Debug)]
pub enum Error {
    InvalidParameterType,
    ErrorHttpCode(StatusCode, Option<String>),
    Reqwest(reqwest::Error),
    Serenity(SerenityError),
    CronoParseError(ParseError),
    Serde(serde_json::Error),
    Database(diesel::result::Error),
    Other(String)
}

impl From<reqwest::Error> for Error {
    fn from(err: reqwest::Error) -> Self {
        Error::Reqwest(err)
    }
}

impl From<SerenityError> for Error {
    fn from(err: SerenityError) -> Self {
        Error::Serenity(err)
    }
}

impl From<ParseError> for Error {
    fn from(err: ParseError) -> Self {
        Error::CronoParseError(err)
    }
}

impl From<serde_json::Error> for Error {
    fn from(err: serde_json::Error) -> Self {
        Error::Serde(err)
    }
}

impl From<diesel::result::Error> for Error {
    fn from(err: diesel::result::Error) -> Self {
        Error::Database(err)
    }
}

impl std::error::Error for Error {}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Error::InvalidParameterType => write!(f, "Discord sent parameter with invalid type"),
            Error::Reqwest(e) => e.fmt(f),
            Error::ErrorHttpCode(status, text) => write!(f, "A HTTP request returned a '{}'. Error: '{:?}'", status, text),
            Error::CronoParseError(e) => e.fmt(f),
            Error::Serenity(e) => e.fmt(f),
            Error::Serde(e) => e.fmt(f),
            Error::Database(db) => db.fmt(f),
            Error::Other(str) => str.fmt(f),
        }
    }
}

impl Error {
    pub async fn handle(&self, ctx: &Context, interaction: Option<&Interaction>, cmd: &str) {

        println!("Command '{}' threw an error '{:?}':\n", cmd, self);

        // First, notify user of error
        let mut notified_user = false;

        // Try to notify of error
        if let Some(val) = interaction {
            match val {
                Interaction::Ping(_) => return,
                Interaction::ApplicationCommand(command) => {
                    match command.create_interaction_response(&ctx.http, |t| {
                        t.kind(InteractionResponseType::ChannelMessageWithSource)
                            .interaction_response_data(|message|
                                message.content("There was an error processing your request, it has been sent to the bot maintenance team.")
                                    .flags(InteractionApplicationCommandCallbackDataFlags::EPHEMERAL))
                    }).await {
                        Ok(_) => notified_user = true,
                        Err(e) => println!("Failed to notify user via interaction response! {}", e)
                    }
                },
                Interaction::MessageComponent(message) => {
                    match message.create_interaction_response(&ctx.http, |t| {
                        t.kind(InteractionResponseType::ChannelMessageWithSource)
                            .interaction_response_data(|message|
                                message.content("There was an error processing your request, it has been sent to the bot maintenance team.")
                                    .flags(InteractionApplicationCommandCallbackDataFlags::EPHEMERAL))
                    }).await {
                        Ok(_) => notified_user = true,
                        Err(e) => println!("Failed to notify user via interaction response! {}", e)
                    }
                }
                Interaction::Autocomplete(_) => {}
            };

            if !notified_user {
                match val {
                    Interaction::Ping(_) => return,
                    Interaction::ApplicationCommand(command) => {
                        // If the first method fails, try to DM them
                        match command.user.direct_message(&ctx.http, |test| {
                            test.content("There was an error processing your request, it has been sent to the bot maintenance team.")
                        }).await {
                            Ok(_) => notified_user = true,
                            Err(e) => println!("Failed to notify user via DM! {}", e)
                        }
                    }
                    Interaction::MessageComponent(message) => {
                        // If the first method fails, try to DM them
                        match message.user.direct_message(&ctx.http, |test| {
                            test.content("There was an error processing your request, it has been sent to the bot maintenance team.")
                        }).await {
                            Ok(_) => notified_user = true,
                            Err(e) => println!("Failed to notify user via DM! {}", e)
                        }
                    }
                    Interaction::Autocomplete(_) => {}
                }
            }
        }

        match ctx.http.get_user(130173614702985216).await {
            Ok(user) => {
                match user.direct_message(&ctx.http, |message| {
                    message.content(format!("Command '{}' threw an error ```{:?}```\nNotified User: {}", cmd, self, notified_user))
                }).await {
                    Ok(_) => {}
                    Err(e) => println!("Error: Could not message user '{}' to send error message to. {}", SETTINGS.get().as_ref().unwrap().user_manager, e)
                }
            }
            Err(e) => println!("Error: Could not look up user '{}' to send error message to {}", SETTINGS.get().as_ref().unwrap().user_manager, e)
        }

        if !notified_user {
            println!("Failed to notify user")
        }
    }
}