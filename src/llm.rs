// use anyhow::Ok;
use futures::StreamExt;
use ollama_rs::generation::completion::{GenerationContext, GenerationResponse};
use ollama_rs::{
    generation::completion::request::GenerationRequest, generation::options::GenerationOptions,
    Ollama,
};
use std::fmt::format;
use std::fs;
use std::io::Write;
use tokio::io::{stdout, AsyncWriteExt};
use tokio::task;

use async_gigachat::{
    chat::{Chat, ChatCompletionRequestBuilder, ChatCompletionResponse, ChatMessageBuilder, Role},
    client::Client,
    config::GigaChatConfig,
    result,
    token::{TokenCountRequestBuilder, Tokens},
};

pub trait LLMStrategy {
    async fn generate(
        &self,
        messages: String,
        grimoire_text: String,
    ) -> Result<(), Box<dyn std::error::Error>>;
    async fn agenerate(
        &self,
        messages: String,
        grimoire_text: String,
    ) -> Result<(), Box<dyn std::error::Error>>;
}

pub struct OllamaStrategy {
    pub default_model: String,
    pub options: GenerationOptions,
    pub api_url: String,
    pub api_port: u16,
}

impl LLMStrategy for OllamaStrategy {
    async fn generate(
        &self,
        messages: String,
        grimoire_text: String,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let ollama = Ollama::new(self.api_url.clone(), self.api_port);
        let options = GenerationOptions::default()
            .temperature(0.1)
            .repeat_penalty(1.5)
            .top_k(25)
            .top_p(0.25);
        let res = ollama
            .generate(
                GenerationRequest::new(self.default_model.clone(), messages)
                    .system(grimoire_text)
                    .options(options),
            )
            .await;
        if let Ok(res) = res {
            println!("{}", res.response);
        }
        Ok(())
    }

    async fn agenerate(
        &self,
        messages: String,
        grimoire_text: String,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let ollama = Ollama::new(self.api_url.clone(), self.api_port);
        let options = GenerationOptions::default()
            .temperature(0.1)
            .repeat_penalty(1.5)
            .top_k(25)
            .top_p(0.25);
        let mut stream = ollama
            .generate_stream(
                GenerationRequest::new(self.default_model.clone(), messages)
                    .system(grimoire_text)
                    .options(options),
            )
            .await
            .unwrap();

        let mut stdout = tokio::io::stdout();
        while let Some(res) = stream.next().await {
            let responses = res.unwrap();
            for resp in responses {
                stdout.write_all(resp.response.as_bytes()).await.unwrap();
                stdout.flush().await.unwrap();
            }
        }
        Ok(())
    }
}

pub struct GigaChatStrategy {
    // pub auth_token: String,
    // pub scope: String,
    // pub auth_url: String,
    // pub api_base_url: String,
}

impl LLMStrategy for GigaChatStrategy {
    async fn generate(
        &self,
        messages: String,
        grimoire_text: String,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let config = GigaChatConfig::default();
        let client: Client = Client::with_config(config);

        let question = ChatMessageBuilder::default()
            .role(Role::System)
            .content(grimoire_text)
            .role(Role::User)
            .content(messages)
            .build()
            .unwrap();

        let request = ChatCompletionRequestBuilder::default()
            .messages(vec![question.clone()])
            .model("GigaChat:latest")
            .build()
            .unwrap();

        let response = Chat::new(client).completion(request).await.unwrap();
        println!("Used tokens: {}", response.usage.total_tokens);
        let choice = response.choices.get(0).unwrap();

        println!("{}: {}", question.role.unwrap(), question.content);
        println!(
            "{}: {}",
            choice.message.clone().role.unwrap(),
            choice.message.content
        );
        Ok(())
    }

    async fn agenerate(
        &self,
        messages: String,
        grimoire_text: String,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let config = GigaChatConfig::default();
        let client: Client = Client::with_config(config);

        let question = ChatMessageBuilder::default()
            .role(Role::System)
            .content(grimoire_text)
            .role(Role::User)
            .content(messages)
            .build()
            .unwrap();

        let request = ChatCompletionRequestBuilder::default()
            .messages(vec![question.clone()])
            .model("GigaChat:latest")
            .stream(true)
            .build()
            .unwrap();

        let mut stream = Chat::new(client).completion_stream(request).await.unwrap();
        // println!("{}", response.usage.total_tokens);
        let mut stdout = tokio::io::stdout();
        let mut message = String::default();

        while let Some(response) = stream.next().await {
            match response {
                Ok(resp) => {
                    for choice in resp.choices.iter() {
                        stdout
                            .write_all(choice.delta.content.as_bytes())
                            .await
                            .unwrap();
                        stdout.flush().await.unwrap();
                        message = format!("{} {}", message, choice.delta.content);
                    }
                }
                Err(e) => {
                    eprintln!("{:?}", e);
                }
            };
        }
        Ok(())
    }
}

pub struct Llmka<T: LLMStrategy> {
    llm_strategy: T,
}

impl<T: LLMStrategy> Llmka<T> {
    pub fn new(llm_strategy: T) -> Self {
        Self { llm_strategy }
    }

    pub async fn generate(
        &self,
        messages: String,
        grimoire_text: String,
        stream: bool,
    ) -> Result<(), Box<dyn std::error::Error>> {
        match stream {
            true => self
                .llm_strategy
                .agenerate(messages, grimoire_text)
                .await
                .unwrap(),
            false => self
                .llm_strategy
                .generate(messages, grimoire_text)
                .await
                .unwrap(),
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::error::Error;

    #[tokio::test]
    async fn test_llmka() -> Result<(), Box<dyn Error>> {
        let options = GenerationOptions::default()
            .temperature(0.1)
            .repeat_penalty(1.5)
            .top_k(25)
            .top_p(0.25);
        let llm = Llmka::new(OllamaStrategy {
            api_url: "http://localhost".to_string(),
            api_port: 11435,
            default_model: "llama3".to_string(),
            options,
        });

        llm.generate(
            "Check".to_string(),
            "You are Link from Zelda.".to_string(),
            false,
        )
        .await;

        let llm = Llmka::new(GigaChatStrategy {});
        llm.generate(
            "Check".to_string(),
            "You are Link from Zelda.".to_string(),
            false,
        )
        .await;

        Ok(())
    }
}
