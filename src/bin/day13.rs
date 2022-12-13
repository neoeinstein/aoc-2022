use std::{cmp, collections::BTreeSet, env, fmt, fs, io, io::Read, str};

use color_eyre::Result;
use itertools::Itertools;
use nom::{
    branch::alt, bytes::complete::tag, combinator::map, multi::separated_list0,
    sequence::delimited, Finish, IResult,
};

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
    let divider_1: Packet = Packet::List(vec![Packet::List(vec![Packet::Integer(2)])]);
    let divider_2: Packet = Packet::List(vec![Packet::List(vec![Packet::Integer(6)])]);
    let mut in_order = 0;
    let mut ordered_packets = BTreeSet::new();
    ordered_packets.extend([divider_1.clone(), divider_2.clone()]);
    for (idx, (l, r)) in input.lines().filter(|s| !s.is_empty()).tuples().enumerate() {
        let l = l.parse::<Packet>()?;
        let r = r.parse::<Packet>()?;
        // println!("{:?} <= {:?} = {}", l, r, l <= r);
        if l <= r {
            in_order += idx + 1;
        }
        ordered_packets.extend([l, r]);
    }

    // dbg!(&ordered_packets);

    ordered_packets.split_off(&divider_2);
    let index_2 = ordered_packets.len() + 1;
    ordered_packets.split_off(&divider_1);
    let index_1 = ordered_packets.len() + 1;

    Ok((in_order, index_1 * index_2))
}

#[derive(Clone)]
enum Packet {
    Integer(u64),
    List(Vec<Packet>),
}

impl fmt::Debug for Packet {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Integer(i) => write!(f, "{}", i),
            Self::List(l) => write!(f, "{:?}", l),
        }
    }
}

impl Packet {
    fn token(s: &str) -> IResult<&str, Self> {
        alt((
            map(nom::character::complete::u64, Self::Integer),
            Self::list_token,
        ))(s)
    }

    fn list_token(s: &str) -> IResult<&str, Self> {
        delimited(
            tag("["),
            map(separated_list0(tag(","), Self::token), Self::List),
            tag("]"),
        )(s)
    }
}

impl str::FromStr for Packet {
    type Err = nom::error::Error<String>;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(Packet::list_token(s).map_err(|e| e.to_owned()).finish()?.1)
    }
}

impl cmp::Ord for Packet {
    fn cmp(&self, other: &Self) -> cmp::Ordering {
        match (self, other) {
            (Self::Integer(l), Self::Integer(r)) => l.cmp(r),
            (Self::List(l), Self::List(r)) => {
                for (l, r) in l.iter().zip(r) {
                    match l.cmp(r) {
                        cmp::Ordering::Equal => {}
                        result => return result,
                    }
                }
                l.len().cmp(&r.len())
            }
            (Self::Integer(l), Self::List(_)) => Packet::List(vec![Packet::Integer(*l)]).cmp(other),
            (Self::List(_), Self::Integer(r)) => self.cmp(&Packet::List(vec![Packet::Integer(*r)])),
        }
    }
}

impl cmp::PartialOrd for Packet {
    fn partial_cmp(&self, other: &Self) -> Option<cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl cmp::PartialEq for Packet {
    fn eq(&self, other: &Self) -> bool {
        self.cmp(other) == cmp::Ordering::Equal
    }
}

impl cmp::Eq for Packet {}

#[cfg(test)]
mod tests {
    use test_case::test_case;

    use super::*;

    #[test_case(include_str!("../../input/day13test") => matches Ok((13, 140)))]
    fn default_tests(input: &str) -> Result<(usize, usize)> {
        run(input)
    }

    #[test_case(
        "[1,1,3,1,1]".parse().unwrap(),
        "[1,1,5,1,1]".parse().unwrap()
        => cmp::Ordering::Less
    )]
    #[test_case(
        "[[1],[2,3,4]]".parse().unwrap(),
        "[[1],4]".parse().unwrap()
        => cmp::Ordering::Less
    )]
    #[test_case(
        "[9]".parse().unwrap(),
        "[[8,7,6]]".parse().unwrap()
        => cmp::Ordering::Greater
    )]
    #[test_case(
        "[[4,4],4,4]".parse().unwrap(),
        "[[4,4],4,4,4]".parse().unwrap()
        => cmp::Ordering::Less
    )]
    #[test_case(
        "[7,7,7,7]".parse().unwrap(),
        "[7,7,7]".parse().unwrap()
        => cmp::Ordering::Greater
    )]
    #[test_case(
        "[]".parse().unwrap(),
        "[3]".parse().unwrap()
        => cmp::Ordering::Less
    )]
    #[test_case(
        "[[[]]]".parse().unwrap(),
        "[[]]".parse().unwrap()
        => cmp::Ordering::Greater
    )]
    #[test_case(
        "[1,[2,[3,[4,[5,6,7]]]],8,9]".parse().unwrap(),
        "[1,[2,[3,[4,[5,6,0]]]],8,9]".parse().unwrap()
        => cmp::Ordering::Greater
    )]
    fn check_ordering(left: Packet, right: Packet) -> cmp::Ordering {
        left.cmp(&right)
    }
}
