use anyhow::{Context, Result};
use aoc_common::aoc::{self, SolutionProvider};
use clap::Parser;

/// Register each year crate here. This is the only runner code that needs to
/// change when adding a new year.
const YEARS: &[(&str, SolutionProvider)] = &[
    ("2022", aoc_2022::get_solution),
    ("2025", aoc_2025::get_solution),
];

#[derive(Parser, Debug)]
#[clap(author, version, about)]
struct Args {
    #[clap(short, long, help = "Year")]
    year: String,

    #[clap(short, long, help = "The day to run (1-25)")]
    day: Option<u8>,

    #[clap(short, long, help = "The part to run (1 or 2)")]
    part: Option<u8>,
}

fn main() -> Result<()> {
    let args = Args::parse();

    let get_solution = YEARS
        .iter()
        .find(|(year, _)| *year == args.year)
        .map(|(_, solver)| *solver)
        .with_context(|| {
            let available = YEARS
                .iter()
                .map(|(year, _)| *year)
                .collect::<Vec<_>>()
                .join(", ");
            format!("Unknown year {:?}. Available years: {available}", args.year)
        })?;

    if let Some(day) = args.day {
        aoc::run_single_solution(&args.year, day, args.part, get_solution)?;
    } else {
        aoc::run_all_yearly_solutions(&args.year, get_solution);
    }

    Ok(())
}
