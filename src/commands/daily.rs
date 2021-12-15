use std::{
    fs::{File, OpenOptions},
    io::Write,
    sync::Arc,
    time::Duration,
};

use chrono::{DateTime, Utc};
use csv::Writer;
use interpolation::lerp;
use serenity::{
    builder::CreateInteractionResponse,
    client::bridge::gateway::ShardMessenger,
    futures::{lock::Mutex, StreamExt},
    http::Http,
    model::{
        channel::ReactionType,
        id::EmojiId,
        interactions::{
            application_command::ApplicationCommandInteraction, InteractionResponseType,
        },
    },
    utils::Color,
};

use crate::commands::util::get_tomorrow_midnight;

/// A struct to represent every daily tasks and corresponding files
pub struct Daily {
    /// The file to load daily tasks from
    source_file: String,
    transaction_file: File,
    records: Vec<(String, u8, Option<i64>)>,
}

impl Daily {
    pub fn new(source_file: &str, transaction_file: &str) -> Self {
        let mut rdr =
            csv::Reader::from_path(source_file).expect("Cannot create Reader from source file");
        let mut records = Vec::new();
        for record in rdr.records() {
            let record = record.expect("Record cannot be read");
            records.push((
                record.get(0).expect("Expected task name").to_owned(),
                record
                    .get(1)
                    .expect("Expected points")
                    .parse::<u8>()
                    .expect("Expected points to be integral"),
                match record.get(2).expect("Expected completed") {
                    "None" => None,
                    timestamp => Some(
                        timestamp
                            .parse::<i64>()
                            .expect("Expected timestamp as an integer"),
                    ),
                },
            ));
        }

        Self {
            source_file: source_file.to_owned(),
            transaction_file: OpenOptions::new()
                .append(true)
                .write(true)
                .open(transaction_file)
                .expect("Unable to transaction file"),

            records,
        }
    }

    fn complete_task(&mut self, task_name: &str) -> Option<()> {
        let record = self
            .records
            .iter_mut()
            .filter(|record| record.0 == task_name)
            .next();
        if let Some(record) = record {
            record.2 = Some(DateTime::timestamp(&Utc::now()));
            self.transaction_file
                .write(
                    format!(
                        "{},{},{}\n",
                        record.0.to_owned(),
                        record.1.to_string(),
                        match record.2 {
                            Some(timestamp) => timestamp.to_string(),
                            None => "None".to_owned(),
                        },
                    )
                    .as_bytes(),
                )
                .expect("Unable to commit transactions");

            self.sync_with_source();
            Some(())
        } else {
            None
        }
    }

    fn sync_with_source(&self) {
        let mut wtr = Writer::from_writer(
            OpenOptions::new()
                .truncate(true)
                .write(true)
                .open(&self.source_file)
                .unwrap(),
        );

        wtr.write_record(&["task", "points", "completed"])
            .expect("Unable to write header to source file");
        for record in &self.records {
            wtr.write_record(&[
                record.0.to_owned(),
                record.1.to_string(),
                match record.2 {
                    Some(timestamp) => timestamp.to_string(),
                    None => "None".to_owned(),
                },
            ])
            .expect("Unable to write record to source file");
        }
    }

    fn delegate_interaction_response<'a>(
        &self,
        interaction: &'a mut CreateInteractionResponse,
    ) -> &'a mut CreateInteractionResponse {
        let mut tasks = String::new();
        let mut rewards = String::new();
        let mut when = String::new();
        let mut completed = 0;

        for record in &self.records {
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

        let completed: f32 = completed as f32 / self.records.len() as f32;

        interaction.interaction_response_data(|data| {
            data.create_embed(|embed| {
                embed
                    .title("Daily tasks! :D")
                    .field("Task", tasks, true)
                    .field("Rewards", rewards, true)
                    .field("Progress", when, true)
                    .color(Color::from_rgb(
                        lerp(&227, &174, &completed),
                        lerp(&36, &243, &completed),
                        lerp(&43, &89, &completed),
                    ))
                    .footer(|footer| {
                        let elapsed = (get_tomorrow_midnight() - Utc::now()).num_seconds();
                        footer.text(format!(
                            "{}h {}m {}s until refresh",
                            elapsed / 3600,
                            elapsed % 3600 / 60,
                            elapsed % 3600 % 60
                        ))
                    })
            })
            .components(|components| {
                if self.records.iter().any(|record| record.2.is_none()) {
                    components.create_action_row(|row| {
                        row.create_select_menu(|menu| {
                            menu.options(|options| {
                                for record in &self.records {
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

#[cfg(test)]
mod tests {
    use std::error::Error;
    use std::fs;
    use std::io::Write;

    use super::*;
    use tempfile::{Builder, NamedTempFile};

    fn create_temporary(prefix: &str, suffix: &str) -> Result<NamedTempFile, Box<dyn Error>> {
        Ok(Builder::new().prefix(prefix).suffix(suffix).tempfile()?)
    }

    #[test]
    fn create_daily() {
        let mut source_file = create_temporary("daily", ".csv").expect("Unable to create tempfile");

        let transaction_file =
            create_temporary("transaction", ".csv").expect("Unable to create tempfile");

        source_file
            .write(b"task,points,completed\ntask1,8,None\ntask2,8,3222")
            .expect("Unable to write source file");

        let daily = Daily::new(
            source_file.path().to_str().unwrap(),
            transaction_file.path().to_str().unwrap(),
        );

        assert_eq!(
            daily.records,
            vec![
                ("task1".to_owned(), 8, None),
                ("task2".to_owned(), 8, Some(3222))
            ]
        );
    }

    #[test]
    fn complete_daily() {
        let mut source_file = create_temporary("daily", ".csv").expect("Unable to create tempfile");
        let transaction_file =
            create_temporary("transaction", ".csv").expect("Unable to create tempfile");

        source_file
            .write(b"task,points,completed\ntask1,8,None\ntask2,8,3222\ntask3,7,None")
            .expect("Unable to write source file");

        let mut daily = Daily::new(
            source_file.path().to_str().unwrap(),
            transaction_file.path().to_str().unwrap(),
        );

        daily.complete_task("task1");
        assert!(daily.records[0].2.is_some());
        let time = daily.records[0].2.unwrap();

        assert_eq!(
            fs::read_to_string(source_file.path().to_str().unwrap()).unwrap(),
            format!(
                "task,points,completed\ntask1,8,{}\ntask2,8,3222\ntask3,7,None\n",
                time
            )
        );
        assert_eq!(
            fs::read_to_string(transaction_file.path().to_str().unwrap()).unwrap(),
            format!("task1,8,{}\n", time)
        );

        daily.complete_task("task3");
        assert!(daily.records[2].2.is_some());
        let time_three = daily.records[2].2.unwrap();

        assert_eq!(
            fs::read_to_string(source_file.path().to_str().unwrap()).unwrap(),
            format!(
                "task,points,completed\ntask1,8,{}\ntask2,8,3222\ntask3,7,{}\n",
                time, time_three
            )
        );
        assert_eq!(
            fs::read_to_string(transaction_file.path().to_str().unwrap()).unwrap(),
            format!("task1,8,{}\ntask3,7,{}\n", time, time_three)
        );
    }
}
