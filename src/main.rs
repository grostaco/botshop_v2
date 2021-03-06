use std::env;

use serenity::{
    async_trait,
    client::{Context, EventHandler},
    model::{gateway::Ready, id::GuildId, interactions::Interaction},
    Client,
};

pub mod commands;
pub mod util;

use crate::commands::modify::Modify;
pub use crate::util::Records;
use commands::{info::Info, Daily, Pending, Transactions};
struct Handler;

#[async_trait]
impl EventHandler for Handler {
    async fn interaction_create(&self, ctx: Context, interaction: Interaction) {
        if let Interaction::ApplicationCommand(command) = interaction {
            match command.data.name.as_str() {
                "daily" => Daily::new("resources/users.db", command.user.id.0)
                    .handle_interaction(&ctx.http, command, &ctx.shard)
                    .await
                    .expect("Something went wrong with daily command!"),
                "pending" => Pending::new("resources/users.db", command.user.id.0)
                    .handle_interaction(&ctx.http, command, &ctx.shard)
                    .await
                    .expect("Something went wrong with pending command!"),
                "transactions" => Transactions::new("resources/users.db", command.user.id.0)
                    .await
                    .handle_interaction(&ctx.http, command, &ctx.shard)
                    .await
                    .expect("Something went wrong with the transactions command!"),
                "modify" => Modify::new("resources/users.db", command.user.id.0)
                    .handle_interaction(&ctx.http, command)
                    .await
                    .expect("Something went wrong with the modify command!"),
                "aboutme" => Info::new("resources/users.db", command.user.id.0)
                    .handle_interaction(&ctx.http, command, &ctx.shard)
                    .await
                    .expect("Something went wrong with the info command!"),
                "nya" => {
                    command
                        .create_interaction_response(&ctx.http, |response| {
                            response.interaction_response_data(|data| data.content("Nya!"))
                        })
                        .await
                        .expect("Unable to nya :(");
                }
                _ => panic!("Unknown command!"),
            }
        }
    }

    async fn ready(&self, ctx: Context, ready: Ready) {
        println!("{} is connected!", ready.user.name);

        let guild_command = GuildId::set_application_commands(
            &GuildId(
                env::var("GUILD_ID")
                    .expect("GUILD_ID not set")
                    .parse()
                    .expect("GUILD_ID must be an integer"),
            ),
            &ctx.http,
            |commands| {
                commands
                    .create_application_command(|command| {
                        command
                            .name("daily")
                            .description("Fetch your daily tasks :D")
                    })
                    .create_application_command(|command| {
                        command
                            .name("pending")
                            .description("Fetch your incomplete tasks! \\o/")
                    })
                    .create_application_command(|command| {
                        command
                            .name("transactions")
                            .description("Fetch your transactions history :>")
                    })
                    .create_application_command(|command| {
                        command
                            .name("aboutme")
                            .description("A personalized embed about how well you have been doing!")
                    })
                    .create_application_command(|command| command.name("nya").description("nya :D"))
                    .add_application_command(Modify::create_application_command())
            },
        )
        .await
        .expect("Unable to set command");

        println!(
            "The bot has registered the following guild slash commands {:#?}",
            guild_command,
        );
    }
}

#[tokio::main]
async fn main() {
    let token = env::var("DISCORD_TOKEN").expect("Expected a token in the environment");

    let application_id: u64 = env::var("APPLICATION_ID")
        .expect("Expected an application id in the environment")
        .parse()
        .expect("Application id is not a valid id");

    let mut client = Client::builder(token)
        .event_handler(Handler)
        .application_id(application_id)
        .await
        .expect("Error creating client");

    if let Err(why) = client.start().await {
        println!("Client error: {:?}", why);
    }
}
