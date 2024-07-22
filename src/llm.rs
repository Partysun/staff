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

pub async fn generate_text(
    ollama: Ollama,
    model: String,
    grimoire_text: String,
    messages: String,
    options: GenerationOptions,
) -> Result<(), Box<dyn std::error::Error>> {
    let res = ollama
        .generate(
            GenerationRequest::new(model, messages)
                .system(grimoire_text)
                .options(options),
        )
        .await;
    if let Ok(res) = res {
        println!("{}", res.response);
    }
    Ok(())
}

pub async fn a_generate_text(
    ollama: Ollama,
    model: String,
    grimoire_text: String,
    messages: String,
    options: GenerationOptions,
) -> Result<(), Box<dyn std::error::Error>> {
    let mut stream = ollama
        .generate_stream(
            GenerationRequest::new(model, messages)
                .system(grimoire_text)
                .options(options),
        )
        .await
        .unwrap();

    let mut stdout = tokio::io::stdout();
    while let Some(res) = stream.next().await {
        let responses = res.unwrap();
        for resp in responses {
            stdout.write(resp.response.as_bytes()).await.unwrap();
            stdout.flush().await.unwrap();
        }
    }
    Ok(())
}
