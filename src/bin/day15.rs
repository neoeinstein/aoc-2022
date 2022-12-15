use std::{cmp::Reverse, env, fs, io, io::Read, ops::RangeInclusive};

use color_eyre::Result;
use fxhash::FxHashSet;
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

    let result = run::<2_000_000>(&input)?;

    println!("Part 1: \n{:?}\n\nPart 2: \n{:?}", result.0, result.1);

    Ok(())
}

fn run<const ROW: i32>(input: &str) -> Result<(usize, i64)> {
    let regex = regex::Regex::new(r"(-?\d+)")?;
    let mut min_x = 0;
    let mut max_x = 0;
    let mut on_row = FxHashSet::default();
    let mut range_set = RangeSet::new((ROW * 2) as usize);
    let ranges = input
        .lines()
        .map(|line| {
            let mut iter = regex
                .captures_iter(line)
                .map(|cap| cap[1].parse::<i32>().unwrap());
            let sensor = Position::new(iter.next().unwrap(), iter.next().unwrap());
            let beacon = Position::new(iter.next().unwrap(), iter.next().unwrap());
            if beacon.y == ROW {
                on_row.insert(beacon);
            }

            let distance = sensor.manhattan_distance(&beacon) as i32;
            for y in sensor.y - distance..=sensor.y + distance {
                if y < 0 || y > ROW * 2 {
                    continue;
                }
                let remaining = distance - sensor.y.abs_diff(y) as i32;
                let lower = sensor.x - remaining;
                let upper = sensor.x + remaining;
                if lower > ROW * 2 || upper < 0 {
                    continue;
                }

                range_set.add(y, lower.max(0).min(ROW * 2)..=upper.max(0).min(ROW * 2));
            }

            // println!("After reading: {:?}", reading);
            // range_set.ranges.iter().enumerate().for_each(|(idx, ranges)| {
            //     print!("{}: ", idx);
            //     for range in ranges {
            //         print!("{:?} ", range);
            //     }
            //     println!();
            // });

            let vertical_difference = sensor.y.abs_diff(ROW) as i32;
            let remaining = distance - vertical_difference;
            let range = sensor.x - remaining..=sensor.x + remaining;
            max_x = max_x.max(*range.end());
            min_x = min_x.min(*range.start());
            range
        })
        .collect_vec();

    let mut count = 0;
    for i in dbg!(min_x..=max_x) {
        // let mut found = false;
        for range in &ranges {
            if range.contains(&i) {
                // found = true;
                count += 1;
                break;
            }
        }
        // print!("{}", if found { "#" } else { "." });
    }

    // range_set.ranges.iter().for_each(|ranges| {
    //     for range in ranges {
    //         print!("{:?} ", range);
    //     }
    //     println!();
    // });

    let position = range_set
        .ranges
        .into_iter()
        .enumerate()
        .filter_map(|(y, ranges)| {
            if ranges.len() == 2 {
                Some(Position::new(ranges[0].end() + 1, y as i32))
            } else {
                None
            }
        })
        .exactly_one()?;

    Ok((count - on_row.len(), position.value()))
}

struct RangeSet {
    ranges: Vec<Vec<RangeInclusive<i32>>>,
}

fn overlaps(l: &RangeInclusive<i32>, r: &RangeInclusive<i32>) -> bool {
    l.end() >= r.start() && l.start() <= r.end()
}

fn is_superset(r1: &RangeInclusive<i32>, r2: &RangeInclusive<i32>) -> bool {
    r1.start() <= r2.start() && r1.end() >= r2.end()
}

fn abutts(l: &RangeInclusive<i32>, r: &RangeInclusive<i32>) -> bool {
    l.end() + 1 == *r.start()
}

impl RangeSet {
    fn new(rows: usize) -> Self {
        Self {
            ranges: vec![Vec::new(); rows],
        }
    }

    fn add(&mut self, row: i32, range: RangeInclusive<i32>) {
        if row < 0 {
            return;
        }
        let Some(row) = self.ranges.get_mut(row as usize) else {
            return
        };

        row.push(range);
        row.sort_unstable_by_key(|r| (*r.start(), Reverse(*r.end())));

        // merge overlapping ranges
        let mut i = 0;
        while i < row.len() - 1 {
            if is_superset(&row[i], &row[i + 1]) {
                row.remove(i + 1);
            } else if overlaps(&row[i], &row[i + 1]) || abutts(&row[i], &row[i + 1]) {
                row[i] = *row[i].start()..=*row[i + 1].end();
                row.remove(i + 1);
            } else {
                i += 1;
            }
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
struct Position {
    x: i32,
    y: i32,
}

impl Position {
    fn new(x: i32, y: i32) -> Self {
        Self { x, y }
    }

    fn manhattan_distance(&self, other: &Self) -> u32 {
        self.x.abs_diff(other.x) + self.y.abs_diff(other.y)
    }

    fn value(&self) -> i64 {
        self.x as i64 * 4_000_000 + self.y as i64
    }
}

#[cfg(test)]
mod tests {
    use test_case::test_case;

    use super::*;

    #[test_case(include_str!("../../input/day15test") => matches Ok((26, 56000011)))]
    fn default_tests(input: &str) -> Result<(usize, i64)> {
        run::<10>(input)
    }
}
