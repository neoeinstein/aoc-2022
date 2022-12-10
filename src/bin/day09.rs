use std::{cmp, env, fmt, fs, io, io::Read};

use color_eyre::Result;
use fxhash::FxHashSet;
use nom::{
    branch::alt,
    bytes::complete::tag,
    character::complete::line_ending,
    combinator::{map, value},
    sequence::{separated_pair, terminated},
    Finish, IResult,
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

    println!("{result:?}");

    Ok(())
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct Move {
    direction: Direction,
    distance: u8,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Direction {
    Up,
    Left,
    Down,
    Right,
}

impl Position {
    fn step(&mut self, direction: Direction) {
        match direction {
            Direction::Up => self.y += 1,
            Direction::Down => self.y -= 1,
            Direction::Right => self.x += 1,
            Direction::Left => self.x -= 1,
        }
    }
}

fn step(s: &str) -> IResult<&str, Move> {
    map(
        terminated(
            separated_pair(dir, tag(" "), nom::character::complete::u8),
            line_ending,
        ),
        |(direction, distance)| Move {
            direction,
            distance,
        },
    )(s)
}

fn dir(s: &str) -> IResult<&str, Direction> {
    alt((
        value(Direction::Up, tag("U")),
        value(Direction::Down, tag("D")),
        value(Direction::Left, tag("L")),
        value(Direction::Right, tag("R")),
    ))(s)
}

fn run(mut input: &str) -> Result<(usize, usize)> {
    let mut snake2 = Snake::<2>::new();
    let mut snake10 = Snake::<10>::new();
    while let Ok((rem, step)) = step(input).map_err(|e| e.to_owned()).finish() {
        input = rem;
        for _ in 0..step.distance {
            snake2.advance(step.direction);
            snake10.advance(step.direction);
        }
        // println!("{:?}", snake10.segments);
    }

    Ok((snake2.tail_visits(), snake10.tail_visits()))
}

#[derive(Clone, Copy, Hash, PartialEq, Eq)]
struct Position {
    x: i32,
    y: i32,
}

impl fmt::Debug for Position {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_tuple("Position")
            .field(&self.x)
            .field(&self.y)
            .finish()
    }
}

impl Position {
    const ORIGIN: Self = Self { x: 0, y: 0 };

    fn is_adjacent(self, other: &Position) -> bool {
        (self.x - 1..=self.x + 1).contains(&other.x) && (self.y - 1..=self.y + 1).contains(&other.y)
    }

    fn pull(self, tail: &mut Position) {
        tail.x += Self::catchup(self.x, tail.x);
        tail.y += Self::catchup(self.y, tail.y);
    }

    fn catchup(h: i32, t: i32) -> i32 {
        match h.cmp(&t) {
            cmp::Ordering::Greater => 1,
            cmp::Ordering::Less => -1,
            cmp::Ordering::Equal => 0,
        }
    }
}

struct Snake<const N: usize> {
    segments: [Position; N],
    tail_positions: FxHashSet<Position>,
}

impl<const N: usize> Snake<N> {
    fn new() -> Self {
        Self {
            segments: [Position::ORIGIN; N],
            tail_positions: [Position::ORIGIN].into_iter().collect(),
        }
    }

    fn advance(&mut self, direction: Direction) {
        let prior_tail = self.tail_position();
        self.segments[0].step(direction);

        let mut head = self.segments[0];
        for tail in &mut self.segments[1..] {
            if !tail.is_adjacent(&head) {
                head.pull(tail);
                head = *tail;
            } else {
                break;
            }
        }

        if prior_tail != self.tail_position() {
            self.tail_positions.insert(self.tail_position());
        }
    }

    fn tail_position(&self) -> Position {
        self.segments[N - 1]
    }

    fn tail_visits(&self) -> usize {
        self.tail_positions.len()
    }
}

#[cfg(test)]
mod tests {
    use test_case::test_case;

    use super::*;

    #[test_case(include_str!("../../input/day09test") => matches Ok((13, 1)))]
    #[test_case(include_str!("../../input/day09test2") => matches Ok((_, 36)))]
    fn default_tests(input: &str) -> Result<(usize, usize)> {
        run(input)
    }
}
