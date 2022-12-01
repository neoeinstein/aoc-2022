use color_eyre::Result;
use itertools::Itertools;
use std::{io, iter};

fn main() -> Result<()> {
    let max = io::stdin()
        .lines()
        .batching(|it| {
            let first = it.next()?;
            Some(
                iter::once(first)
                    .chain(it)
                    .take_while(|l| l.as_ref().map(|l| !l.is_empty()).unwrap_or_default())
                    .try_fold(0u64, |acc, l| -> Result<u64> {
                        let line = l?;
                        let next = acc + line.parse::<u64>()?;
                        Ok(next)
                    }),
            )
        })
        .try_fold((0, 0, 0), |acc, v| -> Result<_> {
            let v = v?;
            let next = if v > acc.0 {
                (v, acc.0, acc.1)
            } else if v > acc.1 {
                (acc.0, v, acc.1)
            } else if v > acc.2 {
                (acc.0, acc.1, v)
            } else {
                acc
            };
            Ok(next)
        })?;

    println!("{max:?}: {}", max.0 + max.1 + max.2);

    Ok(())
}
