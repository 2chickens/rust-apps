// csvjson_cli.rs — a single‑file Rust CLI app for converting between CSV and JSON
// -----------------------------------------------------------------------------
// Usage examples:
//   csvjson to-json -i data.csv -o data.json --pretty
//   csvjson to-csv -i data.json -o data.csv
//   cat data.csv | csvjson to-json > out.json
//   cat data.json | csvjson to-csv > out.csv
//
// Run `csvjson to-json --help` or `csvjson to-csv --help` for CLI flags
// -----------------------------------------------------------------------------

use std::collections::BTreeSet;
use std::io::{self, Read};
use std::{fs, path::PathBuf};

use clap::{Parser, Subcommand};
use serde_json::Value;

#[derive(Parser)]
#[command(name = "csv2json", author = "Junkai Ji", version, about= "Convert CSV to JSON or JSON to CSV.", long_about =None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Convert CSV to JSON
    ToJson {
        #[arg(short, long)]
        input: Option<PathBuf>,

        #[arg(short, long)]
        output: Option<PathBuf>,

        #[arg(short, long)]
        pretty: bool,
    },

    /// Convert JSON to CSV
    ToCsv {
        #[arg(short, long)]
        input: Option<PathBuf>,

        #[arg(short, long)]
        output: Option<PathBuf>,
    },
}

fn main() {
    let cli = Cli::parse();

    match cli.command {
        Commands::ToJson {
            input,
            output,
            pretty,
        } => match read_input(input.as_ref()) {
            Ok(csv_text) => match csv_to_json(&csv_text, pretty) {
                Ok(json_text) => write_output(output.as_ref(), &json_text),
                Err(e) => exit_with_error(&e),
            },
            Err(e) => exit_with_error(&e),
        },
        Commands::ToCsv { input, output } => match read_input(input.as_ref()) {
            Ok(json_text) => match json_to_csv(&json_text) {
                Ok(csv_text) => write_output(output.as_ref(), &csv_text),
                Err(e) => exit_with_error(&e),
            },
            Err(e) => exit_with_error(&e),
        },
    }
}

fn read_input(path: Option<&PathBuf>) -> Result<String, String> {
    match path {
        Some(p) if p.as_os_str() != "-" => {
            fs::read_to_string(p).map_err(|e| format!("failed to read: '{}': {}", p.display(), e))
        }
        _ => {
            let mut buf = String::new();
            io::stdin()
                .read_to_string(&mut buf)
                .map_err(|e| format!("failed to read from STDIN: {}", e))?;
            if buf.trim().is_empty() {
                return Err("input is empty".into());
            }

            Ok(buf)
        }
    }
}

fn write_output(path: Option<&PathBuf>, data: &str) {
    if let Some(p) = path {
        if p.as_os_str() == "-" {
            println!("{data}");
        } else {
            match fs::write(p, data) {
                Ok(_) => println!("Wrote {} bytes to {}", data.len(), p.display()),
                Err(e) => exit_with_error(&format!("failed to write '{}': {}", p.display(), e)),
            }
        }
    } else {
        println!("{}", data);
    }
}

fn exit_with_error(msg: &str) {
    eprintln!("error: {}", msg);
    std::process::exit(1)
}

fn csv_to_json(csv_input: &str, pretty: bool) -> Result<String, String> {
    let mut lines = csv_input.lines().peekable();
    if lines.peek().is_none() {
        return Err("CSV input is empty".into());
    }

    let headers = parse_csv_line(lines.next().unwrap());

    if headers.is_empty() {
        return Err("CSV header row is empty".into());
    }

    let mut records = Vec::new();
    for (idx, line) in lines.enumerate() {
        if line.trim().is_empty() {
            continue;
        }
        let fields = parse_csv_line(line);
        if fields.len() != headers.len() {
            return Err(format!(
                "CSV row {} has {} fields but header has {}",
                idx + 2,
                fields.len(),
                headers.len(),
            ));
        }

        let mut map = serde_json::Map::with_capacity(headers.len());
        for (h, f) in headers.iter().zip(fields.iter()) {
            map.insert(h.clone(), guess_json_value(f));
        }
        records.push(Value::Object(map));
    }

    if pretty {
        serde_json::to_string_pretty(&records).map_err(|e| e.to_string())
    } else {
        serde_json::to_string(&records).map_err(|e| e.to_string())
    }
}

fn parse_csv_line(line: &str) -> Vec<String> {
    let mut fields = Vec::<String>::new();
    let mut current = String::new();
    let mut chars = line.chars().peekable();
    let mut in_quotes = false;

    while let Some(c) = chars.next() {
        match c {
            '"' => {
                if in_quotes {
                    if chars.peek() == Some(&'"') {
                        current.push('"');
                        chars.next();
                    } else {
                        in_quotes = false;
                    }
                } else {
                    in_quotes = true;
                }
            }
            ',' if !in_quotes => {
                fields.push(current.clone());
                current.clear();
            }
            _ => current.push(c),
        }
    }
    fields.push(current);
    fields
}

fn guess_json_value(s: &str) -> Value {
    let trimmed = s.trim();

    if trimmed.is_empty() {
        Value::Null
    } else if let Ok(i) = trimmed.parse::<i64>() {
        Value::from(i)
    } else if let Ok(f) = trimmed.parse::<f64>() {
        Value::from(f)
    } else if trimmed.eq_ignore_ascii_case("true") || trimmed.eq_ignore_ascii_case("false") {
        Value::from(trimmed.eq_ignore_ascii_case("true"))
    } else {
        Value::from(trimmed)
    }
}

fn json_to_csv(json_input: &str) -> Result<String, String> {
    let val: Value =
        serde_json::from_str(json_input).map_err(|e| format!("invalid JSON: {}", e))?;

    let arr = match val {
        Value::Array(a) => a,
        _ => return Err("JSON root must be an array of objects".into()),
    };

    if arr.is_empty() {
        return Err("JSON array is empty".into());
    }

    let mut keys = BTreeSet::new();

    for item in arr.iter() {
        match item {
            Value::Object(map) => {
                for k in map.keys() {
                    keys.insert(k.clone());
                }
            }
            _ => return Err("JSON array elements must be objects".into()),
        }
    }

    let keys: Vec<String> = keys.into_iter().collect();

    let mut out = String::new();
    out.push_str(&keys.join(","));
    out.push('\n');

    for obj in arr {
        if let Value::Object(map) = obj {
            let mut row: Vec<String> = Vec::with_capacity(keys.len());
            for k in &keys {
                let value = map.get(k).unwrap_or(&Value::Null);
                row.push(escape_csv_field(&json_value_to_string(value)));
            }
            out.push_str(&row.join(","));
            out.push('\n');
        }
    }

    Ok(out)
}

fn json_value_to_string(v: &Value) -> String {
    match v {
        Value::Null => String::new(),
        Value::String(s) => s.clone(),
        Value::Bool(b) => b.to_string(),
        Value::Number(n) => n.to_string(),
        _ => v.to_string(),
    }
}

fn escape_csv_field(s: &str) -> String {
    if s.contains([',', '"', '\n']) {
        let escaped = s.replace('"', "\"\"");
        format!("\"{}\"", escaped)
    } else {
        s.to_string()
    }
}
