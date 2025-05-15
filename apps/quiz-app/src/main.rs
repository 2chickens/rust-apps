use clap::{ColorChoice, Parser, Subcommand, arg, command};
use colored::*;
use std::io::{self, Write};
use std::time::Instant;

#[derive(Clone)]
struct Question {
    text: &'static str,
    options: [&'static str; 4],
    correct: usize,
}

#[derive(Clone)]
struct Quiz {
    name: &'static str,
    title: &'static str,
    questions: Vec<Question>,
    pass_mark: f32,
}

fn quizzes() -> Vec<Quiz> {
    vec![
        Quiz {
            name: "general",
            title: "🌍  General Knowledge",
            pass_mark: 0.7,
            questions: vec![
                Question {
                    text: "Which planet is known as the Red Planet?",
                    options: ["Earth", "Mars", "Jupiter", "Venus"],
                    correct: 2,
                },
                Question {
                    text: "Who wrote the play 'Romeo and Juliet'?",
                    options: [
                        "William Shakespeare",
                        "Charles Dickens",
                        "Leo Tolstoy",
                        "Jane Austen",
                    ],
                    correct: 1,
                },
                Question {
                    text: "What is the capital city of Australia?",
                    options: ["Sydney", "Melbourne", "Canberra", "Brisbane"],
                    correct: 3,
                },
                Question {
                    text: "How many degrees are in a right angle?",
                    options: ["45", "90", "180", "360"],
                    correct: 2,
                },
                Question {
                    text: "Which element has the chemical symbol 'O'?",
                    options: ["Gold", "Oxygen", "Silver", "Iron"],
                    correct: 2,
                },
            ],
        },
        Quiz {
            name: "science",
            title: "🔬  Basic Science",
            pass_mark: 0.6,
            questions: vec![
                Question {
                    text: "What gas do plants absorb from the atmosphere?",
                    options: ["Oxygen", "Nitrogen", "Carbon Dioxide", "Hydrogen"],
                    correct: 3,
                },
                Question {
                    text: "What is H₂O more commonly known as?",
                    options: ["Salt", "Water", "Hydrogen Peroxide", "Ozone"],
                    correct: 2,
                },
                Question {
                    text: "How many planets are in our solar system?",
                    options: ["7", "8", "9", "10"],
                    correct: 2,
                },
                Question {
                    text: "At what temperature (°C) does water freeze?",
                    options: ["0", "32", "100", "‑273"],
                    correct: 1,
                },
            ],
        },
    ]
}

#[derive(Parser)]
#[command(
    name= "quiz-app",
    version,
    about = "Junkai Ji",
    about = "A terminal quiz application",
    color = ColorChoice::Always
)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand, Debug)]
enum Commands {
    /// List all bundled quizzes
    List,
    /// Take a quiz by name (see `list`)
    Take {
        /// /// The quiz's short name (e.g. "general")")
        #[arg(value_parser = clap::builder::NonEmptyStringValueParser::new())]
        name: String,
    },
}

fn main() {
    colored::control::set_override(true);

    let cli = Cli::parse();
    let quizzes = quizzes();

    match cli.command {
        Commands::List => {
            println!("{}\n", "Available Quizzzes:".bold().underline());
            for q in &quizzes {
                println!(" • {} {}", q.name.bold().bright_green(), q.title);
            }
        }

        Commands::Take { name } => match quizzes.iter().find(|q| q.name == name) {
            Some(quiz) => run_quiz(quiz),
            None => {
                eprintln!("{} {}", "unknown quiz:".bright_red(), name)
            }
        },
    }
}

fn run_quiz(quiz: &Quiz) {
    println!(
        "\n{} {}\n",
        "▶️  Starting quiz:".bold().bright_cyan(),
        quiz.title.bold()
    );

    let start = Instant::now();
    let mut correct: usize = 0;

    for (i, q) in quiz.questions.iter().enumerate() {
        println!(
            "{} {}",
            format!("Q{}: ", i + 1).bright_magenta().bold(),
            q.text.bold()
        );
        for (opt_i, opt) in q.options.iter().enumerate() {
            println!("  {} {}", format!("{}.", opt_i + 1).bright_yellow(), opt);
        }

        loop {
            print!("{}", "Your answer (1 - 4): ".bright_blue().bold());
            io::stdout().flush().unwrap();
            let mut input = String::new();
            io::stdin().read_line(&mut input).unwrap();

            match input.trim().parse::<usize>() {
                Ok(num @ 1..=4) => {
                    if num == q.correct {
                        println!("{}\n", "✓ Correct!\n".bright_green().bold());
                        correct += 1;
                    } else {
                        println!(
                            "{} {}\n",
                            "✗ Wrong!".bright_red().bold(),
                            format!("(correct: {})", q.correct).dimmed()
                        )
                    }
                    break;
                }
                _ => {
                    println!("{}", "Please type a number between 1 and 4.".bright_red());
                }
            }
        }
    }

    let elapsed = start.elapsed();
    let total = quiz.questions.len();
    let pct = correct as f32 / total as f32;
    let passed = pct >= quiz.pass_mark;

    println!(
        "{}\n├── {} {}/{} ({:.0}%)\n└── {} {}s\n",
        "📊  Results".bold().underline(),
        "Score:".bold(),
        correct,
        total,
        pct * 100.0,
        "Time:".bold(),
        elapsed.as_secs()
    );

    if passed {
        println!("{}", "🎉  You passed!".bold().bright_green());
    } else {
        println!(
            "{}",
            "😞  You did not pass. Better luck next time!"
                .bold()
                .bright_red()
        );
    }
}
