use std::{sync::Arc, time::Duration};

use serenity::{
    async_trait,
    builder::CreateInteractionResponse,
    client::bridge::gateway::ShardMessenger,
    futures::{lock::Mutex, StreamExt},
    http::Http,
    model::interactions::{
        application_command::ApplicationCommandInteraction,
        message_component::MessageComponentInteraction, InteractionType,
    },
};

#[async_trait]
pub trait Component {
    fn delegate_component<'a>(
        &self,
        components: &'a mut CreateInteractionResponse,
    ) -> &'a mut CreateInteractionResponse;
    fn want_interaction(&self, interaction_type: InteractionType) -> bool;
    async fn on_interaction(
        &self,
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

    pub async fn handle_interaction<F>(
        &mut self,
        http: &Arc<Http>,
        interaction: ApplicationCommandInteraction,
        shard_messenger: &ShardMessenger,
    ) -> Result<(), serenity::Error> {
        interaction
            .create_interaction_response(http, |response| {
                self.components
                    .iter()
                    .fold(response, |acc, component| component.delegate_component(acc))
            })
            .await?;

        let collector = interaction
            .get_interaction_response(http)
            .await
            .unwrap()
            .await_component_interactions(shard_messenger)
            .timeout(Duration::from_secs(15))
            .await;

        let components = &Arc::new(Mutex::new(&self.components));
        collector
            .for_each(|interaction| async move {
                for component in components
                    .lock()
                    .await
                    .iter()
                    .filter(|component| component.want_interaction(interaction.kind))
                {
                    component
                        .on_interaction(&interaction)
                        .await
                        .map_err(|_e| panic!("Unable to handle interaction"))
                        .unwrap();
                }
            })
            .await;

        Ok(())
    }
}
