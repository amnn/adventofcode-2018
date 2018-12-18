use std::env;
use std::fs::File;
use std::io::{self, BufRead, BufReader};
use std::str;
use std::string::ToString;

struct Polymer {
    units: Box<[u8]>,
    cursor: usize,
    gap: usize
}

impl Polymer {
    fn new(units: String) -> Polymer {
        Polymer {
            units: units.into_boxed_str().into_boxed_bytes(),
            cursor: 0,
            gap: 0,
        }
    }

    fn len(&self) -> usize {
        assert!(self.units.len() >= self.gap + self.cursor);
        self.units.len() - self.gap
    }

    fn react(&mut self) -> bool {
        if self.cursor == 0 {
            return false;
        }

        if self.cursor >= self.len() {
            return false;
        }

        let l = self.units[self.cursor - 1];
        let r = self.units[self.cursor + self.gap];

        let same_char = l.to_ascii_lowercase() == r.to_ascii_lowercase();
        let diff_case = l.is_ascii_lowercase() != r.is_ascii_lowercase();
        let reactive = same_char && diff_case;

        if reactive {
            self.cursor -= 1;
            self.gap += 2;
        }

        reactive
    }

    fn inc(&mut self) -> bool {
        if self.cursor >= self.len() {
            return false
        }

        let next = self.units[self.cursor + self.gap];
        self.units[self.cursor] = next;
        self.cursor += 1;

        true
    }
}

impl ToString for Polymer {
    fn to_string(&self) -> String {
        let mut rep = String::with_capacity(self.len());

        let lo = self.cursor;
        let hi = self.cursor + self.gap;
        let end = self.units.len();

        unsafe {
            rep.push_str(str::from_utf8_unchecked(&self.units[0..lo]));
            rep.push_str(str::from_utf8_unchecked(&self.units[hi..end]));
        }

        rep
    }
}

fn react(mut poly: Polymer) -> usize {
    loop {
        while poly.react() {}
        if !poly.inc() {
            break;
        }
    }

    poly.len()
}

fn main() -> io::Result<()> {
    let fname = env::args().nth(1).unwrap();

    let file = File::open(fname)?;
    let reader = BufReader::new(file);

    let input = reader.lines().next().unwrap()?;

    {
        let polymer = Polymer::new(input.clone());
        println!("Part 1: {}", react(polymer));
    }

    {
        let smallest = (('a' as u8) ..= ('z' as u8))
            .map(|to_remove| {
                Polymer::new(
                    input.bytes().filter_map(|c| {
                        if c.to_ascii_lowercase() == to_remove {
                            return None
                        }

                        Some(c as char)
                    }).collect())
            })
            .map(react)
            .min()
            .unwrap();

        println!("Part 2: {}", smallest);
    }

    Ok(())
}
