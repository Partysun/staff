mod llm;
mod utils;

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

use llm::{a_generate_text, generate_text};
use utils::{get_or_create_config, read_spell};

#[derive(Parser, Debug)]
#[command(name = "staff")]
#[command(author = "Zatsepin Yura, https://zatsepin.dev")]
#[command(version, about, long_about = None)]
struct Cli {
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
    },
}

#[derive(Debug, Subcommand)]
enum GrimoireCommands {
    Add { grimoire: String },
}

#[tokio::main]
async fn run(mut args: Cli) -> Result<(), Box<dyn std::error::Error>> {
    println!("{:?}", args);
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
                                eprintln!("This is not markdown file: '{grimoire:}'");
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
        } => {
            let ollama = Ollama::new("http://localhost".to_string(), 11435);
            let model = "llama3:latest".to_string();
            let options = GenerationOptions::default()
                .temperature(0.1)
                .repeat_penalty(1.5)
                .top_k(25)
                .top_p(0.25);
            let spell = read_spell(&name);
            println!("Grimoire: {:?}", name.unwrap());
            println!("Magic: {:?}", spell);
            let message: String = words.join(" ");
            println!("Message: {:?}", message);
            println!("\n");
            match stream {
                Some(true) => a_generate_text(ollama, model, spell, message, options).await?,
                _ => generate_text(ollama, model, spell, message, options).await?,
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
