use std::{env, fmt, fs, io, io::Read};

use color_eyre::Result;
use itertools::Itertools;
use petgraph::{graph::EdgeReference, prelude::*};

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
    let mut grid = DiGraph::new();
    let mut start = None;
    let mut end = None;
    let mut end_pos = None;
    let mut width = 0;
    let mut indexes = Vec::new();
    for (row, line) in input.lines().enumerate() {
        let mut current_row = Vec::new();
        width = line.len();
        for (col, mut height) in line.as_bytes().iter().copied().enumerate() {
            let position = Position::new(col, row);
            let node_idx = if height == b'S' {
                height = b'a';
                let node_idx = grid.add_node(Node { position, height });
                start = Some(node_idx);
                node_idx
            } else if height == b'E' {
                height = b'z';
                let node_idx = grid.add_node(Node { position, height });
                end = Some(node_idx);
                end_pos = Some(position);
                node_idx
            } else {
                grid.add_node(Node { position, height })
            };
            current_row.push(node_idx);
        }
        indexes.extend(current_row);
    }

    for (idx, node_idx) in indexes.iter().copied().enumerate() {
        let me = grid[node_idx];
        if let Some(&south_idx) = indexes.get(idx + width) {
            let south = grid[south_idx];
            if south.height <= me.height + 1 {
                grid.add_edge(node_idx, south_idx, 1);
            }
            if me.height <= south.height + 1 {
                grid.add_edge(south_idx, node_idx, 1);
            }
        }
        if (idx + 1) % width > 0 {
            if let Some(&east_idx) = indexes.get(idx + 1) {
                let east = grid[east_idx];
                if east.height <= me.height + 1 {
                    grid.add_edge(node_idx, east_idx, 1);
                }
                if me.height <= east.height + 1 {
                    grid.add_edge(east_idx, node_idx, 1);
                }
            }
        }
    }

    let start = start.unwrap();
    let end = end.unwrap();
    let end_pos = end_pos.unwrap();
    let is_goal = |node_idx| node_idx == end;
    let edge_cost = |_| 1;
    let estimate_cost = |node_idx: NodeIndex| {
        let cur = grid[node_idx];
        let cur_pos = cur.position;
        (end_pos.x.abs_diff(cur_pos.x))
            .min(end_pos.y.abs_diff(cur_pos.y))
            .max((b'z' - cur.height) as usize)
    };

    let (steps_from_origin, path) =
        petgraph::algo::astar(&grid, start, is_goal, edge_cost, estimate_cost).expect("valid path");

    let mut new_grid = grid.clone();
    new_grid.reverse();
    let is_goal = |node_idx: NodeIndex| new_grid[node_idx].height == b'a';
    let estimate_cost = |node_idx: NodeIndex| {
        let cur = new_grid[node_idx];
        (b'z' - cur.height) as usize
    };
    let (steps_to_flat, rev_path) =
        petgraph::algo::astar(&new_grid, end, is_goal, edge_cost, estimate_cost)
            .expect("valid path");

    let edge_attrs = |_, edge_ref: EdgeReference<_>| {
        let mut attrs = String::new();
        if path
            .iter()
            .copied()
            .tuple_windows::<(_, _)>()
            .contains(&(edge_ref.source(), edge_ref.target()))
        {
            attrs.push_str("color = \"#009900\"");
        } else if rev_path
            .iter()
            .copied()
            .tuple_windows::<(_, _)>()
            .contains(&(edge_ref.source(), edge_ref.target()))
        {
            attrs.push_str("color = \"#CC0099\"");
        }
        attrs
    };
    let node_attrs = |_, node_ref: (NodeIndex, &Node)| {
        let scalar = ((node_ref.1.height - b'a') as f64) / 25. * 0.75;
        let mut attrs =
            format!("color = \"{scalar:0.3} 0.75 1.0\" fontcolor = \"{scalar:0.3} 0.75 1.0\"");
        if path.contains(&node_ref.0) {
            attrs.push_str(" fillcolor = \"#112211\" style = filled");
        }
        attrs
    };

    let dot = petgraph::dot::Dot::with_attr_getters(
        &grid,
        &[petgraph::dot::Config::EdgeNoLabel],
        &edge_attrs,
        &node_attrs,
    );

    eprintln!("{dot}");

    Ok((steps_from_origin, steps_to_flat))
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct Node {
    position: Position,
    height: u8,
}

impl fmt::Display for Node {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "({}, {})\n{}",
            self.position.x, self.position.y, self.height as char
        )
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct Position {
    x: usize,
    y: usize,
}

impl Position {
    const fn new(x: usize, y: usize) -> Self {
        Self { x, y }
    }
}

#[cfg(test)]
mod tests {
    use test_case::test_case;

    use super::*;

    #[test_case(include_str!("../../input/day12test") => matches Ok((31, 29)))]
    fn default_tests(input: &str) -> Result<(usize, usize)> {
        run(input)
    }
}
