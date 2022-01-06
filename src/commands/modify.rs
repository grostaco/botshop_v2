use std::{collections::HashMap, sync::Arc};

use crate::util::db::{update_user, User};
use chrono::Utc;
use serenity::{
    builder::CreateApplicationCommand,
    client::bridge::gateway::ShardMessenger,
    http::Http,
    model::interactions::application_command::{
        ApplicationCommandInteraction,
        ApplicationCommandInteractionDataOptionValue as ApplicationValue,
        ApplicationCommandOptionType,
    },
};

macro_rules! cast {
    ($target: expr, $pat: path) => {{
        if let $pat(a) = $target {
            a
        } else {
            panic!("mismatch variant when cast to {}", stringify!($pat))
        }
    }};
}

pub struct Modify<'a> {
    db_path: &'a str,
    user_id: u64,
}

impl<'a> Modify<'a> {
    pub fn new(db_path: &'a str, user_id: u64) -> Self {
        Self { db_path, user_id }
    }

    pub fn create_application_command() -> CreateApplicationCommand {
        let mut command = CreateApplicationCommand::default();
        command
            .name("modify")
            .description("modify your data!")
            .create_option(|option| {
                option
                    .name("insert")
                    .create_sub_option(|option| {
                        option
                            .name("record_type")
                            .description("The record type you'd like to insert to")
                            .add_string_choice("Daily", "daily")
                            .add_string_choice("Pending", "pending")
                            .add_string_choice("Transaction", "transaction")
                            .required(true)
                            .kind(ApplicationCommandOptionType::String)
                    })
                    .create_sub_option(|option| {
                        option
                            .name("name")
                            .description("The name of the task")
                            .required(true)
                            .kind(ApplicationCommandOptionType::String)
                    })
                    .create_sub_option(|option| {
                        option
                            .name("points")
                            .description("Points to be awarded for completing this task")
                            .required(true)
                            .kind(ApplicationCommandOptionType::Integer)
                    })
                    .create_sub_option(|option| {
                        option
                            .name("timestamp")
                            .description("The timestamp for when the task was completed")
                            .kind(ApplicationCommandOptionType::Integer)
                    })
                    .description("insert into a task into a record type!")
                    .kind(ApplicationCommandOptionType::SubCommand)
            })
            .create_option(|option| {
                option
                    .name("delete")
                    .description("Remove a task")
                    .kind(ApplicationCommandOptionType::SubCommand)
            });

        command
    }

    pub async fn handle_interaction(
        &mut self,
        http: &Arc<Http>,
        interaction: ApplicationCommandInteraction,
        shard: &ShardMessenger,
    ) -> Result<(), serenity::Error> {
        let mut user = User::from_file(self.db_path, self.user_id).unwrap();

        let option = interaction.data.options.first().unwrap();
        let options: HashMap<&str, _> = option
            .options
            .iter()
            .map(|option| (option.name.as_str(), option.resolved.as_ref().unwrap()))
            .collect();

        let name = cast!(options.get("name").unwrap(), ApplicationValue::String);
        let points = cast!(options.get("points").unwrap(), ApplicationValue::Integer);
        let record_type = cast!(
            options.get("record_type").unwrap(),
            ApplicationValue::String
        );
        let timestamp = options.get("timestamp").map_or(None, |value| {
            if let ApplicationValue::Integer(value) = value {
                Some(*value)
            } else {
                panic!("Timestamp has an unexpected type variant!");
            }
        });

        let record = match record_type.as_str() {
            "daily" => &mut user.daily,
            "pending" => &mut user.pending,
            "transaction" => &mut user.transactions,
            _ => panic!("Unknown record type!"),
        };

        match option.name.as_str() {
            "insert" => record.push(
                name.to_owned(),
                *points,
                timestamp.map_or_else(
                    || {
                        if record_type == "transaction" {
                            Some(Utc::now().timestamp())
                        } else {
                            None
                        }
                    },
                    |ts| Some(ts),
                ),
            ),
            "update" => {}
            "delete" => {}
            _ => panic!("Cannot handle modify interaction"),
        }
        update_user(self.db_path, &user).unwrap();

        interaction
            .create_interaction_response(http, |response| {
                response
                    .interaction_response_data(|data| data.content("Your record has been altered!"))
            })
            .await?;

        Ok(())
    }
}
