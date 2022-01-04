use chrono::{DateTime, NaiveDateTime, Utc};
use std::sync::Arc;

use serenity::{
    async_trait,
    builder::{CreateComponents, CreateEmbed, CreateInteractionResponse},
    client::bridge::gateway::ShardMessenger,
    http::Http,
    model::interactions::{
        application_command::ApplicationCommandInteraction,
        message_component::{ButtonStyle, ComponentType, MessageComponentInteraction},
        InteractionResponseType,
    },
};

use crate::util::{db::User, Component, ComponentManager, Records};

pub struct Transactions(ComponentManager);

const CHUNK_SIZE: usize = 10;

impl Transactions {
    pub fn new(db_file: &str, user_id: u64) -> Self {
        let user = User::from_file(db_file, user_id).unwrap();
        let mut component_mgr = ComponentManager::new();
        component_mgr.add_component(Box::new(Page::new(user.transactions)));
        Self(component_mgr)
    }

    pub async fn handle_interaction(
        &mut self,
        http: &Arc<Http>,
        interaction: ApplicationCommandInteraction,
        shard: &ShardMessenger,
    ) -> Result<(), serenity::Error> {
        self.0
            .handle_interaction(http, interaction, shard, 15)
            .await?;
        Ok(())
    }
}

struct Page {
    records: Records,
    index: usize,
}

impl Page {
    fn new(records: Records) -> Self {
        Self { records, index: 0 }
    }

    fn get_embed(&self) -> CreateEmbed {
        let mut embed = CreateEmbed::default();
        let records = self
            .records
            .iter()
            .skip(self.index * CHUNK_SIZE)
            .take(CHUNK_SIZE)
            .collect::<Vec<_>>();
        if records.len() != 0 {
            let (task, points, completed) = records.iter().take(10).fold(
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
                .field("Date", completed, true);
        } else {
            embed
                .title("It's a little empty here?")
                .description("Sorry, you don't have any transaction history :(");
        }

        embed
    }

    fn delegate_component<'a>(
        &self,
        component: &'a mut CreateComponents,
    ) -> &'a mut CreateComponents {
        let max_page = (self.records.len() as f64 / CHUNK_SIZE as f64).ceil() as usize;
        component.create_action_row(|row| {
            row.create_button(|button| {
                button
                    .label("⬅️")
                    .custom_id("left_page_select")
                    .style(ButtonStyle::Primary)
                    .disabled(self.index == 0)
            })
            .create_button(|button| {
                button
                    .label(&format!("Page {}/{}", self.index + 1, max_page,))
                    .custom_id("page_display")
                    .style(ButtonStyle::Secondary)
                    .disabled(true)
            })
            .create_button(|button| {
                button
                    .label("➡️")
                    .custom_id("right_page_select")
                    .style(ButtonStyle::Primary)
                    .disabled(self.index + 1 >= max_page)
            })
        })
    }
}

#[async_trait]
impl Component for Page {
    fn want_component_interaction(&self, component_interaction_type: ComponentType) -> bool {
        component_interaction_type == ComponentType::Button
    }

    fn delegate_response<'a>(
        &self,
        response: &'a mut CreateInteractionResponse,
    ) -> &'a mut CreateInteractionResponse {
        response.interaction_response_data(|data| {
            data.add_embed(self.get_embed())
                .components(|component| self.delegate_component(component))
        })
    }

    async fn on_interaction(
        &mut self,
        http: &Arc<Http>,
        interaction: &Arc<MessageComponentInteraction>,
    ) -> Result<(), serenity::Error> {
        if interaction.data.component_type == ComponentType::Button {
            match interaction.data.custom_id.as_str() {
                "left_page_select" => self.index -= 1,
                "right_page_select" => self.index += 1,
                _ => panic!(),
            }
            interaction
                .create_interaction_response(http, |response| {
                    self.delegate_response(response)
                        .kind(InteractionResponseType::UpdateMessage)
                })
                .await
                .expect("Unable to update interaction");
        } else {
            panic!(
                "Unexpectedly received an interaction of type {}\n",
                interaction.data.component_type as u8
            )
        }
        Ok(())
    }
}
