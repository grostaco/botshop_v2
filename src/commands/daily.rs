use std::{sync::Arc, time::Duration};

use chrono::{DateTime, Datelike, NaiveDateTime, Utc};
use interpolation::lerp;
use serenity::{
    builder::CreateInteractionResponse,
    client::bridge::gateway::ShardMessenger,
    futures::{lock::Mutex, StreamExt},
    http::Http,
    model::interactions::{
        application_command::ApplicationCommandInteraction, InteractionResponseType,
    },
    utils::Color,
};

use super::util::{get_today, get_tomorrow};
use crate::util::db::User;

/// A struct to represent every daily tasks and corresponding files
pub struct Daily {
    db_file: String,
    user: User,
}

impl Daily {
    pub fn new(db_file: &str, user_id: u64) -> Self {
        let mut user = User::from_file(db_file, user_id).unwrap();
        user.daily.iter_mut().for_each(|mut record| {
            if record.2.is_some() {
                let days = DateTime::<Utc>::from_utc(
                    NaiveDateTime::from_timestamp(record.2.unwrap(), 0),
                    Utc,
                );
                if days.num_days_from_ce() != get_today().num_days_from_ce() {
                    record.2 = None;
                }
            }
        });

        Self {
            db_file: db_file.to_owned(),
            user,
        }
    }

    fn complete_task(&mut self, task_name: &str) -> Option<()> {
        let record = self
            .user
            .daily
            .iter_mut()
            .filter(|record| record.0 == task_name)
            .next();
        if let Some(record) = record {
            record.2 = Some(DateTime::timestamp(&Utc::now()));
            self.user
                .transactions
                .push(record.0.to_string(), record.1, record.2);
            self.user
                .update(&self.db_file)
                .expect("Cannot update user to database");
            Some(())
        } else {
            None
        }
    }

    fn delegate_interaction_response<'a>(
        &self,
        interaction: &'a mut CreateInteractionResponse,
    ) -> &'a mut CreateInteractionResponse {
        if self.user.daily.len() == 0 {
            return interaction.interaction_response_data(|data| {
                data.create_embed(|embed| {
                    embed
                        .title("This is a little awkward")
                        .description("You have no daily tasks?")
                        .footer(|footer| footer.text("Add something :< Self improvement happens when you have a routine!"))
                })
            });
        }

        let mut completed = 0;

        let (tasks, rewards, when) = self.user.daily.iter().fold(
            (String::new(), String::new(), String::new()),
            |e, record| {
                (
                    e.0 + &format!("{}\n", record.0),
                    e.1 + &format!(":coin:x{}\n", record.1),
                    e.2 + &match record.2 {
                        Some(timestamp) => {
                            completed += 1;
                            let timestamp = DateTime::timestamp(&Utc::now()) - timestamp as i64;
                            format!(
                                "✅ Completed *{}h {}m {}s ago*\n",
                                timestamp / 3600,
                                timestamp % 3600 / 60,
                                timestamp % 3600 % 60
                            )
                        }
                        None => "⌛ Not Completed\n".to_owned(),
                    },
                )
            },
        );

        let completed: f32 = completed as f32 / self.user.daily.len() as f32;

        interaction.interaction_response_data(|data| {
            data.create_embed(|embed| {
                embed
                    .title(format!(
                        "Daily tasks! :D ({}% completed)",
                        (completed * 100_f32) as u64
                    ))
                    .field("Task", tasks, true)
                    .field("Rewards", rewards, true)
                    .field("Progress", when, true)
                    .color(Color::from_rgb(
                        lerp(&227, &174, &completed),
                        lerp(&36, &243, &completed),
                        lerp(&43, &89, &completed),
                    ))
                    .footer(|footer| {
                        let elapsed = (get_tomorrow() - Utc::now()).num_seconds();
                        footer.text(format!(
                            "{}h {}m {}s until refresh",
                            elapsed / 3600,
                            elapsed % 3600 / 60,
                            elapsed % 3600 % 60
                        ))
                    })
            })
            .components(|components| {
                if self.user.daily.iter().any(|record| record.2.is_none()) {
                    components.create_action_row(|row| {
                        row.create_select_menu(|menu| {
                            menu.options(|options| {
                                for record in &self.user.daily {
                                    if record.2.is_none() {
                                        options.create_option(|option| {
                                            option
                                                .label(&record.0)
                                                .description(&format!("{}x coins", record.1))
                                                .value(&record.0)
                                        });
                                    }
                                }
                                options
                            })
                            .placeholder("Pick your poison :>")
                            .custom_id("complete_daily_menu")
                        })
                    })
                } else {
                    components
                }
            });
            data
        })
    }

    pub async fn handle_interaction(
        &mut self,
        http: &Arc<Http>,
        interaction: ApplicationCommandInteraction,
        shard_messenger: &ShardMessenger,
    ) -> Result<(), serenity::Error> {
        interaction
            .create_interaction_response(http, |interaction| {
                self.delegate_interaction_response(interaction)
            })
            .await?;

        let collector = interaction
            .get_interaction_response(http)
            .await
            .unwrap()
            .await_component_interactions(shard_messenger)
            .timeout(Duration::from_secs(15))
            .author_id(self.user.id)
            .await;

        let daily = &Arc::new(Mutex::new(self));
        collector
            .for_each(|interaction| async move {
                let mut daily = daily.lock().await;
                daily.complete_task(&interaction.data.values[0]);
                interaction
                    .create_interaction_response(http, |interaction| {
                        daily
                            .delegate_interaction_response(interaction)
                            .kind(InteractionResponseType::UpdateMessage)
                    })
                    .await
                    .expect("Unable to update interaction");
            })
            .await;

        Ok(())
    }
}
