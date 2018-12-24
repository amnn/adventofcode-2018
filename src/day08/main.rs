use std::env;
use std::fs::File;
use std::io::{self, BufRead, BufReader};

fn sum_metadata<I: Iterator<Item = usize>>(iter: &mut I) -> Option<usize> {
    let children = iter.next()?;
    let metadata = iter.next()?;

    let mut sum = 0;
    for _ in 0..children {
        sum += sum_metadata(iter)?;
    }

    for _ in 0..metadata {
        sum += iter.next()?;
    }

    Some(sum)
}

fn value<I: Iterator<Item = usize>>(iter: &mut I) -> Option<usize> {
    let children = iter.next()?;
    let metadata = iter.next()?;

    let child_vals = (0..children)
        .into_iter()
        .map(|_| value(iter))
        .collect::<Option<Vec<_>>>()?;

    let meta_vals = (0..metadata)
        .into_iter()
        .map(|_| iter.next())
        .collect::<Option<Vec<_>>>()?;

    if children == 0 {
        Some(meta_vals.iter().sum())
    } else {
        Some(meta_vals.iter().map(|&ix| {
            if ix == 0 || ix > children {
                0
            } else {
                child_vals[ix - 1]
            }
        }).sum())
    }
}


fn main() -> io::Result<()> {
    let fname = env::args().nth(1).unwrap();
    let file = File::open(fname)?;
    let mut reader = BufReader::new(file);

    let mut input = String::new();
    reader.read_line(&mut input)?;

    let nums = input.split_whitespace()
        .map(|token| token.parse::<usize>())
        .collect::<Result<Vec<_>, _>>()
        .unwrap();

    println!("Part 1: {:?}", sum_metadata(&mut nums.iter().cloned()));
    println!("Part 2: {:?}", value(&mut nums.iter().cloned()));

    Ok(())
}
