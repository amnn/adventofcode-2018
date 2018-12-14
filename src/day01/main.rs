use std::collections::HashSet;
use std::env;
use std::fs::File;
use std::io::{self, BufRead, BufReader};

fn main() -> io::Result<()> {
    let fname = env::args().nth(1).unwrap();

    let mut frequency: i32 = 0;
    let mut visited = HashSet::new();

    visited.insert(frequency);

    loop {
        let file = File::open(&fname)?;
        let reader = BufReader::new(file);
        for line in reader.lines() {
            let delta: i32 = line.unwrap().parse().unwrap();
            frequency += delta;

            if !visited.insert(frequency) {
                println!("Repeated: {}", frequency);
                return Ok(())
            }
        };
    }
}
