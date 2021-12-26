use std::{iter::Take, sync::Arc, time::Duration};

use serenity::{
    builder::{CreateComponents, CreateEmbed, CreateInteractionResponse},
    client::bridge::gateway::ShardMessenger,
    futures::StreamExt,
    http::Http,
    model::interactions::{
        application_command::ApplicationCommandInteraction,
        message_component::{ButtonStyle, ComponentType},
        InteractionResponseType,
    },
    prelude::Mutex,
};

pub struct Page<T> {
    items: Vec<T>,
    chunk_size: usize,
    index: usize,
}

impl<T> Page<T> {
    pub fn new(chunk_size: usize) -> Self {
        Self {
            items: Vec::new(),
            chunk_size: chunk_size,
            index: 0,
        }
    }

    fn delegate_component<'a>(
        &self,
        component: &'a mut CreateComponents,
    ) -> &'a mut CreateComponents {
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
                    .label(&format!(
                        "Page {}/{}",
                        self.index + 1,
                        self.items.len() / self.chunk_size
                    ))
                    .custom_id("page_display")
                    .style(ButtonStyle::Secondary)
                    .disabled(true)
            })
            .create_button(|button| {
                button
                    .label("➡️")
                    .custom_id("right_page_select")
                    .style(ButtonStyle::Primary)
                    .disabled(self.index + 1 >= self.items.len() / self.chunk_size)
            })
        })
    }

    fn delegate_interaction_response<'a, F>(
        &self,
        response: &'a mut CreateInteractionResponse,
        embed_builder: F,
    ) -> &'a mut CreateInteractionResponse
    where
        F: FnOnce(Vec<&T>) -> CreateEmbed,
    {
        let items = self
            .items
            .iter()
            .skip(self.index * self.chunk_size)
            .take(self.chunk_size)
            .collect::<Vec<_>>();

        response.interaction_response_data(|data| {
            data.add_embed(embed_builder(items))
                .components(|component| self.delegate_component(component))
        })
    }

    pub async fn handle_interaction<F>(
        &mut self,
        http: &Arc<Http>,
        interaction: ApplicationCommandInteraction,
        shard_messenger: &ShardMessenger,
        embed_builder: F,
    ) -> Result<(), serenity::Error>
    where
        F: FnOnce(Vec<&T>) -> CreateEmbed + Copy,
    {
        interaction
            .create_interaction_response(http, |response| {
                self.delegate_interaction_response(response, embed_builder)
            })
            .await?;

        let collector = interaction
            .get_interaction_response(http)
            .await
            .unwrap()
            .await_component_interactions(shard_messenger)
            .timeout(Duration::from_secs(15))
            .await;

        let page = &Arc::new(Mutex::new(self));
        collector
            .for_each(|interaction| async move {
                let mut page = page.lock().await;
                if interaction.data.component_type == ComponentType::Button {
                    match interaction.data.custom_id.as_str() {
                        "left_page_select" => page.index -= 1,
                        "right_page_select" => page.index += 1,
                        _ => panic!(),
                    }
                    interaction
                        .create_interaction_response(http, |response| {
                            page.delegate_interaction_response(response, embed_builder)
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
            })
            .await;

        Ok(())
    }

    pub fn push(&mut self, item: T) {
        self.items.push(item);
    }
}
