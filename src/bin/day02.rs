use std::io;

use ahash::{HashMap, HashMapExt};
use nom::Finish;
use nom::bytes::complete::tag;
use nom::combinator::map;
use nom::branch::alt;
use nom::sequence::separated_pair;
use once_cell::sync::Lazy;

fn main() -> color_eyre::Result<()> {
    let (part1_score, part2_score): (u32, u32) = io::stdin().lines()
        .map(|line| -> color_eyre::Result<_> {
            let line = line?;
            let (_, p1) = round(&line).finish().map_err(|e| nom::error::Error { input: e.input.to_owned(), code: e.code })?;
            let (_, p2) = esp_round(&line).finish().map_err(|e| nom::error::Error { input: e.input.to_owned(), code: e.code })?;
            Ok((p1, p2))
        })
        .map(|round| round.map(|(p1, p2)| (p1.score(), p2.score())))
        .try_fold((0, 0), |(acc_p1, acc_p2), next| -> color_eyre::Result<_> {
            let (p1, p2) = next?;
            Ok((acc_p1 + p1, acc_p2 + p2))
        })?;

    println!("{part1_score} {part2_score}");
    
    Ok(())
}

fn round(s: &str) -> nom::IResult<&str, Round> {
    map(separated_pair(opp_throw, tag(" "), us_throw), |(opponent, us)| Round { opponent, us })(s)
}

fn esp_round(s: &str) -> nom::IResult<&str, EspRound> {
    map(separated_pair(opp_throw, tag(" "), expected_result), |(opponent, expected_result)| EspRound { opponent, expected_result })(s)
}

fn opp_throw(s: &str) -> nom::IResult<&str, OpponentThrow> {
    alt((opp_rock, opp_paper, opp_scissors))(s)
}

fn opp_rock(s: &str) -> nom::IResult<&str, OpponentThrow> {
    map(tag("A"), |_| OpponentThrow::Rock)(s)
}

fn opp_paper(s: &str) -> nom::IResult<&str, OpponentThrow> {
    map(tag("B"), |_| OpponentThrow::Paper)(s)
}

fn opp_scissors(s: &str) -> nom::IResult<&str, OpponentThrow> {
    map(tag("C"), |_| OpponentThrow::Scissors)(s)
}

fn us_throw(s: &str) -> nom::IResult<&str, OurThrow> {
    alt((us_rock, us_paper, us_scissors))(s)
}

fn us_rock(s: &str) -> nom::IResult<&str, OurThrow> {
    map(tag("X"), |_| OurThrow::Rock)(s)
}

fn us_paper(s: &str) -> nom::IResult<&str, OurThrow> {
    map(tag("Y"), |_| OurThrow::Paper)(s)
}

fn us_scissors(s: &str) -> nom::IResult<&str, OurThrow> {
    map(tag("Z"), |_| OurThrow::Scissors)(s)
}

fn expected_result(s: &str) -> nom::IResult<&str, RoundResult> {
    alt((expect_win, expect_draw, expect_loss))(s)
}

fn expect_loss(s: &str) -> nom::IResult<&str, RoundResult> {
    map(tag("X"), |_| RoundResult::Loss)(s)
}

fn expect_draw(s: &str) -> nom::IResult<&str, RoundResult> {
    map(tag("Y"), |_| RoundResult::Draw)(s)
}

fn expect_win(s: &str) -> nom::IResult<&str, RoundResult> {
    map(tag("Z"), |_| RoundResult::Win)(s)
}

static ROUND_TABLE: Lazy<HashMap<Round, RoundResult>> = Lazy::new(generate_round_table);

fn generate_round_table() -> HashMap<Round, RoundResult> {
    let mut rounds = HashMap::with_capacity(9);
    rounds.insert(Round { opponent: OpponentThrow::Rock, us: OurThrow::Rock }, RoundResult::Draw);
    rounds.insert(Round { opponent: OpponentThrow::Rock, us: OurThrow::Paper }, RoundResult::Win);
    rounds.insert(Round { opponent: OpponentThrow::Rock, us: OurThrow::Scissors }, RoundResult::Loss);
    rounds.insert(Round { opponent: OpponentThrow::Paper, us: OurThrow::Rock }, RoundResult::Loss);
    rounds.insert(Round { opponent: OpponentThrow::Paper, us: OurThrow::Paper }, RoundResult::Draw);
    rounds.insert(Round { opponent: OpponentThrow::Paper, us: OurThrow::Scissors }, RoundResult::Win);
    rounds.insert(Round { opponent: OpponentThrow::Scissors, us: OurThrow::Rock }, RoundResult::Win);
    rounds.insert(Round { opponent: OpponentThrow::Scissors, us: OurThrow::Paper }, RoundResult::Loss);
    rounds.insert(Round { opponent: OpponentThrow::Scissors, us: OurThrow::Scissors }, RoundResult::Draw);
    rounds
}

static ESP_ROUND_TABLE: Lazy<HashMap<EspRound, OurThrow>> = Lazy::new(generate_esp_round_table);

fn generate_esp_round_table() -> HashMap<EspRound, OurThrow> {
    let mut rounds = HashMap::with_capacity(9);
    rounds.insert(EspRound { opponent: OpponentThrow::Rock, expected_result: RoundResult::Win}, OurThrow::Paper);
    rounds.insert(EspRound { opponent: OpponentThrow::Rock, expected_result: RoundResult::Draw}, OurThrow::Rock);
    rounds.insert(EspRound { opponent: OpponentThrow::Rock, expected_result: RoundResult::Loss}, OurThrow::Scissors);
    rounds.insert(EspRound { opponent: OpponentThrow::Paper, expected_result: RoundResult::Win}, OurThrow::Scissors);
    rounds.insert(EspRound { opponent: OpponentThrow::Paper, expected_result: RoundResult::Draw}, OurThrow::Paper);
    rounds.insert(EspRound { opponent: OpponentThrow::Paper, expected_result: RoundResult::Loss}, OurThrow::Rock);
    rounds.insert(EspRound { opponent: OpponentThrow::Scissors, expected_result: RoundResult::Win}, OurThrow::Rock);
    rounds.insert(EspRound { opponent: OpponentThrow::Scissors, expected_result: RoundResult::Draw}, OurThrow::Scissors);
    rounds.insert(EspRound { opponent: OpponentThrow::Scissors, expected_result: RoundResult::Loss}, OurThrow::Paper);
    rounds
}

#[derive(Clone, Copy, Hash, PartialEq, Eq)]
struct EspRound {
    opponent: OpponentThrow,
    expected_result: RoundResult,
}

impl EspRound {
    fn score(&self) -> u32 {
        let throw_score = ESP_ROUND_TABLE.get(self).expect("table is perfect").score();
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
    fn score(&self) -> u32 {
        let round_score = ROUND_TABLE.get(self).expect("table is perfect").score();
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