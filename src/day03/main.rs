#[macro_use] extern crate lib;
#[macro_use] extern crate scan_fmt;

use std::collections::{BTreeSet, HashMap, HashSet};
use std::collections::btree_set;
use std::env;
use std::fs::File;
use std::io::{self, BufRead, BufReader};

input! {
    #["#{d} @ {d},{d}: {d}x{d}"; ""]
    struct Rect {
        id: usize,
        left: usize, top: usize,
        width: usize, height: usize
    }
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
enum Edge { Lead, Trail }

#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
struct Boundary { pos: usize, id: usize, edge: Edge }

impl Rect {
    fn vert(&self) -> (Boundary, Boundary) {
        (
            Boundary{ pos: self.top, id: self.id, edge: Edge::Lead },
            Boundary{ pos: self.top + self.height, id: self.id, edge: Edge::Trail }
        )
    }

    fn horiz(&self) -> (Boundary, Boundary) {
        (
            Boundary{ pos: self.left, id: self.id, edge: Edge::Lead },
            Boundary{ pos: self.left + self.width, id: self.id, edge: Edge::Trail }
        )
    }

    fn is_overlapping(&self, other: &Rect) -> bool {
        fn llr(a: &Rect, b: &Rect) -> bool {
            a.left <= b.left && b.left < a.left + a.width
        }

        fn ttb(a: &Rect, b: &Rect) -> bool {
            a.top <= b.top && b.top < a.top + a.height
        }

        (llr(&self, &other) || llr(&other, &self))
            && (ttb(&self, &other) || ttb(&other, &self))
    }
}

type Input = HashMap<usize, Rect>;
fn parse_input() -> io::Result<Input> {
    let fname = env::args().nth(1).unwrap();
    let file = File::open(fname)?;
    let reader = BufReader::new(file);

    let mut rects = HashMap::new();
    for line in reader.lines() {
        let r = Rect::new(&line?)?;
        rects.insert(r.id, r);
    };

    Ok(rects)
}

struct ScanLine { boundaries: BTreeSet<Boundary> }

impl ScanLine {
    fn new() -> ScanLine {
        ScanLine { boundaries: BTreeSet::new() }
    }

    fn vert(rects: &Input) -> ScanLine {
        let mut boundaries = BTreeSet::new();
        for r in rects.values() {
            let (t, b) = r.vert();
            boundaries.insert(t);
            boundaries.insert(b);
        };
        ScanLine { boundaries }
    }

    fn iter(&self) -> btree_set::Iter<Boundary> {
        self.boundaries.iter()
    }

    fn horiz_insert(&mut self, r: &Rect) {
        let (l, r) = r.horiz();
        self.boundaries.insert(l);
        self.boundaries.insert(r);
    }

    fn horiz_remove(&mut self, r: &Rect) {
        let (l, r) = r.horiz();
        self.boundaries.remove(&l);
        self.boundaries.remove(&r);
    }

    fn overlap(&self) -> usize {
        let mut overlap = 0;
        let mut nest = 0;
        let mut start = 0;
        for b in &self.boundaries {
            match b.edge {
                Edge::Lead => {
                    nest += 1;
                    if nest == 2 {
                        start = b.pos;
                    }
                },
                Edge::Trail => {
                    nest -= 1;
                    if nest == 1 {
                        overlap += b.pos - start;
                    }
                },
            }
        };
        overlap
    }

    fn is_empty(&self) -> bool {
        self.boundaries.is_empty()
    }
}

fn main() -> io::Result<()> {
    let rects = parse_input()?;

    /* Part 1 */

    let vscan = ScanLine::vert(&rects);

    let mut overlap = 0;
    let mut prev_row = 0;
    let mut hscan = ScanLine::new();

    let mut vit = vscan.iter().peekable();
    while let Some(&next) = vit.peek() {
        // Extrude
        overlap += hscan.overlap() * (next.pos - prev_row);
        prev_row = next.pos;

        // Update
        while let Some(&b) = vit.peek() {
            if b.pos != prev_row {
                break;
            } else {
                vit.next();
            }

            let ref r = rects[&b.id];
            match b.edge {
                Edge::Lead  => hscan.horiz_insert(r),
                Edge::Trail => hscan.horiz_remove(r),
            }
        }
    }

    assert!(hscan.is_empty(), "Last row should be empty");
    println!("Overlapping squares: {}", overlap);

    /* Part 2 */
    let mut candidates: HashSet<usize> = rects.keys().cloned().collect();

    for a in rects.values() {
        if !candidates.contains(&a.id) {
            continue
        }

        for b in rects.values() {
            if a.id != b.id && a.is_overlapping(b) {
                candidates.remove(&a.id);
                candidates.remove(&b.id);
                break
            }
        }
    }

    for c in candidates {
        println!("No overlaps: {}", c);
    }

    Ok(())
}

