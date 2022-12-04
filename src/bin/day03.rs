use std::io;

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
            (acc.0 + sack_priority_sum, acc.1 + group_priority)
        })
}

fn rucksack_priority(elf: &str) -> u32 {
    let mid = elf.len() / 2;
    let (first, last) = elf.split_at(mid);
    let first_set = contents_set(first);
    let last_set = contents_set(last);

    intersect_contents([first_set, last_set])
}

fn intersect_contents<I: IntoIterator<Item = u64>>(sacks: I) -> u32 {
    let intersection = sacks.into_iter().fold(u64::MAX, |acc, sack| acc & sack);

    intersection.leading_zeros()
}

fn contents_set(s: &str) -> u64 {
    let mut set = 0;
    for c in s.bytes() {
        set |= 1 << (63 - calc_priority(c));
    }
    set
}

fn calc_priority(c: u8) -> u8 {
    match c {
        b'a'..=b'z' => c - b'a' + 1,
        b'A'..=b'Z' => c - b'A' + 27,
        _ => unimplemented!(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_calc_priority() {
        assert_eq!(calc_priority(b'a'), 1);
        assert_eq!(calc_priority(b'p'), 16);
        assert_eq!(calc_priority(b'z'), 26);
        assert_eq!(calc_priority(b'A'), 27);
        assert_eq!(calc_priority(b'Z'), 52);
    }

    #[test]
    fn test_find_rucksack_priority() {
        assert_eq!(rucksack_priority("vJrwpWtwJgWrhcsFMMfFFhFp"), 16);
        assert_eq!(rucksack_priority("jqHRNqRjqzjGDLGLrsFMfFZSrLrFZsSL"), 38);
        assert_eq!(rucksack_priority("PmmdzqPrVvPwwTWBwg"), 42);
        assert_eq!(rucksack_priority("wMqvLMZHhHMvwLHjbvcjnnSBnvTQFn"), 22);
        assert_eq!(rucksack_priority("ttgJtRGJQctTZtZT"), 20);
        assert_eq!(rucksack_priority("CrZsJsPPZsGzwwsLwLmpwMDw"), 19);
    }
}
