use chrono::{DateTime, Local};
use clap::{Parser, Subcommand};
use owo_colors::OwoColorize;
use serde::{Deserialize, Serialize};

use color_eyre::eyre::Result;

#[derive(Debug, Serialize, Deserialize)]
struct Note {
    id: usize,
    title: String,
    body: String,
    created: DateTime<Local>,
}

#[derive(Parser)]
#[command(
    name = "notectl",
    author = "Junkai",
    version,
    about = "Take notes from your terminal 🔅",
    long_about = "A beautiful, local-first note-taking command-line application written in Rust."
)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    Add {
        title: String,
        body: Vec<String>,
    },

    List {
        #[arg(short, long)]
        verbose: bool,
    },

    View {
        id: usize,
    },
    Delete {
        id: usize,
    },

    Search {
        query: String,
    },
}

fn main() -> Result<()> {
    color_eyre::install()?;
    print_banner();

    Ok(())
}

fn print_banner() {
    use figlet_rs::FIGfont;
    let font = FIGfont::standard().unwrap();
    let figure = font.convert("Notectl").unwrap();
    println!("{}", figure.to_string().bright_magenta());
}
