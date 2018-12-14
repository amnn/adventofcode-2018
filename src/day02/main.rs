use std::collections::HashSet;
use std::env;
use std::fs::File;
use std::io::{self, BufRead, BufReader};
use std::vec::Vec;

struct BoxID {
    id: [u8; 26],
}

impl BoxID {
    fn new(rep: &str) -> BoxID {
        static LO: u8 = 'a' as u8;
        let mut id = [0u8; 26];
        for c in rep.chars() {
           id[(c as u8 - LO) as usize] += 1
        };
        BoxID{id}
    }

    fn has_n(&self, n: u8) -> bool {
        self.id.into_iter().find(|x| **x == n).is_some()
    }

    fn has_two(&self) -> bool {
        self.has_n(2)
    }

    fn has_three(&self) -> bool {
        self.has_n(3)
    }
}

fn similar_except_at(ix: usize, ids: &Vec<String>) -> Option<String> {
    let snip = |id: &str| -> String {
        let mut snipped = String::from(id);
        snipped.remove(ix);
        snipped
    };

    let mut unmatched = HashSet::new();
    for id in ids {
        if !unmatched.insert(snip(id)) {
            return Some(snip(id))
        }
    }

    None
}

fn main() -> io::Result<()> {
    let fname = env::args().nth(1).unwrap();
    let file = File::open(fname)?;
    let reader = BufReader::new(file);

    let mut ids: Vec<String> = vec![];

    let mut twos = 0u32;
    let mut threes = 0u32;
    for opt_line in reader.lines() {
        let line = opt_line?;
        let box_id = BoxID::new(&line);

        if box_id.has_two() {
            twos += 1;
        }
        if box_id.has_three() {
            threes += 1;
        }

        ids.push(line)
    }

    println!("Checksum: {}", twos * threes);

    let len = match ids.first() {
        Some(id) => id.len(),
        None => return Ok(()) /* No IDs */
    };

    for i in 0..len {
        match similar_except_at(i, &ids) {
            Some(partial) => {
                println!("Partial ID: {}", partial);
                break
            },
            None => continue,
        }
    }

    Ok(())
}
