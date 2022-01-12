use serenity::{
    builder::CreateInteractionResponse, client::bridge::gateway::ShardMessenger, http::Http,
    model::interactions::application_command::ApplicationCommandInteraction, model::user::User,
};
use std::{
    cmp::{max, min},
    sync::Arc,
};

use crate::util::db::{self, query_user};

pub struct Info(db::User);

impl Info {
    pub fn new(db_path: &str, user_id: u64) -> Self {
        Self(
            query_user(db_path, user_id)
                .expect("Cannot cannot to the database")
                .expect("Get user under the provided id"),
        )
    }

    fn delegate_interaction_response<'a>(
        &self,
        interaction: &'a mut CreateInteractionResponse,
        user: &User,
    ) -> &'a mut CreateInteractionResponse {
        interaction.interaction_response_data(|data| {
            data.create_embed(|embed| {
                embed
                    .title("About you!")
                    .field(
                        "__**Points Gathered**__",
                        self.0
                            .transactions
                            .iter()
                            .fold(0, |acc, record| acc + max(0, record.1)),
                        true,
                    )
                    .field(
                        "__**Points Spent**__",
                        self.0
                            .transactions
                            .iter()
                            .fold(0, |acc, record| acc + -min(0, record.1)),
                        true,
                    )
                    .field(
                        "__**Points Balance**__",
                        self.0
                            .transactions
                            .iter()
                            .fold(0, |acc, record| acc + record.1),
                        true,
                    )
                    .thumbnail(user.avatar_url().unwrap())
            })
        })
    }

    pub async fn handle_interaction(
        &self,
        http: &Arc<Http>,
        interaction: ApplicationCommandInteraction,
        shard_messenger: &ShardMessenger,
    ) -> Result<(), serenity::Error> {
        interaction
            .create_interaction_response(http, |response| {
                self.delegate_interaction_response(response, &interaction.user)
            })
            .await?;

        Ok(())
    }
}
