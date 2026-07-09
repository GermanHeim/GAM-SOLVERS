use crate::parse;

pub use crate::parse::Data;

pub async fn scrape_gams_solvers(url: &str) -> Result<Vec<Data>, reqwest::Error> {
    let response = reqwest::get(url).await?;
    let html = response.text().await?;
    Ok(parse::parse_solver_options(&html))
}
