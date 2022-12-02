use std::io;

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
    alt((opp_rock, opp_paper, opp_scissors))(s)
}

fn opp_rock(s: &str) -> nom::IResult<&str, OpponentThrow> {
    value(OpponentThrow::Rock, tag("A"))(s)
}

fn opp_paper(s: &str) -> nom::IResult<&str, OpponentThrow> {
    value(OpponentThrow::Paper, tag("B"))(s)
}

fn opp_scissors(s: &str) -> nom::IResult<&str, OpponentThrow> {
    value(OpponentThrow::Scissors, tag("C"))(s)
}

fn us_throw(s: &str) -> nom::IResult<&str, OurThrow> {
    alt((us_rock, us_paper, us_scissors))(s)
}

fn us_rock(s: &str) -> nom::IResult<&str, OurThrow> {
    value(OurThrow::Rock, tag("X"))(s)
}

fn us_paper(s: &str) -> nom::IResult<&str, OurThrow> {
    value(OurThrow::Paper, tag("Y"))(s)
}

fn us_scissors(s: &str) -> nom::IResult<&str, OurThrow> {
    value(OurThrow::Scissors, tag("Z"))(s)
}

fn expected_result(s: &str) -> nom::IResult<&str, RoundResult> {
    alt((expect_win, expect_draw, expect_loss))(s)
}

fn expect_loss(s: &str) -> nom::IResult<&str, RoundResult> {
    value(RoundResult::Loss, tag("X"))(s)
}

fn expect_draw(s: &str) -> nom::IResult<&str, RoundResult> {
    value(RoundResult::Draw, tag("Y"))(s)
}

fn expect_win(s: &str) -> nom::IResult<&str, RoundResult> {
    value(RoundResult::Win, tag("Z"))(s)
}

#[derive(Clone, Copy, Hash, PartialEq, Eq)]
struct EspRound {
    opponent: OpponentThrow,
    expected_result: RoundResult,
}

impl EspRound {
    fn best_move(&self) -> OurThrow {
        match (self.opponent, self.expected_result) {
            (OpponentThrow::Rock, RoundResult::Win) => OurThrow::Paper,
            (OpponentThrow::Rock, RoundResult::Draw) => OurThrow::Rock,
            (OpponentThrow::Rock, RoundResult::Loss) => OurThrow::Scissors,
            (OpponentThrow::Paper, RoundResult::Win) => OurThrow::Scissors,
            (OpponentThrow::Paper, RoundResult::Draw) => OurThrow::Paper,
            (OpponentThrow::Paper, RoundResult::Loss) => OurThrow::Rock,
            (OpponentThrow::Scissors, RoundResult::Win) => OurThrow::Rock,
            (OpponentThrow::Scissors, RoundResult::Draw) => OurThrow::Scissors,
            (OpponentThrow::Scissors, RoundResult::Loss) => OurThrow::Paper,
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
    fn result(&self) -> RoundResult {
        match (self.opponent, self.us) {
            (OpponentThrow::Rock, OurThrow::Paper) => RoundResult::Win,
            (OpponentThrow::Rock, OurThrow::Rock) => RoundResult::Draw,
            (OpponentThrow::Rock, OurThrow::Scissors) => RoundResult::Loss,
            (OpponentThrow::Paper, OurThrow::Scissors) => RoundResult::Win,
            (OpponentThrow::Paper, OurThrow::Paper) => RoundResult::Draw,
            (OpponentThrow::Paper, OurThrow::Rock) => RoundResult::Loss,
            (OpponentThrow::Scissors, OurThrow::Rock) => RoundResult::Win,
            (OpponentThrow::Scissors, OurThrow::Scissors) => RoundResult::Draw,
            (OpponentThrow::Scissors, OurThrow::Paper) => RoundResult::Loss,
        }
    }

    fn score(&self) -> u32 {
        let round_score = self.result().score();
        let throw_score = self.us.score();
        round_score + throw_score
    }
}

#[derive(Clone, Copy, Hash, PartialEq, Eq)]
enum OurThrow {
    Rock,
    Paper,
    Scissors,
}

impl OurThrow {
    fn score(&self) -> u32 {
        match self {
            Self::Rock => 1,
            Self::Paper => 2,
            Self::Scissors => 3,
        }
    }
}

#[derive(Clone, Copy, Hash, PartialEq, Eq)]
enum OpponentThrow {
    Rock,
    Paper,
    Scissors,
}

#[derive(Clone, Copy, Hash, PartialEq, Eq)]
enum RoundResult {
    Loss,
    Draw,
    Win,
}

impl RoundResult {
    fn score(&self) -> u32 {
        match self {
            Self::Loss => 0,
            Self::Draw => 3,
            Self::Win => 6,
        }
    }
}
