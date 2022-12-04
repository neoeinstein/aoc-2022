use std::{fmt, io, iter};

use color_eyre::Result;
use itertools::Itertools;

fn main() -> Result<()> {
    let (total_priority, group_ids_sum) = run(io::stdin().lines())?;

    println!("total priority: {}", total_priority);
    println!("group ids sum: {}", group_ids_sum);

    Ok(())
}

fn run<I: IntoIterator<Item = io::Result<String>>>(lines: I) -> io::Result<(u32, u32)> {
    lines
        .into_iter()
        .tuples()
        .map(|(e1, e2, e3)| -> io::Result<_> {
            let owned_group = [e1?, e2?, e3?];
            let group = owned_group.iter().map(|s| s.as_str());

            let sack_priority_sum: u32 = group.clone().map(rucksack_priority).sum();

            let group_priority = intersect_contents(group.map(contents_set));

            Ok((sack_priority_sum, group_priority))
        })
        .fold_ok((0, 0), |acc, (sack_priority_sum, group_priority)| {
            (acc.0 + sack_priority_sum, acc.1 + group_priority.0 as u32)
        })
}

fn rucksack_priority(elf: &str) -> RucksackPriority {
    let mid = elf.len() / 2;
    let (first, last) = elf.split_at(mid);
    let first_set = contents_set(first);
    let last_set = contents_set(last);

    intersect_contents([first_set, last_set])
}

fn intersect_contents<I: IntoIterator<Item = ContentsSet>>(sacks: I) -> RucksackPriority {
    let intersection = sacks
        .into_iter()
        .fold(ContentsSet::FULL, ContentsSet::intersect);

    intersection.priority()
}

fn contents_set(s: &str) -> ContentsSet {
    let mut set = ContentsSet::EMPTY;
    for c in s.bytes() {
        set.insert(calc_priority(c));
    }
    set
}

fn calc_priority(c: u8) -> RucksackPriority {
    match c {
        b'a'..=b'z' => RucksackPriority(c - b'a' + 1),
        b'A'..=b'Z' => RucksackPriority(c - b'A' + 27),
        _ => unimplemented!(),
    }
}

#[derive(Clone, Copy, PartialEq, Eq)]
struct ContentsSet(u64);

impl ContentsSet {
    const EMPTY: Self = Self(0);
    const FULL: Self = Self(u64::MAX);

    fn insert(&mut self, priority: RucksackPriority) {
        self.0 |= 1 << (63 - priority.0);
    }

    fn intersect(self, other: Self) -> Self {
        Self(self.0 & other.0)
    }

    fn priority(self) -> RucksackPriority {
        RucksackPriority(self.0.leading_zeros() as u8)
    }
}

impl fmt::Debug for ContentsSet {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_tuple("ContentsSet")
            .field(&format_args!("{:064b}", self.0))
            .finish()
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct RucksackPriority(u8);

impl iter::Sum<RucksackPriority> for u32 {
    fn sum<I: Iterator<Item = RucksackPriority>>(iter: I) -> Self {
        iter.fold(0, |acc, p| acc + p.0 as u32)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_calc_priority() {
        assert_eq!(calc_priority(b'a'), RucksackPriority(1));
        assert_eq!(calc_priority(b'p'), RucksackPriority(16));
        assert_eq!(calc_priority(b'z'), RucksackPriority(26));
        assert_eq!(calc_priority(b'A'), RucksackPriority(27));
        assert_eq!(calc_priority(b'Z'), RucksackPriority(52));
    }

    #[test]
    fn test_find_rucksack_priority() {
        assert_eq!(
            rucksack_priority("vJrwpWtwJgWrhcsFMMfFFhFp"),
            RucksackPriority(16)
        );
        assert_eq!(
            rucksack_priority("jqHRNqRjqzjGDLGLrsFMfFZSrLrFZsSL"),
            RucksackPriority(38)
        );
        assert_eq!(
            rucksack_priority("PmmdzqPrVvPwwTWBwg"),
            RucksackPriority(42)
        );
        assert_eq!(
            rucksack_priority("wMqvLMZHhHMvwLHjbvcjnnSBnvTQFn"),
            RucksackPriority(22)
        );
        assert_eq!(rucksack_priority("ttgJtRGJQctTZtZT"), RucksackPriority(20));
        assert_eq!(
            rucksack_priority("CrZsJsPPZsGzwwsLwLmpwMDw"),
            RucksackPriority(19)
        );
    }

    #[test]
    fn test_contents_set() {
        assert_eq!(
            contents_set("a"),
            ContentsSet(0b0100000000000000000000000000000000000000000000000000000000000000)
        );
        assert_eq!(
            contents_set("aa"),
            ContentsSet(0b0100000000000000000000000000000000000000000000000000000000000000)
        );
        assert_eq!(
            contents_set("aaZ"),
            ContentsSet(0b0100000000000000000000000000000000000000000000000000100000000000)
        );
    }
}
