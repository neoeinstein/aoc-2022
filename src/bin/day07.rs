use core::fmt;
use std::{env, fs, io, io::Read};

use color_eyre::{Report, Result};
use nom::{
    branch::alt,
    bytes::complete::tag,
    character::complete::{digit1, line_ending, not_line_ending},
    combinator::{eof, map, map_res, peek, value},
    multi::many_till,
    sequence::{delimited, preceded, separated_pair, terminated},
    Finish, IResult,
};
use petgraph::{algo::toposort, prelude::*, stable_graph::NodeIndex, visit::EdgeRef};

fn main() -> Result<()> {
    color_eyre::install()?;

    let input = if let Some(path) = env::args_os().nth(1) {
        fs::read_to_string(path)?
    } else {
        let mut input = String::new();
        io::stdin().read_to_string(&mut input)?;
        input
    };

    let (total_size, to_delete) = run(&input)?;

    println!("total_size: {total_size}");
    println!("to delete: {to_delete}");

    Ok(())
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Node {
    Directory,
    File { size: usize },
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum NodeWithDirSize {
    Directory { size: usize },
    File { size: usize },
}

impl NodeWithDirSize {
    fn size(&self) -> usize {
        match self {
            Self::Directory { size } | Self::File { size } => *size,
        }
    }
}

impl fmt::Display for NodeWithDirSize {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.size())
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct Edge<'a>(&'a str);

impl fmt::Display for Edge<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.0)
    }
}

fn run(mut input: &str) -> Result<(usize, usize)> {
    let mut graph = petgraph::graph::DiGraph::<Node, Edge>::new();
    let root_node = graph.add_node(Node::Directory);

    let mut current_path = Vec::new();
    let mut current_node = root_node;

    while let (rest, Some(cmd)) = parse_command_or_end(input)
        .map_err(|e| e.to_owned())
        .finish()?
    {
        input = match cmd {
            Command::ChangeDirectory("..") => {
                current_path.pop();
                current_node = graph
                    .neighbors_directed(current_node, petgraph::Direction::Incoming)
                    .next()
                    .ok_or_else(|| Report::msg("attempt to change directory up from root"))?;
                rest
            }
            Command::ChangeDirectory("/") => {
                current_path.clear();
                current_node = root_node;
                rest
            }
            Command::ChangeDirectory(name) => {
                current_node = add_or_get_directory(&mut graph, current_node, name);
                current_path.push(name);
                rest
            }
            Command::List => {
                let (rest, resp) = parse_list_response(rest)
                    .map_err(|e| e.to_owned())
                    .finish()?;

                for line in resp {
                    match line {
                        ListResponseLine::Directory(name) => {
                            add_or_get_directory(&mut graph, current_node, name)
                        }
                        ListResponseLine::File(name, size) => {
                            add_child(&mut graph, current_node, name, Node::File { size })
                        }
                    };
                }

                rest
            }
        }
    }

    let graph = calculate_sizes(&graph);
    let part1_sum = graph
        .node_weights()
        .filter_map(|n| match n {
            NodeWithDirSize::Directory { size } if *size <= 100000 => Some(*size),
            _ => None,
        })
        .sum();

    println!("{}", petgraph::dot::Dot::new(&graph));

    let total_size = graph
        .node_weight(root_node)
        .map(|n| n.size())
        .unwrap_or_default();
    let capacity = 70000000;
    let remaining = capacity - total_size;
    let need = 30000000;
    let to_free = need - remaining;

    let part2_ans = graph
        .node_weights()
        .filter_map(|n| match n {
            NodeWithDirSize::Directory { size } if *size >= to_free => Some(*size),
            _ => None,
        })
        .min()
        .unwrap_or_default();

    Ok((part1_sum, part2_ans))
}

fn calculate_sizes<'e>(graph: &DiGraph<Node, Edge<'e>>) -> DiGraph<NodeWithDirSize, Edge<'e>> {
    let mut new_graph = graph.map(
        |_, node| match node {
            Node::Directory => NodeWithDirSize::Directory { size: 0 },
            Node::File { size } => NodeWithDirSize::File { size: *size },
        },
        |_, e| *e,
    );
    for node in toposort(&new_graph, None).expect("file system is acyclic") {
        if let Some(size) = match new_graph.node_weight(node).unwrap() {
            NodeWithDirSize::Directory { size } if *size == 0 => {
                Some(calculate_dir_size(&new_graph, node))
            }
            _ => None,
        } {
            *new_graph.node_weight_mut(node).unwrap() = NodeWithDirSize::Directory { size };
        }
    }

    new_graph
}

fn calculate_dir_size(
    graph: &DiGraph<NodeWithDirSize, Edge<'_>>,
    current_node: NodeIndex,
) -> usize {
    graph
        .edges_directed(current_node, Direction::Outgoing)
        .map(|e| match graph.node_weight(e.target()).unwrap() {
            NodeWithDirSize::Directory { size: 0 } => calculate_dir_size(graph, e.target()),
            NodeWithDirSize::Directory { size } => *size,
            NodeWithDirSize::File { size } => *size,
        })
        .sum()
}

fn add_or_get_directory<'a>(
    graph: &mut DiGraph<Node, Edge<'a>>,
    current_node: NodeIndex,
    name: &'a str,
) -> NodeIndex {
    if let Some(edge) = graph
        .edges_directed(current_node, petgraph::Direction::Outgoing)
        .find(|e| e.weight().0 == name)
    {
        edge.target()
    } else {
        add_child(graph, current_node, name, Node::Directory)
    }
}

fn add_child<'a>(
    graph: &mut DiGraph<Node, Edge<'a>>,
    current_node: NodeIndex,
    name: &'a str,
    node: Node,
) -> NodeIndex {
    let node = graph.add_node(node);
    graph.add_edge(current_node, node, Edge(name));
    node
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Command<'a> {
    ChangeDirectory(&'a str),
    List,
}

fn parse_command(s: &str) -> IResult<&str, Command> {
    delimited(
        tag("$ "),
        alt((
            value(Command::List, tag("ls")),
            map(
                preceded(tag("cd "), not_line_ending),
                Command::ChangeDirectory,
            ),
        )),
        line_ending,
    )(s)
}

fn parse_command_or_end(s: &str) -> IResult<&str, Option<Command>> {
    alt((value(None, eof), map(parse_command, Some)))(s)
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum ListResponseLine<'a> {
    Directory(&'a str),
    File(&'a str, usize),
}

fn parse_list_line(s: &str) -> IResult<&str, ListResponseLine> {
    terminated(
        alt((
            map(
                separated_pair(map_res(digit1, str::parse), tag(" "), not_line_ending),
                |(size, name)| ListResponseLine::File(name, size),
            ),
            map(
                preceded(tag("dir "), not_line_ending),
                ListResponseLine::Directory,
            ),
        )),
        line_ending,
    )(s)
}

#[derive(Clone, Default, Debug, PartialEq, Eq)]
struct DirectoryContents<'a> {
    child_dirs: Vec<&'a str>,
    child_files: Vec<(&'a str, usize)>,
}

fn parse_list_response(s: &str) -> IResult<&str, Vec<ListResponseLine>> {
    map(
        many_till(parse_list_line, peek(alt((tag("$"), eof)))),
        |(list, _)| list,
    )(s)
}

#[cfg(test)]
mod tests {
    use test_case::test_case;

    use super::*;

    #[test_case(include_str!("../../input/day07test") => matches Ok((48381165, 24933642)))]
    fn default_tests(input: &str) -> Result<(usize, usize)> {
        run(input)
    }
}
