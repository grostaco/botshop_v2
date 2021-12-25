use std::sync::Arc;

use serenity::{
    builder::CreateInteractionResponse, http::Http,
    model::interactions::application_command::ApplicationCommandInteraction,
};

use super::util::Records;

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
                    for record in &self.0 {}
                    embed
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
