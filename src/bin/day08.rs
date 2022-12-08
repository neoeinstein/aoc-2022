use std::{env, fs, io, io::Read};

use color_eyre::Result;
use fxhash::FxHashSet;

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

struct Forest<'a> {
    trees: &'a [u8],
    width: usize,
    width_and_gutter: usize,
}

#[derive(Clone, Copy, Debug, Hash, PartialEq, Eq)]
struct Position {
    x: usize,
    y: usize,
}

enum View {
    North,
    East,
    South,
    West,
}

enum Direction<N, R> {
    Forward(N),
    Reverse(R),
}

impl<N, R> Iterator for Direction<N, R>
where
    N: Iterator,
    R: DoubleEndedIterator<Item = N::Item>,
{
    type Item = N::Item;
    fn next(&mut self) -> Option<Self::Item> {
        match self {
            Self::Forward(i) => i.next(),
            Self::Reverse(i) => i.next_back(),
        }
    }
}

impl<'a> Forest<'a> {
    fn new(trees: &'a [u8]) -> Self {
        let width = trees.iter().position(|&b| b.is_ascii_whitespace()).unwrap();
        let gutter = if trees[width] == b'\r' && trees[width + 1] == b'\n' {
            2
        } else {
            1
        };
        Self {
            trees,
            width,
            width_and_gutter: width + gutter,
        }
    }

    fn get(&self, tree: Position) -> Option<u8> {
        if tree.x > self.width || tree.y > self.width {
            None
        } else {
            self.trees
                .get(tree.x + tree.y * self.width_and_gutter)
                .copied()
        }
    }

    fn view_score(&self, tree: Position) -> usize {
        let view_height = self.get(tree).unwrap();
        let seen_left = self.view_from(View::East, tree, view_height).count();
        let seen_right = self.view_from(View::West, tree, view_height).count();
        let seen_down = self.view_from(View::South, tree, view_height).count();
        let seen_up = self.view_from(View::North, tree, view_height).count();
        seen_left * seen_right * seen_up * seen_down
    }

    fn view_from(
        &self,
        view: View,
        tree: Position,
        view_height: u8,
    ) -> impl Iterator<Item = Position> + '_ {
        let (constant, change) = match view {
            View::North => (tree.x, Direction::Reverse(0..tree.y)),
            View::South => (tree.x, Direction::Forward((tree.y + 1)..self.width)),
            View::East => (tree.y, Direction::Reverse(0..tree.x)),
            View::West => (tree.y, Direction::Forward((tree.x + 1)..self.width)),
        };

        let mut stop = false;
        change.map_while(move |change| {
            if stop {
                return None;
            }

            let tree = Position {
                x: match view {
                    View::East | View::West => change,
                    _ => constant,
                },
                y: match view {
                    View::North | View::South => change,
                    _ => constant,
                },
            };

            let tree_height = self.get(tree)?;
            if tree_height >= view_height {
                stop = true;
            }
            Some(tree)
        })
    }

    fn taller_than_priors(&self, initial_height: u8) -> impl FnMut(&Position) -> bool + '_ {
        let mut max_height = initial_height;
        move |&tree| {
            let tree_height = self.get(tree).unwrap_or_default();
            if tree_height > max_height {
                // println!("tree at {:?} height {} TALLER", tree, tree_height as char);
                max_height = tree_height;
                true
            } else {
                // println!("tree at {:?} height {} short", tree, tree_height as char);
                false
            }
        }
    }
}

fn run(input: &str) -> Result<(usize, usize)> {
    let forest = Forest::new(input.as_bytes());

    let edge_seen = trees_seen_from_edge(&forest);

    let mut maximum = 0;
    for x in 0..forest.width {
        for y in 0..forest.width {
            maximum = forest.view_score(Position { x, y }).max(maximum);
        }
    }

    Ok((edge_seen, maximum))
}

fn trees_seen_from_edge(forest: &Forest) -> usize {
    let mut seen = FxHashSet::default();
    seen.extend([
        Position { x: 0, y: 0 },
        Position {
            x: forest.width - 1,
            y: 0,
        },
        Position {
            x: 0,
            y: forest.width - 1,
        },
        Position {
            x: forest.width - 1,
            y: forest.width - 1,
        },
    ]);

    for x in 1..forest.width - 1 {
        // println!("SOUTH");
        let tree = Position { x, y: 0 };
        let initial_height = forest.get(tree).unwrap_or_default();
        // println!("tree at {:?} height {}", tree, initial_height as char);
        seen.insert(tree);
        seen.extend(
            forest
                .view_from(View::South, tree, b'9')
                .filter(forest.taller_than_priors(initial_height)),
        );

        // println!("NORTH");
        let tree = Position {
            x,
            y: forest.width - 1,
        };
        let initial_height = forest.get(tree).unwrap_or_default();
        // println!("tree at {:?} height {}", tree, initial_height as char);
        seen.insert(tree);
        seen.extend(
            forest
                .view_from(View::North, tree, b'9')
                .filter(forest.taller_than_priors(initial_height)),
        );
    }
    // dbg!(&seen);

    for y in 1..forest.width - 1 {
        // println!("WEST");
        let tree = Position { x: 0, y };
        let initial_height = forest.get(tree).unwrap_or_default();
        // println!("tree at {:?} height {}", tree, initial_height as char);
        seen.insert(tree);
        seen.extend(
            forest
                .view_from(View::West, tree, b'9')
                .filter(forest.taller_than_priors(initial_height)),
        );

        // println!("EAST");
        let tree = Position {
            x: forest.width - 1,
            y,
        };
        let initial_height = forest.get(tree).unwrap_or_default();
        // println!("tree at {:?} height {}", tree, initial_height as char);
        seen.insert(tree);
        seen.extend(
            forest
                .view_from(View::East, tree, b'9')
                .filter(forest.taller_than_priors(initial_height)),
        );
    }

    seen.len()
}

#[cfg(test)]
mod tests {
    use test_case::test_case;

    use super::*;

    #[test_case(include_str!("../../input/day08test") => matches Ok((21, 8)))]
    fn default_tests(input: &str) -> Result<(usize, usize)> {
        run(input)
    }

    #[test_case(include_str!("../../input/day08test"), Position { x: 2, y: 1 } => 4)]
    #[test_case(include_str!("../../input/day08test"), Position { x: 2, y: 3 } => 8)]
    fn trees_seen_tests(input: &str, tree: Position) -> usize {
        let forest = Forest::new(input.as_bytes());
        forest.view_score(tree)
    }
}
