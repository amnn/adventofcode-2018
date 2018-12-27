#[macro_use] extern crate lib;
#[macro_use] extern crate scan_fmt;

use std::collections::HashMap;
use std::env;
use std::fs::File;
use std::io::{self, BufRead, BufReader};
use std::iter::Iterator;

input! {
    #["initial state: {}"; ""]
    struct InitialState {
        init: String
    }
}

input! {
    #["{} => {[.#]}"; ""]
    struct Transition {
        before: String,
        after: char
    }
}

fn is_live(c: char) -> bool {
    match c {
        '#' => true,
        '.' => false,
        _   => panic!("Unrecognised character"),
    }
}

impl Transition {
    fn update(&self, trn: &mut TrnMap) {
        let key = self.before.chars().fold(0, |mut k, c| {
            k <<= 1;
            if is_live(c) {
                k |= 1;
            };
            k
        });

        trn[key] = is_live(self.after);
    }
}

#[derive(Clone)]
struct BitVec {
    len: usize,
    storage: Vec<usize>,
}

static INDEX_SF: usize = std::mem::size_of::<usize>() * 8;
fn to_storage_size(bit_size: usize) -> usize {
    if bit_size == 0 {
        0
    } else {
        (bit_size - 1) / INDEX_SF + 1
    }
}

impl BitVec {
    fn new(len: usize) -> BitVec {
        BitVec { len, storage: vec![0; to_storage_size(len)] }
    }

    fn len(&self) -> usize {
        self.len
    }

    fn get(&self, i: usize) -> usize {
        assert!(i < self.len);

        let wrd_ix = i / INDEX_SF;
        let bit_ix = i % INDEX_SF;
        let word = self.storage[wrd_ix];

        (word >> bit_ix) & 0x1
    }

    fn set(&mut self, i: usize) {
        assert!(i < self.len);

        let wrd_ix = i / INDEX_SF;
        let bit_ix = i % INDEX_SF;
        let word = &mut self.storage[wrd_ix];

        *word |= 1 << bit_ix;
    }

    fn clear(&mut self, i: usize) {
        assert!(i < self.len);

        let wrd_ix = i / INDEX_SF;
        let bit_ix = i % INDEX_SF;
        let word = &mut self.storage[wrd_ix];

        *word &= !(1 << bit_ix);
    }

    fn resize(&mut self, new_len: usize) {
        self.len = new_len;
        self.storage.resize(to_storage_size(new_len), 0)
    }

    fn grow(&mut self, amt: usize) {
        let new_len = self.len() + amt;
        self.resize(new_len);
    }

    fn find_first_set(&self) -> Option<usize> {
        self.storage.iter().enumerate().find_map(|(i, &word)| {
            if word == 0 {
                None
            } else {
                Some(i * INDEX_SF + word.trailing_zeros() as usize)
            }
        })
    }

    fn find_last_set(&self) -> Option<usize> {
        self.storage.iter().enumerate().rev().find_map(|(i, &word)| {
            if word == 0 {
                None
            } else {
                Some((i + 1) * INDEX_SF - 1 - word.leading_zeros() as usize)
            }
        })
    }

    /// Remove the first `n` elements from the bit vector.
    fn shift(&mut self, n: usize) {
        assert!(n < self.len);
        self.len -= n;

        let wrd_ix = n / INDEX_SF;
        let bit_ix = n % INDEX_SF;

        self.storage.drain(0 .. wrd_ix);

        if bit_ix == 0 {
            return
        }

        let mask = (1 << bit_ix) - 1;
        let shift = INDEX_SF - bit_ix;

        let mut sink: usize = 0;
        let mut carry: &mut usize = &mut sink;
        for word in &mut self.storage {
            *carry |= (*word & mask) << shift;
            *word >>= bit_ix;
            carry = word;
        }
    }

    fn iter<'a>(&'a self) -> impl Iterator<Item = bool> + 'a {
        (0 .. self.len).map(move |i| self.get(i) == 1)
    }
}

struct Pots {
    offset: isize,
    liveness: BitVec,
}

#[derive(Debug)]
struct Config {
    offset: isize,
    generation: usize,
}

type TrnMap = [bool; 32];
type CfgMap = HashMap<Vec<bool>, Config>;

static PAD: usize = 2;

impl Pots {
    fn new(init: &str) -> Pots {
        let size = init.len();
        let mut liveness = BitVec::new(size);

        for (i, c) in init.chars().enumerate() {
            if is_live(c) {
                liveness.set(i);
            }
        }

        Pots { offset: 0, liveness }
    }

    fn next(&mut self, trn: &TrnMap) {
        // New offset and size
        self.offset -= PAD as isize;
        self.liveness.grow(2 * PAD);

        let mut key: usize = 0;
        for i in 0 .. self.liveness.len() {
            key <<= 1;
            key |= self.liveness.get(i);
            key &= 0b11111;

            if trn[key] {
                self.liveness.set(i);
            } else {
                self.liveness.clear(i);
            }
        }
    }

    fn travel(&mut self, delta: isize) {
        self.offset += delta;
    }

    /// Normalise the representation of the live pots by shifting the offset
    /// to the first live pot.
    fn normalize(&mut self) {
        if let Some(i) = self.liveness.find_first_set() {
            self.liveness.shift(i);
            self.offset += i as isize;
        }

        if let Some(j) = self.liveness.find_last_set() {
            self.liveness.resize(j + 1);
        }
    }

    fn memoize(&self, generation: usize, configs: &mut CfgMap) -> Option<Config> {
        let &Pots { offset, .. } = self;

        let patt: Vec<_> = self.liveness.iter().collect();
        let config = Config { offset, generation };

        configs.insert(patt, config)
    }

