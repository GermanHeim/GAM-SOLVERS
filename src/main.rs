mod constants;
mod fetch;
mod parse;
mod utils;

use constants::*;
use fetch::*;
use parse::generate_solver_params;


#[tokio::main]
async fn main() {
    supported_solvers! {
        BARON => "BARON";
        GUROBI => "GUROBI";
        HIGHS => "HIGHS";
    }
    
    for &solver in SupportedSolver::ALL {

        let link = format!("{BASE_URL}S_{}.html", solver.url_name());
        eprintln!("Fetching: {link}");

        match scrape_gams_solvers(&link).await {
            Ok(data) => {
                eprintln!("  -> {} options", data.len());
                let params = generate_solver_params(&data);
                println!("// {:?}", solver);
                print!("{params}");
            }
            Err(e) => {
                eprintln!("  -> Error: {e}");
            }
        }
    }
}
