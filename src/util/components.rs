use std::{sync::Arc, time::Duration};

use serenity::{
    async_trait,
    builder::CreateInteractionResponse,
    client::bridge::gateway::ShardMessenger,
    futures::{lock::Mutex, StreamExt},
    http::Http,
    model::interactions::{
        application_command::ApplicationCommandInteraction,
        message_component::{ComponentType, MessageComponentInteraction},
    },
};

#[async_trait]
pub trait Component: Send + Sync {
    fn delegate_response<'a>(
        &self,
        response: &'a mut CreateInteractionResponse,
    ) -> &'a mut CreateInteractionResponse;
    fn want_component_interaction(&self, component_interaction_type: ComponentType) -> bool;
    async fn on_interaction(
        &mut self,
        http: &Arc<Http>,
        interaction: &Arc<MessageComponentInteraction>,
    ) -> Result<(), serenity::Error>;
}
pub struct ComponentManager {
    components: Vec<Box<dyn Component>>,
}

impl ComponentManager {
    pub fn new() -> Self {
        Self {
            components: Vec::new(),
        }
    }

    pub fn add_component(&mut self, component: Box<dyn Component>) {
        self.components.push(component)
    }

    pub async fn handle_interaction(
        &mut self,
        http: &Arc<Http>,
        interaction: ApplicationCommandInteraction,
        shard_messenger: &ShardMessenger,
        timeout: u64,
    ) -> Result<(), serenity::Error> {
        interaction
            .create_interaction_response(http, |response| {
                self.components
                    .iter()
                    .fold(response, |acc, component| component.delegate_response(acc))
            })
            .await?;

        let collector = interaction
            .get_interaction_response(http)
            .await
            .unwrap()
            .await_component_interactions(shard_messenger)
            .timeout(Duration::from_secs(timeout))
            .await;

        let components = &Arc::new(Mutex::new(&mut self.components));
        collector
            .for_each(|interaction| async move {
                for component in components.lock().await.iter_mut().filter(|component| {
                    component.want_component_interaction(interaction.data.component_type)
                }) {
                    component
                        .on_interaction(http, &interaction)
                        .await
                        .map_err(|_e| panic!("Unable to handle interaction"))
                        .unwrap();
                }
            })
            .await;

        Ok(())
    }
}
