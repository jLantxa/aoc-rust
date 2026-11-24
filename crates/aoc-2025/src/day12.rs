use std::collections::{BTreeSet, HashSet};

use anyhow::{Context, Result};

pub(crate) fn part1(input: &str) -> Result<String> {
    let (shapes, problems) = parse_input(input)?;

    Ok(problems
        .iter()
        .filter(|problem| Packer::new(problem, &shapes).can_fit())
        .count()
        .to_string())
}

pub(crate) fn part2(_input: &str) -> Result<String> {
    Ok("N/A".to_string())
}

/// A gift shape, with every distinct rotation/reflection pre-normalised so
/// that the top-left cell of its bounding box is (0, 0).
#[derive(Debug, Clone)]
struct Shape {
    orientations: Vec<Vec<(usize, usize)>>,
    area: usize,
}

impl Shape {
    fn new(cells: &[(i32, i32)]) -> Self {
        let mut variants = BTreeSet::new();

        for mirrored in [false, true] {
            let mut current: Vec<(i32, i32)> = cells
                .iter()
                .map(|&(x, y)| if mirrored { (-x, y) } else { (x, y) })
                .collect();

            for _ in 0..4 {
                let min_x = current.iter().map(|&(x, _)| x).min().unwrap_or(0);
                let min_y = current.iter().map(|&(_, y)| y).min().unwrap_or(0);
                let mut normalised: Vec<(usize, usize)> = current
                    .iter()
                    .map(|&(x, y)| ((x - min_x) as usize, (y - min_y) as usize))
                    .collect();
                normalised.sort_unstable();
                variants.insert(normalised);

                current = current.iter().map(|&(x, y)| (-y, x)).collect();
            }
        }

        Self {
            orientations: variants.into_iter().collect(),
            area: cells.len(),
        }
    }
}

#[derive(Debug)]
struct Problem {
    width: usize,
    height: usize,
    counts: Vec<usize>,
}

/// Backtracking search that tries to place every gift in a region.
struct Packer<'a> {
    problem: &'a Problem,
    shapes: &'a [Shape],
    remaining: Vec<usize>,
    occupied: Vec<u64>,
    /// Memoisation prevents the ambiguous (small) regions from exploding into
    /// permutations of identical gifts. Large regions have enough slack to be
    /// solved by the first few placements, so they don't need it.
    seen: HashSet<(Vec<u64>, Vec<usize>)>,
}

impl<'a> Packer<'a> {
    fn new(problem: &'a Problem, shapes: &'a [Shape]) -> Self {
        let mut remaining = vec![0; shapes.len()];
        for (slot, &count) in remaining.iter_mut().zip(&problem.counts) {
            *slot = count;
        }

        Self {
            problem,
            shapes,
            remaining,
            occupied: vec![0u64; problem.width * problem.height / 64 + 1],
            seen: HashSet::new(),
        }
    }

    fn can_fit(&mut self) -> bool {
        let required_area: usize = self
            .remaining
            .iter()
            .zip(self.shapes)
            .map(|(&count, shape)| count * shape.area)
            .sum();
        if required_area > self.problem.width * self.problem.height {
            return false;
        }

        // Every shape fits inside its own 3x3 box, so if there are enough
        // grid-aligned 3x3 cells for all the gifts, they trivially fit.
        let gift_count: usize = self.remaining.iter().sum();
        if gift_count <= (self.problem.width / 3) * (self.problem.height / 3) {
            return true;
        }

        self.search()
    }

