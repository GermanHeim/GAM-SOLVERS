mod constants;
mod fetch;
mod parse;
mod utils;

use constants::*;
use fetch::*;

#[tokio::main]
async fn main() {
    let mut all_data: Vec<(&str, Vec<Data>)> = Vec::new();

    for i in SOLVERS {
        let link = format!("{}S_{}.html", BASE_URL, i,);
        println!("Fetching: {}", link);
        match scrape_gams_solvers(&link).await {
            Ok(data) => {
                println!("  -> {} options", data.len());
                all_data.push((i, data));
            }
            Err(e) => {
                eprintln!("  -> Error: {}", e);
            }
        }
    }

    let generated = parse::generate_all_rs(&all_data);
    let path = "src/generated.rs";
    std::fs::write(path, &generated).expect("Failed to write generated file");
    println!("\nWrote {} structs to {}", all_data.len(), path);
}
