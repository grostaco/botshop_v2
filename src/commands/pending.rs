use crate::util::db::User;
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
use std::{sync::Arc, time::Duration};

use super::util::{get_today, get_tomorrow};
use crate::util::Records;

pub struct Pending {
    db_file: String,
    user: User,
}

impl Pending {
    pub fn new(db_file: &str, user_id: u64) -> Self {
        let mut user = User::from_file(db_file, user_id).unwrap();
        user.pending = Records(
            user.pending
                .into_iter()
                .filter(|record| match record.2 {
                    Some(timestamp) => {
                        let days = DateTime::<Utc>::from_utc(
                            NaiveDateTime::from_timestamp(timestamp, 0),
                            Utc,
                        );
                        days.num_days_from_ce() == get_today().num_days_from_ce()
                    }
                    None => true,
                })
                .collect::<Vec<_>>(),
        );

        Self {
            db_file: db_file.to_owned(),
            user,
        }
    }

    fn complete_task(&mut self, task_name: &str) -> Option<()> {
        let record = self
            .user
            .pending
            .iter_mut()
            .filter(|record| record.0 == task_name)
            .next();
        if let Some(record) = record {
            record.2 = Some(DateTime::timestamp(&Utc::now()));
            self.user
                .transactions
                .push(record.0.to_owned(), record.1, record.2);
            self.user
                .update(&self.db_file)
                .expect("Cannot update db for user");
            Some(())
        } else {
            None
        }
    }

    fn delegate_interaction_response<'a>(
        &self,
        interaction: &'a mut CreateInteractionResponse,
    ) -> &'a mut CreateInteractionResponse {
        if self.user.pending.len() == 0 {
            return interaction.interaction_response_data(|data| {
                data.create_embed(|embed| {
                    embed
                        .title("You have no pending tasks! :tada: Good work!")
                        .description("Give yourself a pat on the back :>")
                        .footer(|footer| footer.text("Don't be too happy, more will come"))
                        .color((174, 243, 89))
                })
            });
        }

        let mut tasks = String::new();
        let mut rewards = String::new();
        let mut when = String::new();
        let mut completed = 0;

        for record in &self.user.pending {
            tasks.push_str(&format!("{}\n", record.0));
            rewards.push_str(&format!(":coin:x{}\n", record.1));
            when.push_str(&match record.2 {
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
            })
        }

        let completed = completed as f32 / self.user.pending.len() as f32;

        interaction.interaction_response_data(|data| {
            data.create_embed(|embed| {
                embed
                    .title(format!(
                        "Pending tasks! :D ({}% completed)",
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
                if self.user.pending.iter().any(|record| record.2.is_none()) {
                    components.create_action_row(|row| {
                        row.create_select_menu(|menu| {
                            menu.options(|options| {
                                for record in &self.user.pending {
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
            .await?
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
