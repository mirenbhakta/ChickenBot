#![feature(once_cell)]
#![feature(async_closure)]

pub mod commands;
pub mod settings;

use std::env;
use crate::commands::command::Command;
use crate::commands::invite::InviteCommand;
use crate::commands::help::HelpCommand;

use serenity::{
    async_trait,
    model::{
        gateway::Ready,
        id::GuildId,
        interactions::{
            application_command::{
                ApplicationCommand,
            },
            Interaction,
            InteractionResponseType,
        },
    },
    prelude::*,
};
use serenity::builder::{CreateApplicationCommand, CreateApplicationCommands};
use crate::commands::delete_commands::DeleteCommandsCommand;
use crate::settings::Settings;

const VERSION: &str = env!("CARGO_PKG_VERSION");

struct ChickenBot {
    commands: Vec<Box<dyn Command>>
}

impl ChickenBot {
    pub async fn new() -> ChickenBot {
        ChickenBot {
            commands: ChickenBot::load_commands().await
        }
    }

    pub async fn load_commands() -> Vec<Box<dyn Command>> {
        let mut commands: Vec<Box<dyn Command>> = vec![
            Box::new(InviteCommand::new().await),
            Box::new(HelpCommand::new().await),
        ];

        if env::var("DEV").is_ok() {
            commands.push(Box::new(DeleteCommandsCommand::new().await));
        }

        commands
    }
}

#[async_trait]
impl EventHandler for ChickenBot {
    async fn ready(&self, ctx: Context, ready: Ready) {
        println!("{} v{} is connected!", ready.user.name, VERSION);

        self.register_commands(&ctx).await
    }

    async fn interaction_create(&self, ctx: Context, interaction: Interaction) {
        self.interaction_created(ctx, interaction).await
    }
}

#[tokio::main]
async fn main() {

    Settings::load();

    // Configure the client with your Discord bot token in the environment.
    let token = env::var("DISCORD_TOKEN").expect("Expected a token in the environment");

    // The Application Id is usually the Bot User Id.
    let application_id: u64 = env::var("APPLICATION_ID")
        .expect("Expected an application id in the environment")
        .parse()
        .expect("application id is not a valid id");

    // Build our client.
    let mut client = Client::builder(token)
        .event_handler(ChickenBot::new().await)
        .application_id(application_id)
        .await
        .expect("Error creating client");

    if let Err(why) = client.start().await {
        println!("Client error: {:?}", why);
    }
}
