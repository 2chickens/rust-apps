use clap::Parser;
use color_eyre::{eyre::Result, eyre::WrapErr};
use colored::*;
use rand::Rng;
use serde::{Deserialize, Serialize};
use std::{
    fs,
    io::{self, Write},
    path::Path,
};

const STRENGTH_PER_DAY: u32 = 100;
const CHEST_COST: u32 = 50;

#[derive(Parser, Debug)]
#[command(
    name = "rock_treasure_hunter",
    version,
    about = "Bust rocks, earn coins, and discover treasures!"
)]
struct Cli {
    /// Your adventurer name
    #[arg(short, long, default_value = "Adventurer")]
    name: String,

    /// Load previous save if it exists
    #[arg(short, long)]
    load: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
enum Rarity {
    Common,
    Rare,
    Epic,
    Legendary,
}

impl Rarity {
    fn color(&self) -> Color {
        match self {
            Rarity::Common => Color::White,
            Rarity::Rare => Color::Cyan,
            Rarity::Epic => Color::Magenta,
            Rarity::Legendary => Color::Yellow,
        }
    }

    fn all() -> [Self; 4] {
        [Self::Common, Self::Rare, Self::Epic, Self::Legendary]
    }

    fn weight(&self) -> u8 {
        match self {
            Rarity::Common => 60,
            Rarity::Rare => 25,
            Rarity::Epic => 10,
            Rarity::Legendary => 5,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct Treasure {
    name: String,
    rarity: Rarity,
}

impl Treasure {
    fn display(&self) {
        println!(
            "{} {}",
            self.name.color(self.rarity.color()).bold(),
            format!("({:?})", self.rarity).color(self.rarity.color())
        );
    }
}

#[derive(Debug, Serialize, Deserialize)]
struct Player {
    name: String,
    strength: u32,
    coins: u32,
    collection: Vec<Treasure>,
}

impl Player {
    fn new(name: &str) -> Self {
        Self {
            name: name.to_string(),
            strength: STRENGTH_PER_DAY,
            coins: 0,
            collection: vec![],
        }
    }

    fn new_day(&mut self) {
        self.strength = STRENGTH_PER_DAY;
        println!(
            "\n{} It's a new day! Your strength is full ({}).",
            "☀️".yellow(),
            self.strength
        );
    }

    fn hit_rock(&mut self) {
        if self.strength == 0 {
            println!("{} You are out of strength for today!", "⚠️".yellow());
            return;
        }
        self.strength -= 1;
        let coins_found: u32 = rand::rng().random_range(0..=10);
        self.coins += coins_found;
        println!(
            "{} You swing your pickaxe... {} coins fly out! (+{})",
            rock_art().dimmed(),
            "💰".yellow(),
            coins_found
        );
    }

    fn open_chest(&mut self) {
        if self.coins < CHEST_COST {
            println!(
                "{} Not enough coins ({} needed). You have {}.",
                "🚫".red(),
                CHEST_COST,
                self.coins
            );
            return;
        }
        self.coins -= CHEST_COST;
        println!("{} Opening chest...", chest_art().yellow());
        let treasure = random_treasure();
        treasure.display();
        self.collection.push(treasure);
    }

    fn view_collection(&self) {
        if self.collection.is_empty() {
            println!("{} Your collection is empty!", "📭".dimmed());
            return;
        }
        println!("\n{} Treasure Collection:", "📜".bright_white().bold());
        for (i, t) in self.collection.iter().enumerate() {
            print!("{:3}. ", i + 1);
            t.display();
        }
    }
}

fn random_treasure() -> Treasure {
    let treasures = vec![
        ("Rusty Dagger", Rarity::Common),
        ("Old Boots", Rarity::Common),
        ("Silver Ring", Rarity::Rare),
        ("Emerald Amulet", Rarity::Rare),
        ("Phoenix Feather", Rarity::Epic),
        ("Dragon Scale", Rarity::Epic),
        ("Excalibur", Rarity::Legendary),
        ("Philosopher's Stone", Rarity::Legendary),
    ];

    let mut rng = rand::rng();
    let roll: u8 = rng.random_range(0..100);

    let rarity = {
        let mut cumulative = 0;
        let mut selected = Rarity::Common;
        for r in Rarity::all() {
            cumulative += r.weight();
            if roll < cumulative {
                selected = r;
                break;
            }
        }
        selected.clone()
    };

    let candidates: Vec<_> = treasures
        .into_iter()
        .filter(|(_, r)| *r == rarity)
        .collect();

    let (name, _rar) = &candidates[rng.random_range(0..candidates.len())];
    Treasure {
        name: name.to_string(),
        rarity,
    }
}

fn rock_art() -> &'static str {
    "🪨"
}

fn chest_art() -> &'static str {
    "📦"
}

fn prompt(input: &str) -> Result<String> {
    print!("{} ", input.green().bold());
    io::stdout().flush()?;
    let mut buf = String::new();
    io::stdin().read_line(&mut buf)?;
    Ok(buf.trim().to_string())
}

fn save_path(name: &str) -> String {
    format!("{}.json", name.to_lowercase())
}

fn load_player(path: &str) -> Option<Player> {
    if Path::new(path).exists() {
        let data = fs::read_to_string(path).ok()?;
        serde_json::from_str(&data).ok()
    } else {
        None
    }
}

fn save_player(player: &Player, path: &str) -> Result<()> {
    let json = serde_json::to_string_pretty(player)?;
    fs::write(path, json).wrap_err("Failed to save game")?;
    Ok(())
}

fn main() -> Result<()> {
    color_backtrace::install();
    let cli = Cli::parse();

    let save_file = save_path(&cli.name);
    let mut player = if cli.load {
        load_player(&save_file).unwrap_or_else(|| Player::new(&cli.name))
    } else {
        Player::new(&cli.name)
    };

    println!("{} Welcome, {}!", "✨".bright_yellow(), player.name.bold());
    println!("Type the number of an action and press Enter.\n");

    let mut day = 1;
    loop {
        println!(
            "\n{} Day {} | Strength: {} | Coins: {}",
            "🗓️".cyan(),
            day,
            player.strength.to_string().blue(),
            player.coins.to_string().yellow()
        );
        println!(
            "1️⃣  Hit Rock\n2️⃣  Open Chest (cost {})\n3️⃣  View Collection\n4️⃣  End Day\n5️⃣  Save & Quit",
            CHEST_COST
        );

        match prompt("Your choice?")?.as_str() {
            "1" => player.hit_rock(),
            "2" => player.open_chest(),
            "3" => player.view_collection(),
            "4" => {
                day += 1;
                player.new_day();
            }
            "5" => {
                save_player(&player, &save_file)?;
                println!("{} Game saved. Goodbye!", "💾".green());
                break;
            }
            _ => println!("{} Invalid choice!", "❓".red()),
        }
    }

    Ok(())
}
