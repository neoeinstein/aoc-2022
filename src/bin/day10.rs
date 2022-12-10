use std::{env, fs, io, io::Read, ops, fmt::{self, Write}};

use color_eyre::Result;
use nom::{IResult, sequence::preceded, combinator::{map, value, all_consuming}, branch::alt, bytes::complete::tag, Finish};

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

    println!("Part 1: \n{}\n\nPart 2: \n{}", result.0, result.1);
    
    Ok(())
}

fn run(input: &str) -> Result<(i64, String)> {
    let mut regs = Registers::default();
    let mut clock = Cycles(0);
    let mut signal = 0;
    let mut crt = Crt::default();
    for line in input.lines() {
        clock.0 += 1;
        // println!("Cycle {}", clock.0);
        let op = OpCode::parse(line)?;
        // println!("Starting {:?}", op);
        crt.draw(regs.x);
        for _ in 0..op.delay().0-1 {
            if (clock.0 + 20) % 40 == 0 {
                signal += clock.0 as i64 * regs.x;
            }
            clock.0 += 1;
            // println!("Cycle {}", clock.0);
            crt.draw(regs.x);
        }
        if (clock.0 + 20) % 40 == 0 {
            signal += clock.0 as i64 * regs.x;
        }
        regs.apply(op);
        // println!("x: {}", regs.x);
    }

    Ok((signal, crt.to_string()))
}

#[derive(Clone, Debug)]
struct Crt {
    column: usize,
    row: usize,
    screen: [[bool; 40]; 6],
}

impl Default for Crt {
    fn default() -> Self {
        Self {
            column: 0,
            row: 0,
            screen: [[false; 40]; 6],
        }
    }
}

impl fmt::Display for Crt {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for row in &self.screen {
            for &pixel in row {
                f.write_char(if pixel { '#' } else { '.' })?;
            }
            f.write_char('\n')?;
        }
        Ok(())
    }
}

impl Crt {
    fn tick(&mut self) {
        self.column = (self.column + 1) % 40;
        if self.column == 0 {
            self.row += 1;
        }
    }

    fn draw(&mut self, x: i64) {
        // println!("Drawing at {}", self.column);
        if (x - 1..=x + 1).contains(&(self.column as i64)) {
            self.screen[self.row][self.column] = true;
        }
        // for &pixel in &self.screen[self.row][..=self.column] {
        //     print!("{}", if pixel { '#' } else { '.' });
        // }
        // println!();
        self.tick()
    }
}

#[derive(Debug)]
struct Registers {
    x: i64
}

impl Registers {
    fn apply(&mut self, op: OpCode) {
        match op {
            OpCode::Noop => {},
            OpCode::Addx(val) => self.x += val,
        }
    }
}

impl Default for Registers {
    fn default() -> Self {
        Self {
            x: 1,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct Cycles(u32);

impl ops::AddAssign for Cycles {
    fn add_assign(&mut self, rhs: Self) {
        self.0 += rhs.0
    }
}

impl ops::Add for Cycles {
    type Output = Self;

    fn add(mut self, rhs: Self) -> Self::Output {
        self += rhs;
        self
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum OpCode {
    Noop,
    Addx(i64),
}

impl OpCode {
    fn delay(&self) -> Cycles {
        match self {
            Self::Noop => Cycles(1),
            Self::Addx(_) => Cycles(2)
        }
    }

    fn parse(s: &str) -> Result<Self, nom::error::Error<String>> {
        all_consuming(Self::token)(s).map_err(|e| e.to_owned()).finish().map(|o| o.1)
    }

    fn token(s: &str) -> IResult<&str, Self> {
        alt((
            value(Self::Noop, tag("noop")),
            map(preceded(tag("addx "), nom::character::complete::i64), Self::Addx)
        ))(s)
    }
}


#[cfg(test)]
mod tests {
    use test_case::test_case;

    use super::*;

    const EXPECTED: &str = "\
        ##..##..##..##..##..##..##..##..##..##..\n\
        ###...###...###...###...###...###...###.\n\
        ####....####....####....####....####....\n\
        #####.....#####.....#####.....#####.....\n\
        ######......######......######......####\n\
        #######.......#######.......#######.....\n";

    #[test_case(include_str!("../../input/day10test") => matches Ok((13140, s)) if s == EXPECTED)]
    #[test_case(include_str!("../../input/day10test2") => matches Ok((_, s)) if &s[..21] == "##..##..##..##..##..#")]
    fn default_tests(input: &str) -> Result<(i64, String)> {
        println!("{:?}", EXPECTED);
        run(input)
    }
}