    fn search(&mut self) -> bool {
        if self.remaining.iter().all(|&count| count == 0) {
            return true;
        }

        if self.problem.width * self.problem.height <= 256
            && !self
                .seen
                .insert((self.occupied.to_vec(), self.remaining.to_vec()))
        {
            return false;
        }

        // Most constrained first: fewer orientations means fewer placements.
        let Some(shape_index) = self
            .remaining
            .iter()
            .enumerate()
            .filter(|&(_, &count)| count > 0)
            .min_by_key(|&(index, _)| self.shapes[index].orientations.len())
            .map(|(index, _)| index)
        else {
            return true;
        };

        for orientation in 0..self.shapes[shape_index].orientations.len() {
            // Copy the cells out so the search can borrow `self` mutably.
            let mut cells = [(0usize, 0usize); 9];
            let source = &self.shapes[shape_index].orientations[orientation];
            cells[..source.len()].copy_from_slice(source);

            let box_width = cells[..source.len()]
                .iter()
                .map(|&(x, _)| x)
                .max()
                .unwrap_or(0)
                + 1;
            let box_height = cells[..source.len()]
                .iter()
                .map(|&(_, y)| y)
                .max()
                .unwrap_or(0)
                + 1;

            for top in 0..=self.problem.height - box_height {
                for left in 0..=self.problem.width - box_width {
                    if self.try_place(shape_index, &cells[..source.len()], left, top) {
                        return true;
                    }
                }
            }
        }

        false
    }

    fn try_place(
        &mut self,
        shape_index: usize,
        cells: &[(usize, usize)],
        left: usize,
        top: usize,
    ) -> bool {
        let mut bits = [(0usize, 0u64); 9];
        for (slot, &(dx, dy)) in bits.iter_mut().zip(cells) {
            let index = (top + dy) * self.problem.width + left + dx;
            *slot = (index / 64, 1u64 << (index % 64));
        }

        for &(word, bit) in bits.iter().take(cells.len()) {
            if self.occupied[word] & bit != 0 {
                return false;
            }
        }
        for &(word, bit) in bits.iter().take(cells.len()) {
            self.occupied[word] |= bit;
        }
        self.remaining[shape_index] -= 1;

        if self.search() {
            return true;
        }

        self.remaining[shape_index] += 1;
        for &(word, bit) in bits.iter().take(cells.len()) {
            self.occupied[word] &= !bit;
        }
        false
    }
}

fn parse_input(input: &str) -> Result<(Vec<Shape>, Vec<Problem>)> {
    let mut shapes = Vec::new();
    let mut problems = Vec::new();

    for block in input.split("\n\n").filter(|block| !block.trim().is_empty()) {
        let mut lines = block.lines();
        let header = lines.next().context("Empty input block")?.trim();

        if let Some(index) = header.strip_suffix(':') {
            let index: usize = index
                .trim()
                .parse()
                .with_context(|| format!("Invalid shape index {header:?}"))?;
            let cells: Vec<(i32, i32)> = lines
                .enumerate()
                .flat_map(|(y, line)| {
                    line.chars()
                        .enumerate()
                        .filter_map(move |(x, cell)| (cell == '#').then_some((x as i32, y as i32)))
                })
                .collect();
            let shape = Shape::new(&cells);
            if shapes.len() <= index {
                shapes.resize_with(index + 1, || Shape {
                    orientations: Vec::new(),
                    area: 0,
                });
            }
            shapes[index] = shape;
        } else {
            for line in std::iter::once(header).chain(lines) {
                let (size, counts) = line
                    .split_once(':')
                    .with_context(|| format!("Invalid region line {line:?}"))?;
                let (width, height) = size
                    .split_once('x')
                    .with_context(|| format!("Invalid region size {size:?}"))?;
                problems.push(Problem {
                    width: width.trim().parse()?,
                    height: height.trim().parse()?,
                    counts: counts
                        .split_whitespace()
                        .map(|count| count.parse().context("Invalid gift count"))
                        .collect::<Result<_>>()?,
                });
            }
        }
    }

    Ok((shapes, problems))
}

#[cfg(test)]
mod tests {
    use super::*;

    const INPUT: &str = "\
0:
###
##.
##.

1:
###
##.
.##

2:
.##
###
##.

3:
##.
###
##.

4:
###
#..
###

5:
###
.#.
###

4x4: 0 0 0 0 2 0
12x5: 1 0 1 0 2 2
12x5: 1 0 1 0 3 2";

    #[test]
    fn test_part1() -> Result<()> {
        assert_eq!(part1(INPUT)?, 2.to_string());
        Ok(())
    }

    #[test]
    fn test_orientations_are_distinct() {
        let (shapes, _) = parse_input(INPUT).unwrap();
        assert_eq!(shapes[0].area, 7);
        assert_eq!(shapes[0].orientations.len(), 8);
        assert_eq!(shapes[2].orientations.len(), 2);
    }
}
