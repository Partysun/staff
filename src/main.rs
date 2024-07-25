mod config;
mod llm;
mod utils;

use anyhow::{bail, Error, Result};
use clap::error::ContextKind;
use clap::{Args, Parser, Subcommand};
use futures::{StreamExt, TryFutureExt};
use ollama_rs::{
    generation::completion::request::GenerationRequest, generation::options::GenerationOptions,
    Ollama,
};
use regex::Regex;
use std::fmt::format;
use std::fs;
use std::fs::File;
use std::io::prelude::*;
use std::path::Path;
use tokio::io::{stdout, AsyncWriteExt};
use tokio::task;

use config::{Config, OllamaConfig};
use llm::{GigaChatStrategy, LLMStrategy, Llmka, OllamaStrategy};
use utils::{get_or_create_config, read_spell};

#[derive(Parser, Debug)]
#[command(name = "staff")]
#[command(author = "Zatsepin Yura, https://zatsepin.dev")]
#[command(version, about, long_about = None)]
struct Cli {
    // Path to a config file
    #[arg(short, long)]
    config: Option<String>,

    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand, Debug)]
enum Commands {
    #[command(about = "List of available grimoires")]
    Grimoires {
        #[clap(long, default_value = "true")]
        list: Option<bool>,
        #[command(subcommand)]
        command: Option<GrimoireCommands>,
    },
    #[command()]
    Cast {
        #[clap(short, long, default_value = "basic")]
        name: Option<String>,
        #[clap(default_value = "why is the sky blue?")]
        words: Vec<String>,
        #[clap(long, default_value = "true")]
        stream: Option<bool>,
        #[clap(short, long, default_value = "ollama/llama3.1")]
        model: String,
    },
}

#[derive(Debug, Subcommand)]
enum GrimoireCommands {
    Add { grimoire: String },
}

#[tokio::main]
async fn run(mut args: Cli) -> Result<()> {
    let cfg: Config = Config::parse(args.config).unwrap_or(Config {
        ..Config::default()
    });

    match args.command {
        Commands::Grimoires { command, list } => {
            match &command {
                Some(GrimoireCommands::Add { grimoire }) => {
                    let re = Regex::new(r"^https?:\/\/").unwrap();
                    if re.is_match(grimoire) {
                        println!("This is an HTML link: {}", grimoire);
                    } else {
                        println!("This is path local string: {}", grimoire);
                        match fs::metadata(grimoire).is_ok() {
                            true => (),
                            false => {
                                eprintln!("File does not exist: '{grimoire:}'");
                            }
                        }
                        match grimoire.ends_with(".md") {
                            true => (),
                            false => {
                                eprintln!("This is not a markdown file: '{grimoire:}'");
                            }
                        }
                        let mut grimoires_path = get_or_create_config(Some("grimoires")).unwrap();
                        grimoires_path.push(Path::new(&grimoire));
                        match fs::copy(grimoire, &grimoires_path).is_ok() {
                            true => {
                                println!("File copied successfully to: {:?}", grimoires_path);
                            }
                            false => {
                                eprintln!("Failed to copy file");
                            }
                        }
                    }
                }
                _ => {
                    println!("List of available grimoires: ");
                    let grimoires_path = get_or_create_config(Some("grimoires")).unwrap();
                    // show all files in the folder without extension name
                    match fs::read_dir(grimoires_path) {
                        Ok(entries) => {
                            for entry in entries {
                                match entry {
                                    Ok(entry) => {
                                        let path = entry.path().with_extension("");
                                        let filename = path.file_name().unwrap().to_string_lossy();
                                        println!("  * {}", filename);
                                    }
                                    Err(e) => eprintln!("Error: {}", e),
                                }
                            }
                        }
                        Err(e) => eprintln!("Error: {}", e),
                    }
                }
            }
        }
        Commands::Cast {
            name,
            words,
            stream,
            model,
        } => {
            let options = GenerationOptions::default()
                .temperature(0.1)
                .repeat_penalty(1.5)
                .top_k(25)
                .top_p(0.25);
            let spell = read_spell(&name);
            println!("Grimoire: {:?}", name.unwrap());
            let messages: String = words.join(" ");
            let stream = stream.unwrap_or(true);
            let re = Regex::new(r"^ollama\/(.*)").unwrap();
            match re.captures(&model).map(|m| m.get(1)) {
                Some(Some(x)) => {
                    if (cfg.ollama.is_none()) {
                        bail!("You should provide Ollama configuration in the config file. See help -h or --help.");
                    }
                    let model = x.as_str().to_string();
                    let ollama_cfg: OllamaConfig = cfg.ollama.unwrap();
                    let llm = Llmka::new(OllamaStrategy {
                        api_url: ollama_cfg.api_url,   //"http://localhost".to_string(),
                        api_port: ollama_cfg.api_port, //11435,
                        default_model: model,
                        options,
                    });
                    llm.generate(messages, spell, stream).await;
                } // llama3
                _ if model == "giga" => {
                    if (cfg.giga.unwrap().auth_token.is_none()) {
                        bail!("You should provide GigaChat configuration in the config file. See help -h or --help.");
                    }
                    let llm = Llmka::new(GigaChatStrategy {});
                    llm.generate(messages, spell, stream).await;
                } // giga
                _ => {}
            }
        }
    }

    Ok(())
}

fn main() {
    if let Err(e) = run(Cli::parse()) {
        eprintln!("{e}");
        std::process::exit(1);
    }
}
