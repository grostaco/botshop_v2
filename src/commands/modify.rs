use std::{collections::HashMap, sync::Arc};

use crate::util::db::{update_user, User};
use chrono::Utc;
use serenity::{
    builder::CreateApplicationCommand,
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

macro_rules! cast_opt {
    ($target: expr, $pat: path) => {
        $target.map_or(None, |value| Some(cast!(value, $pat)))
    };
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
                    .name("update")
                    .create_sub_option(|option| {
                        option
                            .name("index")
                            .description("The record's index")
                            .required(true)
                            .kind(ApplicationCommandOptionType::Integer)
                    })
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
                    .description("update an existing task of a record type!")
                    .kind(ApplicationCommandOptionType::SubCommand)
            })
            .create_option(|option| {
                option
                    .name("delete")
                    .description("remove a task!")
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
                            .name("index")
                            .description("The record's index")
                            .required(true)
                            .kind(ApplicationCommandOptionType::Integer)
                    })
                    .kind(ApplicationCommandOptionType::SubCommand)
            });

        command
    }

    pub async fn handle_interaction(
        &mut self,
        http: &Arc<Http>,
        interaction: ApplicationCommandInteraction,
    ) -> Result<(), serenity::Error> {
        let mut user = User::from_file(self.db_path, self.user_id).unwrap();

        let option = interaction.data.options.first().unwrap();
        let options: HashMap<&str, _> = option
            .options
            .iter()
            .map(|option| (option.name.as_str(), option.resolved.as_ref().unwrap()))
            .collect();

        let name = cast_opt!(options.get("name"), ApplicationValue::String);
        let points = cast_opt!(options.get("points"), ApplicationValue::Integer);
        let record_type = cast_opt!(options.get("record_type"), ApplicationValue::String);
        let timestamp = cast_opt!(options.get("timestamp"), ApplicationValue::Integer);
        let index = cast_opt!(options.get("index"), ApplicationValue::Integer);

        let record = record_type.map_or(None, |record_type| {
            Some(match record_type.as_str() {
                "daily" => &mut user.daily,
                "pending" => &mut user.pending,
                "transaction" => &mut user.transactions,
                _ => panic!("Unknown record type!"),
            })
        });

        match option.name.as_str() {
            "insert" => record.unwrap().push(
                name.unwrap().to_owned(),
                *points.unwrap(),
                timestamp.map_or_else(
                    || {
                        if record_type.unwrap() == "transaction" {
                            Some(Utc::now().timestamp())
                        } else {
                            None
                        }
                    },
                    |ts| Some(*ts),
                ),
            ),
            "update" => {
                record.unwrap().0[*index.unwrap() as usize] = (
                    name.unwrap().to_owned(),
                    *points.unwrap(),
                    timestamp.map(|ts| *ts),
                );
            }
            "delete" => {
                record.unwrap().0.remove(*index.unwrap() as usize);
            }
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
