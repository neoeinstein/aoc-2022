use std::{
    env,
    fmt::{self, Write},
    fs, io,
    io::Read,
    iter,
    ops::{self, ControlFlow}, marker::PhantomData,
};

use color_eyre::{Result, eyre::Context};
use nom::{
    branch::alt,
    bytes::complete::tag,
    combinator::{all_consuming, map, value},
    sequence::preceded,
    Finish, IResult, error::{VerboseError, convert_error},
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

    println!("Part 1: \n{}\n\nPart 2: \n{}", result.0, result.1);

    Ok(())
}

fn run(input: &str) -> Result<(i64, String)> {
    let mut computer = Computer::<6, 40>::default();
    let mut signal = 0;
    let ops = input
        .lines()
        .enumerate()
        .map(|(idx, l)| OpCode::parse(l).wrap_err_with(|| format!("invalid instruction {} at line {}", l, idx + 1)));
    let mut exec = computer.execute(ops);

    loop {
        match exec.tick() {
            ControlFlow::Continue(_) => {}
            ControlFlow::Break(Ok(_)) => break,
            ControlFlow::Break(Err(err)) => return Err(err),
        }

        let cycle = exec.computer.clock.cycle.0 + 1;
        if (cycle + 20) % 40 == 0 {
            signal += cycle as i64 * exec.computer.cpu.registers.x;
        }
    }

    Ok((signal, exec.computer.screen.to_string()))
}

#[derive(Debug, Default)]
struct Computer<const R: usize, const C: usize> {
    clock: Clock,
    cpu: Cpu,
    screen: Crt<R, C>,
}

impl<const R: usize, const C: usize> Computer<R, C> {
    fn execute<I, E>(&mut self, ops: I) -> Execution<I::IntoIter, E, R, C>
    where
        I: IntoIterator<Item = Result<OpCode, E>>,
    {
        let ops = ops.into_iter().fuse();
        Execution {
            computer: self,
            ops,
            _phantom: PhantomData,
        }
    }
}

#[derive(Debug)]
struct Execution<'a, I, E, const R: usize, const C: usize> {
    computer: &'a mut Computer<R, C>,
    ops: iter::Fuse<I>,
    _phantom: PhantomData<*const E>,
}

impl<'a, I, E, const R: usize, const C: usize> Execution<'a, I, E, R, C>
where
    I: Iterator<Item = Result<OpCode, E>>,
{
    fn tick(&mut self) -> ControlFlow<Result<(), E>> {
        self.computer.cpu.read_instruction(&mut self.ops)?;
        self.computer.clock.tick();
        self.computer.screen.tick(&self.computer.cpu.registers);
        self.computer.cpu.tick();
        ControlFlow::Continue(())
    }
}

#[derive(Debug)]
struct Clock {
    cycle: Cycles,
}

impl Default for Clock {
    fn default() -> Self {
        Self {
            cycle: Cycles::ZERO,
        }
    }
}

impl Clock {
    fn tick(&mut self) {
        self.cycle.incr();
        // println!("Cycle {}", self.cycle.0);
    }
}

#[derive(Debug)]
struct Cpu {
    registers: Registers,
    delay: Cycles,
    current_op: OpCode,
}

impl Default for Cpu {
    fn default() -> Self {
        Self {
            registers: Registers::default(),
            delay: Cycles::ZERO,
            current_op: OpCode::Noop,
        }
    }
}

impl Cpu {
    fn read_instruction<E>(&mut self, ops: &mut impl Iterator<Item = Result<OpCode, E>>) -> ControlFlow<Result<(), E>> {
        if self.delay == Cycles::ZERO {
            let Some(op) = ops.next() else {
                return ControlFlow::Break(Ok(()));
            };
            let op = match op {
                Ok(op) => op,
                Err(err) => return ControlFlow::Break(Err(err)),
            };

            self.current_op = op;
            self.delay = op.delay();
            // println!("Starting {:?} ({:?})", self.current_op, self.delay);
        }

        ControlFlow::Continue(())
    }

    fn tick(&mut self) {
        self.delay.decr();
        if self.delay == Cycles::ZERO {
            self.registers.apply(self.current_op);
            // println!("{:?}", self.registers);
        }
    }
}

#[derive(Clone, Debug)]
struct Crt<const R: usize, const C: usize> {
    column: usize,
    row: usize,
    screen: [[bool; C]; R],
}

impl<const R: usize, const C: usize> Default for Crt<R, C> {
    fn default() -> Self {
        Self {
            column: 0,
            row: 0,
            screen: [[false; C]; R],
        }
    }
}

impl<const R: usize, const C: usize> fmt::Display for Crt<R, C> {
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

impl<const R: usize, const C: usize> Crt<R, C> {
    fn tick_cursor(&mut self) {
        self.column = (self.column + 1) % C;
        if self.column == 0 {
            self.row = (self.row + 1) % R;
        }
    }

    fn tick(&mut self, registers: &Registers) {
        // println!("Drawing at {}", self.column);
        if (registers.x - 1..=registers.x + 1).contains(&(self.column as i64)) {
            self.screen[self.row][self.column] = true;
        }
        // for &pixel in &self.screen[self.row][..=self.column] {
        //     print!("{}", if pixel { '#' } else { '.' });
        // }
        // println!();
        self.tick_cursor()
    }
}

#[derive(Debug)]
struct Registers {
    x: i64,
}

impl Registers {
    fn apply(&mut self, op: OpCode) {
        match op {
            OpCode::Noop => {}
            OpCode::Addx(val) => self.x += val,
        }
    }
}

impl Default for Registers {
    fn default() -> Self {
        Self { x: 1 }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct Cycles(u32);

impl Cycles {
    const ZERO: Self = Self(0);

    fn decr(&mut self) {
        self.0 -= 1;
    }

    fn incr(&mut self) {
        self.0 += 1;
    }
}

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
            Self::Addx(_) => Cycles(2),
        }
    }

    fn parse(s: &str) -> Result<Self> {
        all_consuming(Self::token)(s)
            .finish()
            .map(|o| o.1)
            .map_err(|e| color_eyre::Report::msg(convert_error(s, e)))
    }

    fn token(s: &str) -> IResult<&str, Self, VerboseError<&str>> {
        alt((
            value(Self::Noop, tag("noop")),
            map(
                preceded(tag("addx "), nom::character::complete::i64),
                Self::Addx,
            ),
        ))(s)
    }
}

#[cfg(test)]
mod tests {
    use test_case::test_case;

    use super::*;

    #[rustfmt::skip]
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
        run(input)
    }
}
