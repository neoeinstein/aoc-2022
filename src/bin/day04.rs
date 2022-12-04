use std::{io, ops::RangeInclusive};

use color_eyre::Result;
use itertools::Itertools;
use nom::{
    bytes::complete::tag,
    character::complete::digit1,
    combinator::{all_consuming, map, map_res},
    sequence::separated_pair,
    Finish, IResult,
};

fn main() -> Result<()> {
    let (contains_count, overlap_count) = run(io::stdin().lines())?;

    println!("contains count: {}", contains_count);
    println!("overlap count: {}", overlap_count);

    Ok(())
}

fn run<I: IntoIterator<Item = io::Result<String>>>(lines: I) -> Result<(u32, u32)> {
    let sums = lines
        .into_iter()
        .map(|line| -> Result<(bool, bool)> {
            let line = line?;
            let pair = parse_elf_pair(&line)?;
            Ok((pair.is_one_subset_of_other(), pair.overlaps()))
        })
        .fold_ok((0, 0), |acc, b| {
            (
                if b.0 { acc.0 + 1 } else { acc.0 },
                if b.1 { acc.1 + 1 } else { acc.1 },
            )
        })?;

    Ok(sums)
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct ElfPair(RangeInclusive<u8>, RangeInclusive<u8>);

impl ElfPair {
    fn overlaps(&self) -> bool {
        self.0.end() >= self.1.start() && self.0.start() <= self.1.end()
    }

    fn is_one_subset_of_other(&self) -> bool {
        is_superset(&self.0, &self.1) || is_superset(&self.1, &self.0)
    }
}

fn is_superset(r1: &RangeInclusive<u8>, r2: &RangeInclusive<u8>) -> bool {
    r1.start() <= r2.start() && r1.end() >= r2.end()
}

fn parse_elf_pair(s: &str) -> Result<ElfPair> {
    Ok(all_consuming(elf_pair)(s)
        .map_err(|e| e.to_owned())
        .finish()?
        .1)
}

fn elf_pair(s: &str) -> IResult<&str, ElfPair> {
    map(separated_pair(range, tag(","), range), |(e1, e2)| {
        ElfPair(e1, e2)
    })(s)
}

fn range(s: &str) -> IResult<&str, RangeInclusive<u8>> {
    map(separated_pair(number, tag("-"), number), |(start, end)| {
        start..=end
    })(s)
}

fn number(s: &str) -> IResult<&str, u8> {
    map_res(digit1, str::parse)(s)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_elf_pair() {
        assert_eq!(elf_pair("2-4,6-8"), Ok(("", ElfPair(2..=4, 6..=8))));
    }

    #[test]
    fn test_contains() {
        assert!(is_superset(&(1..=5), &(2..=4)));
        assert!(is_superset(&(1..=5), &(1..=4)));
        assert!(is_superset(&(1..=5), &(2..=5)));
        assert!(is_superset(&(1..=5), &(1..=5)));
        assert!(!is_superset(&(1..=5), &(0..=5)));
        assert!(!is_superset(&(1..=5), &(1..=6)));
    }

    #[test]
    fn test_is_one_superset_of_other() {
        assert!(ElfPair(1..=5, 2..=4).is_one_subset_of_other());
        assert!(ElfPair(1..=5, 1..=4).is_one_subset_of_other());
        assert!(ElfPair(1..=5, 2..=5).is_one_subset_of_other());
        assert!(ElfPair(1..=5, 1..=5).is_one_subset_of_other());
        assert!(ElfPair(1..=5, 0..=5).is_one_subset_of_other());
        assert!(ElfPair(1..=5, 1..=6).is_one_subset_of_other());
        assert!(!ElfPair(2..=4, 1..=3).is_one_subset_of_other());
    }
}
