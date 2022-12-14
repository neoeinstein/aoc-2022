use std::{env, fmt, fs, io, io::Read};

use color_eyre::Result;
use itertools::Itertools;

fn main() -> Result<()> {
    color_eyre::install()?;

    let input = if let Some(path) = env::args_os().nth(1) {
        fs::read_to_string(path)?
    } else {
        let mut input = String::new();
        io::stdin().read_to_string(&mut input)?;
        input
    };

    let result = run(&input)?;

    println!("Part 1: \n{:?}\n\nPart 2: \n{:?}", result.0, result.1);

    Ok(())
}

fn run(input: &str) -> Result<(usize, usize)> {
    let mut max_y = 0;
    let lines = input
        .lines()
        .map(|line| {
            line.split(" -> ")
                .map(|vertex| {
                    let (x, y) = vertex.split_once(',').unwrap();
                    let x = x.parse::<usize>().unwrap();
                    let y = y.parse::<usize>().unwrap();
                    max_y = max_y.max(y);
                    (x, y)
                })
                .collect_vec()
        })
        .collect_vec();

    let mut sand_pit = SandPit::with_depth(max_y);

    for line in lines {
        for (prior, next) in line.into_iter().tuple_windows() {
            let x_range = prior.0.min(next.0)..=prior.0.max(next.0);
            let y_range = prior.1.min(next.1)..=prior.1.max(next.1);
            for (x, y) in x_range.cartesian_product(y_range) {
                sand_pit.add_rock(x, y);
            }
        }
    }

    let mut count = 0;
    while sand_pit.drop_sand() {
        count += 1;
    }

    print!("{sand_pit}");

    sand_pit.add_floor();

    let mut count_2 = count;
    while !sand_pit.safe_to_stand() {
        sand_pit.drop_sand();
        count_2 += 1;
    }

    print!("{sand_pit}");

    Ok((count, count_2))
}

#[derive(Debug, Clone, Default)]
struct SandPit {
    cells: Vec<[Cell; 1000]>,
    min_x: usize,
    max_x: usize,
}

impl SandPit {
    fn with_depth(max_y: usize) -> Self {
        Self {
            cells: vec![[Cell::Empty; 1000]; max_y + 1],
            max_x: 500,
            min_x: 500,
        }
    }

    fn add_rock(&mut self, x: usize, y: usize) {
        if y >= self.cells.len() {
            self.cells.resize_with(y, || [Cell::Empty; 1000]);
        }
        self.cells[y][x] = Cell::Rock;
        self.min_x = self.min_x.min(x);
        self.max_x = self.max_x.max(x);
    }

    fn add_floor(&mut self) {
        self.cells.push([Cell::Empty; 1000]);
        self.cells.push([Cell::Rock; 1000]);
    }

    fn is_in_bounds(&self, y: usize) -> bool {
        y < self.cells.len() - 1
    }

    fn drop_sand(&mut self) -> bool {
        let mut position = (500, 0);
        while let Some((x, y)) = self.next_position(position) {
            if !self.is_in_bounds(y) {
                return false;
            }
            position = (x, y);
        }
        self.cells[position.1][position.0] = Cell::Sand;
        self.max_x = self.max_x.max(position.0);
        self.min_x = self.min_x.min(position.0);
        true
    }

    fn next_position(&self, (x, y): (usize, usize)) -> Option<(usize, usize)> {
        let options = &self.cells.get(y + 1)?[x - 1..=x + 1];
        if options[1].is_empty() {
            Some((x, y + 1))
        } else if options[0].is_empty() {
            Some((x - 1, y + 1))
        } else if options[2].is_empty() {
            Some((x + 1, y + 1))
        } else {
            None
        }
    }

    fn safe_to_stand(&self) -> bool {
        !self.cells[0][500].is_empty()
    }
}

impl fmt::Display for SandPit {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let range = (self.min_x - 1)..=(self.max_x + 1);
        for i in range.clone() {
            write!(f, "{}", if i == 500 { '*' } else { ' ' })?;
        }
        writeln!(f)?;
        for row in &self.cells {
            for cell in &row[range.clone()] {
                write!(
                    f,
                    "{}",
                    match cell {
                        Cell::Empty => '.',
                        Cell::Sand => 'o',
                        Cell::Rock => '#',
                    }
                )?;
            }
            writeln!(f)?;
        }
        Ok(())
    }
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
enum Cell {
    #[default]
    Empty,
    Sand,
    Rock,
}

impl Cell {
    fn is_empty(&self) -> bool {
        matches!(self, Cell::Empty)
    }
}

#[cfg(test)]
mod tests {
    use test_case::test_case;

    use super::*;

    #[test_case(include_str!("../../input/day14test") => matches Ok((24, 93)))]
    fn default_tests(input: &str) -> Result<(usize, usize)> {
        run(input)
    }
}