    fn live_pot_sum(&self) -> isize {
        let mut sum = 0;

        for (i, is_live) in self.liveness.iter().enumerate() {
            if is_live {
                sum += self.offset + i as isize;
            }
        }

        sum
    }

    fn print_range(&self, from: isize, to: isize) {
        let mut i = from;

        let before = to.min(self.offset);
        while i < before {
            print!(".");
            i += 1
        }

        let after = to.min(self.offset + self.liveness.len() as isize);
        while i < after {
            let ix = (i - self.offset) as usize;
            let is_live = self.liveness.get(ix) == 1;
            let chr = if is_live { '#' } else { '.' };
            print!("{}", chr);
            i += 1;
        }

        while i < to {
            print!(".");
            i += 1;
        }

        println!(" [{}, +{}]", self.offset, self.liveness.len());
    }

    fn print(&self) {
        let from = self.offset;
        let to = self.offset + self.liveness.len() as isize;
        self.print_range(from, to);
    }
}

fn main() -> io::Result<()> {
    let (init, trn) = {
        let fname = env::args().nth(1).unwrap();
        let file = File::open(fname)?;
        let mut reader = BufReader::new(file);

        let mut buf = String::new();
        reader.read_line(&mut buf)?;
        let init_parsed = InitialState::new(buf.trim())?;

        buf.clear();
        reader.read_line(&mut buf)?;
        assert_eq!(
            buf.trim().len(), 0,
            "Expected empty line between initial state and transition map");

        let mut trn = [false; 32];
        for line in reader.lines() {
            let entry = Transition::new(&line?)?;
            entry.update(&mut trn);
        }

        (init_parsed.init, trn)
    };

    assert!(!trn[0], "Cannot bound pot growth");

    let mut pots = Pots::new(&init);
    let gens: usize = 50_000_000_000;

    let mut prev_configs = HashMap::new();

    let mut i = 0;
    while i < gens {
        pots.next(&trn);
        i += 1;

        if i % 97 == 0 {
            pots.normalize();
            if let Some(prev) = pots.memoize(i, &mut prev_configs) {
                // We have seen this configuration before.  Use this fact to fast
                // forward the simulation.

                let time_left = gens - i;
                let time_delta = i - prev.generation;
                let skips = time_left / time_delta;
                let drift = pots.offset - prev.offset;

                println!("Drift detected: {} -> {} by {}", prev.generation, i, drift);
                pots.print();

                pots.travel(drift * skips as isize);
                i += time_delta * skips;
                println!("Fast forwarded to {}", i);
            }
        }
    }

    println!("Live: {}", pots.live_pot_sum());

    Ok(())
}


#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn find_set() {
        let mut bv = BitVec::new(10);
        assert_eq!(None, bv.find_first_set());
        assert_eq!(None, bv.find_last_set());

        bv.set(5);
        assert_eq!(Some(5), bv.find_first_set());
        assert_eq!(Some(5), bv.find_last_set());

        bv.set(3);
        assert_eq!(Some(3), bv.find_first_set());
        assert_eq!(Some(5), bv.find_last_set());

        bv.set(1);
        assert_eq!(Some(1), bv.find_first_set());
        assert_eq!(Some(5), bv.find_last_set());

        bv.clear(3);
        assert_eq!(Some(1), bv.find_first_set());
        assert_eq!(Some(5), bv.find_last_set());

        bv.clear(5);
        assert_eq!(Some(1), bv.find_first_set());
        assert_eq!(Some(1), bv.find_last_set());
    }

    #[test]
    fn first_set_multiword() {
        let i = INDEX_SF * 2 + 5;
        let j = INDEX_SF * 3 + 6;

        let mut bv = BitVec::new(INDEX_SF * 4);
        assert_eq!(None, bv.find_first_set());
        assert_eq!(None, bv.find_last_set());

        bv.set(j);
        assert_eq!(Some(j), bv.find_first_set());
        assert_eq!(Some(j), bv.find_last_set());

        bv.set(i);
        assert_eq!(Some(i), bv.find_first_set());
        assert_eq!(Some(j), bv.find_last_set());

        bv.clear(j);
        assert_eq!(Some(i), bv.find_first_set());
        assert_eq!(Some(i), bv.find_last_set());

        bv.clear(i);
        assert_eq!(None, bv.find_first_set());
        assert_eq!(None, bv.find_last_set());
    }

    #[test]
    fn shift() {
        let mut bv = BitVec::new(4);
        bv.set(1);
        bv.set(3);

        bv.shift(0);
        assert_eq!(bv.iter().collect::<Vec<_>>(), vec![false, true, false, true]);

        bv.shift(1);
        assert_eq!(bv.iter().collect::<Vec<_>>(), vec![true, false, true]);

        bv.shift(2);
        assert_eq!(bv.iter().collect::<Vec<_>>(), vec![true]);
    }

    #[test]
    fn shift_multiword() {
        let i = INDEX_SF * 2 + 5;
        let j = INDEX_SF * 3 + 6;

        let mut bv = BitVec::new(INDEX_SF * 4);
        let mut expected = vec![false; INDEX_SF * 4];

        bv.set(i); expected[i] = true;
        bv.set(j); expected[j] = true;

        bv.shift(0);
        assert_eq!(bv.iter().collect::<Vec<_>>(), expected);

        bv.shift(1); expected.drain(0 .. 1);
        assert_eq!(bv.iter().collect::<Vec<_>>(), expected);

        bv.shift(2); expected.drain(0 .. 2);
        assert_eq!(bv.iter().collect::<Vec<_>>(), expected);
    }
}
