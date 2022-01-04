/*
use std::sync::Arc;

use serenity::{
    builder::CreateInteractionResponse, client::bridge::gateway::ShardMessenger, http::Http,
    model::interactions::application_command::ApplicationCommandInteraction,
};

use crate::util::Records;

pub struct Info(Records);

impl Info {
    pub fn new() -> Self {
        Self(Records::new())
    }

    fn delegate_interaction_response<'a>(
        &self,
        interaction: &'a mut CreateInteractionResponse,
    ) -> &'a mut CreateInteractionResponse {
        interaction.interaction_response_data(|data| {
            data.create_embed(|embed| embed.)
        })
    }

    pub async fn handle_interaction(
        &self,
        http: &Arc<Http>,
        interaction: ApplicationCommandInteraction,
        shard_messenger: &ShardMessenger,
    ) -> Result<(), serenity::Error> {
        interaction
            .create_interaction_response(http, |interaction| {
                self.delegate_interaction_response(interaction)
            })
            .await?;

        Ok(())
    }
}
 */
