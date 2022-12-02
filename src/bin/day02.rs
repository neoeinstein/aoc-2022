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

fn opp_throw(s: &str) -> nom::IResult<&str, Throw> {
    alt((opp_rock, opp_paper, opp_scissors))(s)
}

fn opp_rock(s: &str) -> nom::IResult<&str, Throw> {
    map(tag("A"), |_| Throw::Rock)(s)
}

fn opp_paper(s: &str) -> nom::IResult<&str, Throw> {
    map(tag("B"), |_| Throw::Paper)(s)
}

fn opp_scissors(s: &str) -> nom::IResult<&str, Throw> {
    map(tag("C"), |_| Throw::Scissors)(s)
}

fn us_throw(s: &str) -> nom::IResult<&str, Throw> {
    alt((us_rock, us_paper, us_scissors))(s)
}

fn us_rock(s: &str) -> nom::IResult<&str, Throw> {
    map(tag("X"), |_| Throw::Rock)(s)
}

fn us_paper(s: &str) -> nom::IResult<&str, Throw> {
    map(tag("Y"), |_| Throw::Paper)(s)
}

fn us_scissors(s: &str) -> nom::IResult<&str, Throw> {
    map(tag("Z"), |_| Throw::Scissors)(s)
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
    rounds.insert(Round { opponent: Throw::Rock, us: Throw::Rock }, RoundResult::Draw);
    rounds.insert(Round { opponent: Throw::Rock, us: Throw::Paper }, RoundResult::Win);
    rounds.insert(Round { opponent: Throw::Rock, us: Throw::Scissors }, RoundResult::Loss);
    rounds.insert(Round { opponent: Throw::Paper, us: Throw::Rock }, RoundResult::Loss);
    rounds.insert(Round { opponent: Throw::Paper, us: Throw::Paper }, RoundResult::Draw);
    rounds.insert(Round { opponent: Throw::Paper, us: Throw::Scissors }, RoundResult::Win);
    rounds.insert(Round { opponent: Throw::Scissors, us: Throw::Rock }, RoundResult::Win);
    rounds.insert(Round { opponent: Throw::Scissors, us: Throw::Paper }, RoundResult::Loss);
    rounds.insert(Round { opponent: Throw::Scissors, us: Throw::Scissors }, RoundResult::Draw);
    rounds
}

static ESP_ROUND_TABLE: Lazy<HashMap<EspRound, Throw>> = Lazy::new(generate_esp_round_table);

fn generate_esp_round_table() -> HashMap<EspRound, Throw> {
    let mut rounds = HashMap::with_capacity(9);
    rounds.insert(EspRound { opponent: Throw::Rock, expected_result: RoundResult::Win}, Throw::Paper);
    rounds.insert(EspRound { opponent: Throw::Rock, expected_result: RoundResult::Draw}, Throw::Rock);
    rounds.insert(EspRound { opponent: Throw::Rock, expected_result: RoundResult::Loss}, Throw::Scissors);
    rounds.insert(EspRound { opponent: Throw::Paper, expected_result: RoundResult::Win}, Throw::Scissors);
    rounds.insert(EspRound { opponent: Throw::Paper, expected_result: RoundResult::Draw}, Throw::Paper);
    rounds.insert(EspRound { opponent: Throw::Paper, expected_result: RoundResult::Loss}, Throw::Rock);
    rounds.insert(EspRound { opponent: Throw::Scissors, expected_result: RoundResult::Win}, Throw::Rock);
    rounds.insert(EspRound { opponent: Throw::Scissors, expected_result: RoundResult::Draw}, Throw::Scissors);
    rounds.insert(EspRound { opponent: Throw::Scissors, expected_result: RoundResult::Loss}, Throw::Paper);
    rounds
}

#[derive(Clone, Copy, Hash, PartialEq, Eq)]
struct EspRound {
    opponent: Throw,
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
    opponent: Throw,
    us: Throw,
}

impl Round {
    fn score(&self) -> u32 {
        let round_score = ROUND_TABLE.get(self).expect("table is perfect").score();
        let throw_score = self.us.score();
        round_score + throw_score
    }
}

#[derive(Clone, Copy, Hash, PartialEq, Eq)]
enum Throw {
    Rock,
    Paper,
    Scissors,
}

impl Throw {
    fn score(&self) -> u32 {
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
    fn score(&self) -> u32 {
        match self {
            Self::Loss => 0,
            Self::Draw => 3,
            Self::Win => 6,
        }
    }
}