use std::{io, io::Read, str};

use color_eyre::Result;

const FORCE_SLOW_MODE: bool = false;

fn main() -> Result<()> {
    let mut input = String::new();
    io::stdin().read_to_string(&mut input)?;

    let (marker, idx) = find_marker::<4>(&input);
    println!("marker: {} at {}", marker, idx);

    let (marker, idx) = find_marker::<14>(&input);
    println!("marker: {} at {}", marker, idx);

    Ok(())
}

fn find_marker<const N: usize>(input: &str) -> (&str, usize) {
    let bytes = input.as_bytes();
    let mut window: [u8; N] = std::array::from_fn(|i| bytes[i]);
    let mut idx = N;
    let mut cmps = 0;

    loop {
        let (fill, cmp) = calculate_fill(&window);
        cmps += cmp;
        if fill == 0 {
            break;
        }
        window.copy_within(fill.., 0);
        window[N - fill..].copy_from_slice(&bytes[idx..][..fill]);
        idx += fill;
    }

    println!("Comparisons: {}", cmps);

    (&input[idx - N..][..N], idx)
}

fn calculate_fill(window: &[u8]) -> (usize, usize) {
    let mut fill = 0;
    let mut cmp = 0;
    'outer: for (idx, val) in window[..window.len() - 1].iter().copied().enumerate().rev() {
        for &test in &window[idx + 1..] {
            cmp += 1;
            if val == test {
                fill = if FORCE_SLOW_MODE { 1 } else { idx + 1 };
                break 'outer;
            }
        }
    }

    (fill, cmp)
}
