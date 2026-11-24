use std::path::{Path, PathBuf};

use anyhow::{Context, Result, bail};
use clap::Parser;

const DAY_TEMPLATE: &str = r#"use anyhow::{Result, bail};

pub(crate) fn part1(input: &str) -> Result<String> {
    bail!("Unimplemented");
}

pub(crate) fn part2(input: &str) -> Result<String> {
    bail!("Unimplemented");
}

#[cfg(test)]
mod tests {
    use super::*;
}
"#;

#[derive(Parser, Debug)]
#[clap(author, version, about)]
struct Args {
    #[clap(short, long)]
    year: String,

    #[clap(short, long)]
    day: u8,
}

fn write_if_missing(path: &Path, contents: &str) -> Result<bool> {
    if path.exists() {
        return Ok(false);
    }

    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)
            .with_context(|| format!("Could not create directory {}", parent.display()))?;
    }

    std::fs::write(path, contents)
        .with_context(|| format!("Could not create {}", path.display()))?;
    println!("Created {}", path.display());
    Ok(true)
}

fn year_manifest(year: &str) -> String {
    format!(
        "[package]\n\
         name = \"aoc-{year}\"\n\
         version = \"0.1.0\"\n\
         edition = \"2024\"\n\
         \n\
         [dependencies]\n\
         aoc-common = {{ path = \"../aoc-common\" }}\n\
         anyhow = \"1.0.100\"\n"
    )
}

fn year_lib(day: u8) -> String {
    let module = format!("day{day:02}");
    format!(
        "use aoc_common::aoc::DailySolutions;\n\
         \n\
         mod {module};\n\
         \n\
         pub fn get_solution(day: u8) -> Option<DailySolutions> {{\n\
         \x20   match day {{\n\
         \x20       {day} => Some(({module}::part1, {module}::part2)),\n\
         \x20       _ => None,\n\
         \x20   }}\n\
         }}\n"
    )
}

fn insert_before(source: &mut String, marker: &str, text: &str) -> Result<()> {
    let pos = source
        .find(marker)
        .with_context(|| format!("Could not find {marker:?} in generated source"))?;
    source.insert_str(pos, text);
    Ok(())
}

fn register_day(lib_path: &Path, day: u8) -> Result<()> {
    let original = std::fs::read_to_string(lib_path)
        .with_context(|| format!("Could not read {}", lib_path.display()))?;
    let mut updated = original.clone();

    let module = format!("day{day:02}");
    if !updated.contains(&format!("mod {module};")) {
        insert_before(
            &mut updated,
            "pub fn get_solution",
            &format!("mod {module};\n\n"),
        )?;
    }

    let arm = format!("        {day} => Some(({module}::part1, {module}::part2)),\n");
    if !updated.contains(&arm) {
        insert_before(&mut updated, "        _ => None,", &arm)?;
    }

    if updated != original {
        std::fs::write(lib_path, updated)
            .with_context(|| format!("Could not update {}", lib_path.display()))?;
        println!("Registered {module} in {}", lib_path.display());
    }

    Ok(())
}

fn register_dependency(manifest_path: &Path, year: &str) -> Result<()> {
    let original = std::fs::read_to_string(manifest_path)
        .with_context(|| format!("Could not read {}", manifest_path.display()))?;
    let dependency = format!("aoc-{year} = {{ path = \"crates/aoc-{year}\" }}\n");

    if original.contains(&dependency) {
        return Ok(());
    }

    let mut updated = original.clone();
    let marker = "[dependencies]\n";
    let pos = updated.find(marker).with_context(|| {
        format!(
            "No {} section in {}",
            marker.trim(),
            manifest_path.display()
        )
    })? + marker.len();
    updated.insert_str(pos, &dependency);
    std::fs::write(manifest_path, updated)
        .with_context(|| format!("Could not update {}", manifest_path.display()))?;
    println!("Registered aoc-{year} in {}", manifest_path.display());

    Ok(())
}

fn register_year(main_path: &Path, year: &str) -> Result<()> {
    let original = std::fs::read_to_string(main_path)
        .with_context(|| format!("Could not read {}", main_path.display()))?;
    let entry = format!("    (\"{year}\", aoc_{year}::get_solution),\n");

    if original.contains(&entry) {
        return Ok(());
    }

    let mut updated = original.clone();
    insert_before(&mut updated, "];", &entry)?;
    std::fs::write(main_path, updated)
        .with_context(|| format!("Could not update {}", main_path.display()))?;
    println!("Registered {year} in {}", main_path.display());

    Ok(())
}

fn main() -> Result<()> {
    let args = Args::parse();

    if args.year.is_empty() || !args.year.chars().all(|c| c.is_ascii_digit()) {
        bail!("Year must be a number, got {:?}", args.year);
    }
    if !(1..=25).contains(&args.day) {
        bail!("Day must be between 1 and 25, got {}", args.day);
    }

    let crate_root = PathBuf::from("crates").join(format!("aoc-{}", args.year));
    let src_dir = crate_root.join("src");
    let lib_path = src_dir.join("lib.rs");
    let day_path = src_dir.join(format!("day{:02}.rs", args.day));
    let input_path = PathBuf::from("input")
        .join(&args.year)
        .join(format!("day{:02}.txt", args.day));

    write_if_missing(&crate_root.join("Cargo.toml"), &year_manifest(&args.year))?;
    let created_lib = write_if_missing(&lib_path, &year_lib(args.day))?;
    write_if_missing(&day_path, DAY_TEMPLATE)?;
    write_if_missing(&input_path, "")?;

    if !created_lib {
        register_day(&lib_path, args.day)?;
    }
    register_dependency(Path::new("Cargo.toml"), &args.year)?;
    register_year(Path::new("src/main.rs"), &args.year)?;

    println!("Ready to solve {} day {}!", args.year, args.day);
    Ok(())
}
