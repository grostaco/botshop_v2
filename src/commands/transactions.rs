use chrono::{DateTime, NaiveDateTime, Utc};
use std::sync::Arc;

use serenity::{
    builder::CreateInteractionResponse, http::Http,
    model::interactions::application_command::ApplicationCommandInteraction,
};

use crate::util::records::Records;

pub struct Transactions(Records);

impl Transactions {
    pub fn new(transaction_file: &str) -> Self {
        Self(Records::from_file(transaction_file).expect("Unable to open process transaction file"))
    }

    fn delegate_interaction_response<'a>(
        &self,
        response: &'a mut CreateInteractionResponse,
    ) -> &'a mut CreateInteractionResponse {
        response.interaction_response_data(|data| {
            if self.0.len() != 0 {
                data.create_embed(|embed| {
                    let (task, points, completed) = self.0.iter().take(10).fold(
                        (String::new(), String::new(), String::new()),
                        |a, b| {
                            (
                                a.0 + &b.0 + "\n",
                                a.1 + &b.1.to_string() + "\n",
                                a.2 + &DateTime::<Utc>::from_utc(
                                    NaiveDateTime::from_timestamp(b.2.unwrap(), 0),
                                    Utc,
                                )
                                .format("%m/%d/%Y (%I:%M %p)\n")
                                .to_string(),
                            )
                        },
                    );

                    embed
                        .title("Transactions history :>")
                        .field("Transaction Name", task, true)
                        .field("Coins", points, true)
                        .field("Date", completed, true)
                })
            } else {
                data.create_embed(|embed| {
                    embed
                        .title("It's a little empty here?")
                        .description("Sorry, you don't have any transaction history :(")
                })
            }
        })
    }

    pub async fn handle_interaction(
        &self,
        http: &Arc<Http>,
        interaction: ApplicationCommandInteraction,
    ) -> Result<(), serenity::Error> {
        interaction
            .create_interaction_response(http, |response| {
                self.delegate_interaction_response(response)
            })
            .await?;

        Ok(())
    }
}
