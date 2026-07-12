## GAMS Scraper

Scrapes solver option tables from the official GAMS docs (`gams.com/latest/docs/`) and
generates Rust code snippets for solver option structs in oximo.

### What it does

For each supported solver (BARON, GUROBI, HIGHS), it fetches the solver's options page,
parses the option name, description, and default value, and prints ready-to-paste
`(kind, method, Variant)` tuples to stdout — matching the format used by each solver's
`*_params!` macro (see `crates/gurobi/src/options.rs`).

### Usage

```bash
# Scrape all supported solvers
cargo run

# Scrape a single solver
cargo run -- GUROBI
```

Output goes to **stdout**; redirect it to save:

```bash
cargo run -- GUROBI > gurobi_tuples.txt
```

### Workflow

1. Run the scraper for a solver.
2. Review the output and manually merge new/changed entries into that solver's
   `*_params!` macro invocation.
3. `cargo build`/`cargo test` in oximo verifies the pasted variant names match the
   backend crate's real parameter enums.
4. Open a PR with the update.

