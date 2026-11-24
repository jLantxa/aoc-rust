# AoC Rust

Advent of Code solutions organised as a Cargo workspace. Each year is a separate
crate, so a new year starts with a clean set of dependencies and never inherits
the previous years'.

## Layout

- `crates/aoc-common` — shared boilerplate: input loading, timing, result
  printing, and the `Grid` utility.
- `crates/aoc-<year>` — one crate per year, with only that year's dependencies
  and solutions (`src/dayNN.rs`).
- `src/main.rs` — the `aoc` CLI and the year registry.
- `utils/setup.rs` — the `setup` scaffolding tool.
- `input/<year>/dayNN.txt` — puzzle inputs (gitignored).

## Usage

```sh
cargo run --bin aoc -- --year 2025 --day 10 --part 1
cargo run --bin aoc -- --year 2022          # run every implemented day
cargo test                                 # tests in all crates
```

## Adding a new year or day

```sh
cargo run --bin setup -- --year 2026 --day 1
```

This creates the year crate if needed, the day stub, and an input file, and
registers both the day and the year with the runner. Any dependencies you need
for 2026 go in `crates/aoc-2026/Cargo.toml`; no other crate is affected.
