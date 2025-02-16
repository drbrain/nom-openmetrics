use std::env;

use nom_openmetrics::parser::prometheus;

fn main() {
    let file = env::args()
        .nth(1)
        .expect("missing prometheus file to parse");

    let input = std::fs::read(file).expect("unable to read file");
    let input = String::from_utf8(input).expect("invalid UTF-8");

    let result = prometheus(&input);

    match result {
        Err(e) => {
            eprintln!("Failed to parse: {e:?}");
        }
        Ok((remaining, output)) => {
            eprintln!("Parsing complete");

            if !remaining.is_empty() {
                eprintln!();
                eprintln!("Remaining text: {remaining}");
            }

            eprintln!("Output:");

            println!("{output:#?}");
        }
    }
}
