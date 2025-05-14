use std::io::{self};
use std::{fs, path::PathBuf};

use chrono::{DateTime, Local};
use clap::{Parser, Subcommand};
use directories::ProjectDirs;
use owo_colors::OwoColorize;
use serde::{Deserialize, Serialize};

use color_eyre::eyre::{Result, eyre};

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
    about = "Take notes straight from your terminal — with style ✨",
    long_about = "A beautiful, local-first note-taking command-line application written in Rust."
)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Add a new note
    #[command(about = "Add a new note")]
    Add {
        /// Title of the note (required)
        #[arg(short, long)]
        title: String,

        /// Body (omit to enter via stdin)
        #[arg(short, long)]
        body: Vec<String>,
    },
    /// List notes
    #[command(about = "List existing notes")]
    List {
        #[arg(short, long, help = "Show full body text for each note")]
        verbose: bool,
    },
    /// View a note by ID
    #[command(about = "Show a note")]
    View {
        #[arg(short, long, value_name = "ID")]
        id: usize,
    },
    /// Delete a note by ID
    #[command(about = "Delete a note")]
    Delete {
        /// Note ID
        id: usize,
    },
    /// Search for notes containing a query string
    #[command(about = "Search notes")]
    Search {
        #[arg(short, long, value_name = "QUERY")]
        query: String,
    },
}

fn main() -> Result<()> {
    color_eyre::install()?;
    let cli = Cli::parse();
    print_banner();

    let mut notes = load_notes()?;

    match cli.command {
        Commands::Add { title, body } => {
            let body_text = if body.is_empty() {
                prompt_multiline("Enter note body. Finish with an empty line:")?
            } else {
                body.join(" ")
            };
            let id = notes.last().map(|n| n.id + 1).unwrap_or(1);
            let note = Note {
                id,
                title,
                body: body_text,
                created: Local::now(),
            };
            notes.push(note);
            save_notes(&notes)?;
            println!("{}", "✅ Note added!".green().bold());
        }
        Commands::List { verbose } => {
            if notes.is_empty() {
                println!(
                    "{}",
                    "No notes yet. Add one with `notectl add <title>`!".yellow()
                )
            } else {
                for note in notes {
                    println!(
                        "{} {} · {}",
                        format!("[#{}]", note.id).cyan().bold(),
                        note.title.bold(),
                        note.created.format("%Y-%m-%d %H:%M").dimmed()
                    );
                    if verbose {
                        println!("  {}", note.body);
                    }
                }
            }
        }
        Commands::View { id } => {
            if let Some(note) = notes.iter().find(|n| n.id == id) {
                println!(
                    "{}\n{}\n{}",
                    note.title.bold().underline(),
                    "-".repeat(note.title.len()).green(),
                    note.body
                )
            } else {
                println!("{}", "Note not found".red());
            }
        }
        Commands::Delete { id } => {
            let original_len = notes.len();
            notes.retain(|n| n.id != id);
            if notes.len() < original_len {
                save_notes(&notes)?;
                println!("{}", "🗑️ Note deleted".red().bold());
            } else {
                println!("{}", "Note not found".red());
            }
        }
        Commands::Search { query } => {
            let query_lower = query.to_lowercase();
            let results: Vec<_> = notes
                .iter()
                .filter(|n| {
                    n.title.to_lowercase().contains(&query_lower)
                        || n.body.to_lowercase().contains(&query_lower)
                })
                .collect();
            if results.is_empty() {
                println!("{}", "No matches 😯".yellow());
            } else {
                for note in results {
                    println!(
                        "{} {}",
                        format!("[#{}]", note.id).cyan().bold(),
                        note.title.bold()
                    );
                }
            }
        }
    }

    Ok(())
}

fn prompt_multiline(prompt: &str) -> Result<String> {
    println!("{}", prompt.blue().bold());
    let mut lines = Vec::new();

    loop {
        let mut buffer = String::new();
        io::stdin().read_line(&mut buffer)?;
        let trimmed = buffer.trim_end();
        if trimmed.is_empty() {
            break;
        }
        lines.push(trimmed.to_owned());
    }
    Ok(lines.join("\n"))
}

fn print_banner() {
    use figlet_rs::FIGfont;
    let font = FIGfont::standard().unwrap();
    let figure = font.convert("Notectl").unwrap();
    println!("{}", figure.to_string().bright_magenta());
}

fn load_notes() -> Result<Vec<Note>> {
    let path = get_db_path()?;
    if !path.exists() {
        return Ok(vec![]);
    }
    let data = fs::read_to_string(path)?;
    let notes: Vec<Note> = serde_json::from_str(&data)?;
    Ok(notes)
}

fn save_notes(notes: &[Note]) -> Result<()> {
    let path = get_db_path()?;
    let data = serde_json::to_string_pretty(notes)?;
    fs::write(path, data)?;
    Ok(())
}

fn get_db_path() -> Result<PathBuf> {
    let proj = ProjectDirs::from("", "", "notectl")
        .ok_or_else(|| eyre!("cannot determine data directory"))?;
    let path = proj.data_dir().join("notes.json");
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    Ok(path)
}
