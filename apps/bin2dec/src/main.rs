use clap::Parser;

#[derive(Parser, Debug)]
#[command(name="Bin2Dec", author="Junkai Ji", about="Convert binary numbers (up to 8 digits) to decimal format.", long_about = None)]
struct Args {
    #[arg(
        value_name = "BINARY",
        help = "Binary number to convert (max 8 digits)"
    )]
    binary: String,
}

fn main() {
    let args = Args::parse();

    match bin2dec(&args.binary) {
        Ok(value) => println!("Decimal output: {}", value),
        Err(e) => eprintln!("Error: {}", e),
    }
}

fn bin2dec(input: &str) -> Result<u32, String> {
    if input.len() > 8 {
        return Err("input must be no more than 8 digits".into());
    }

    if !input.chars().all(|c| c == '0' || c == '1') {
        return Err("input must only contain 0 and 1.".into());
    }

    input
        .chars()
        .rev()
        .enumerate()
        .map(|(i, c)| {
            let digit = c.to_digit(10).unwrap();
            Ok(digit * 2u32.pow(i as u32))
        })
        .try_fold(0u32, |acc, x| x.map(|val| acc + val))
}

