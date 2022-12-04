use std::{cmp, io};

use nom::{
    branch::alt,
    bytes::complete::tag,
    combinator::{map, value},
    sequence::separated_pair,
    Finish,
};

fn main() -> color_eyre::Result<()> {
    let (part1_score, part2_score): (u32, u32) = io::stdin()
        .lines()
        .map(|line| -> color_eyre::Result<_> {
            let line = line?;
            let (_, p1) = round(&line).map_err(|e| e.to_owned()).finish()?;
            let (_, p2) = esp_round(&line).map_err(|e| e.to_owned()).finish()?;
            Ok((p1.score(), p2.score()))
        })
        .try_fold((0, 0), |(acc_p1, acc_p2), next| -> color_eyre::Result<_> {
            let (p1, p2) = next?;
            Ok((acc_p1 + p1, acc_p2 + p2))
        })?;

    println!("{part1_score} {part2_score}");

    Ok(())
}

fn round(s: &str) -> nom::IResult<&str, Round> {
    map(
        separated_pair(opp_throw, tag(" "), us_throw),
        |(opponent, us)| Round { opponent, us },
    )(s)
}

fn esp_round(s: &str) -> nom::IResult<&str, EspRound> {
    map(
        separated_pair(opp_throw, tag(" "), expected_result),
        |(opponent, expected_result)| EspRound {
            opponent,
            expected_result,
        },
    )(s)
}

fn opp_throw(s: &str) -> nom::IResult<&str, OpponentThrow> {
    alt((
        value(OpponentThrow(Move::Rock), tag("A")),
        value(OpponentThrow(Move::Paper), tag("B")),
        value(OpponentThrow(Move::Scissors), tag("C")),
    ))(s)
}

fn us_throw(s: &str) -> nom::IResult<&str, OurThrow> {
    alt((
        value(OurThrow(Move::Rock), tag("X")),
        value(OurThrow(Move::Paper), tag("Y")),
        value(OurThrow(Move::Scissors), tag("Z")),
    ))(s)
}

fn expected_result(s: &str) -> nom::IResult<&str, RoundResult> {
    alt((
        value(RoundResult::Loss, tag("X")),
        value(RoundResult::Draw, tag("Y")),
        value(RoundResult::Win, tag("Z")),
    ))(s)
}

#[derive(Clone, Copy, Hash, PartialEq, Eq)]
struct EspRound {
    opponent: OpponentThrow,
    expected_result: RoundResult,
}

impl EspRound {
    fn best_move(&self) -> OurThrow {
        match self.expected_result {
            RoundResult::Win => OurThrow(self.opponent.r#move().beaten_by()),
            RoundResult::Draw => OurThrow(self.opponent.r#move()),
            RoundResult::Loss => OurThrow(self.opponent.r#move().beats()),
        }
    }

    fn score(&self) -> u32 {
        let throw_score = self.best_move().score();
        let round_score = self.expected_result.score();
        round_score + throw_score
    }
}

#[derive(Clone, Copy, Hash, PartialEq, Eq)]
struct Round {
    opponent: OpponentThrow,
    us: OurThrow,
}

impl Round {
    fn result(self) -> RoundResult {
        match self.us.r#move().partial_cmp(&self.opponent.r#move()) {
            Some(cmp::Ordering::Equal) => RoundResult::Draw,
            Some(cmp::Ordering::Greater) => RoundResult::Win,
            Some(cmp::Ordering::Less) => RoundResult::Loss,
            None => unreachable!("moves don't have a total ordering, but any pair can be compared"),
        }
    }

    fn score(self) -> u32 {
        let round_score = self.result().score();
        let throw_score = self.us.score();
        round_score + throw_score
    }
}

#[derive(Clone, Copy, Hash, PartialEq, Eq)]
struct OurThrow(Move);

impl OurThrow {
    fn r#move(self) -> Move {
        self.0
    }

    fn score(self) -> u32 {
        self.0.score()
    }
}

#[derive(Clone, Copy, Hash, PartialEq, Eq)]
struct OpponentThrow(Move);

impl OpponentThrow {
    fn r#move(self) -> Move {
        self.0
    }
}

#[derive(Clone, Copy, Hash, PartialEq, Eq)]
enum Move {
    Rock,
    Paper,
    Scissors,
}

impl Move {
    fn beats(self) -> Self {
        match self {
            Self::Rock => Self::Scissors,
            Self::Paper => Self::Rock,
            Self::Scissors => Self::Paper,
        }
    }

    fn beaten_by(self) -> Self {
        match self {
            Self::Rock => Self::Paper,
            Self::Paper => Self::Scissors,
            Self::Scissors => Self::Rock,
        }
    }

    fn score(self) -> u32 {
        match self {
            Self::Rock => 1,
            Self::Paper => 2,
            Self::Scissors => 3,
        }
    }
}

#[derive(Clone, Copy, Hash, PartialEq, Eq)]
enum RoundResult {
    Loss,
    Draw,
    Win,
}

impl RoundResult {
    fn score(self) -> u32 {
        match self {
            Self::Loss => 0,
            Self::Draw => 3,
            Self::Win => 6,
        }
    }
}

impl cmp::PartialEq<OurThrow> for OpponentThrow {
    fn eq(&self, other: &OurThrow) -> bool {
        self.0 == other.0
    }
}

impl cmp::PartialOrd for Move {
    fn partial_cmp(&self, other: &Self) -> Option<cmp::Ordering> {
        let order = if self == other {
            cmp::Ordering::Equal
        } else if self.beats() == *other {
            cmp::Ordering::Greater
        } else {
            cmp::Ordering::Less
        };
        Some(order)
    }
}
