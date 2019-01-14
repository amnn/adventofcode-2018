#[macro_use] extern crate lib;
#[macro_use] extern crate scan_fmt;

use std::collections::{HashMap, HashSet};
use std::env;
use std::fs::File;
use std::io::{self, BufRead, BufReader};
use std::iter::Peekable;

type RegisterFile = [usize; 4];
type Translation = [OpCode; 16];
type PossibleOps = HashMap<usize, HashSet<OpCode>>;

#[derive(Copy, Clone, Debug, Hash, Eq, PartialEq)]
enum OpCode {
    ADDR, ADDI,
    MULR, MULI,
    BANR, BANI,
    BORR, BORI,
    SETR, SETI,
    GTIR, GTRI, GTRR,
    EQIR, EQRI, EQRR,
}
use OpCode::*;

static ALL_OPS: [OpCode; 16] = [
    ADDR, ADDI,
    MULR, MULI,
    BANR, BANI,
    BORR, BORI,
    SETR, SETI,
    GTIR, GTRI, GTRR,
    EQIR, EQRI, EQRR,
];

fn exec(op: OpCode, in1: usize, in2: usize, out: usize, r: &mut RegisterFile) {
    r[out] = match op {
        ADDR => r[in1] + r[in2],
        ADDI => r[in1] + in2,

        MULR => r[in1] * r[in2],
        MULI => r[in1] * in2,

        BANR => r[in1] & r[in2],
        BANI => r[in1] & in2,

        BORR => r[in1] | r[in2],
        BORI => r[in1] | in2,

        SETR => r[in1],
        SETI => in1,

        GTIR => if in1 > r[in2] { 1 } else { 0 },
        GTRI => if r[in1] > in2 { 1 } else { 0 },
        GTRR => if r[in1] > r[in2] { 1 } else { 0 },

        EQIR => if in1 == r[in2] { 1 } else { 0 },
        EQRI => if r[in1] == in2 { 1 } else { 0 },
        EQRR => if r[in1] == r[in2] { 1 } else { 0 },
    }
}

fn eval<I: Iterator<Item = Insn>>(trn: &Translation, insns: I) -> usize {
    let mut regs = [0; 4];
    for i in insns {
        exec(trn[i.op], i.in1, i.in2, i.out, &mut regs);
    }

    regs[0]
}

input! {
    #["Before: [{d}, {d}, {d}, {d}]"; ""]
    struct Before {
        r0: usize, r1: usize, r2: usize, r3: usize
    }
}

input! {
    #["After:  [{d}, {d}, {d}, {d}]"; ""]
    struct After {
        r0: usize, r1: usize, r2: usize, r3: usize
    }
}

input! {
    #["{d} {d} {d} {d}"; ""]
    struct Insn {
        op: usize, in1: usize, in2: usize, out: usize
    }
}

struct Sample {
    before: Before,
    insn:   Insn,
    after:  After,
}

impl Sample {
    fn new(
            b: io::Result<String>,
            i: io::Result<String>,
            a: io::Result<String>,
        ) -> io::Result<Sample>
    {
        let before = Before::new(&b?)?;
        let insn   = Insn::new(&i?)?;
        let after  = After::new(&a?)?;

        Ok(Sample { before, insn, after })
    }
}

struct Samples<I>
    where I: Iterator
{
    underlying: Peekable<I>,
}

impl <I> Samples<I>
    where I: Iterator
{
    fn new(iter: I) -> Samples<I> {
        Samples { underlying: iter.peekable() }
    }
}

fn is_empty_line(l: &io::Result<String>) -> bool {
    l.as_ref().map(String::is_empty).unwrap_or(false)
}

impl <I> Iterator for Samples<I>
    where I: Iterator<Item = io::Result<String>>
{
    type Item = io::Result<Sample>;

    fn next(&mut self) -> Option<Self::Item> {
        let Samples { ref mut underlying } = self;

        if underlying.peek().map_or(false, is_empty_line) {
            underlying.next();
        }

        let b = underlying.next()?;
        let i = underlying.next()?;
        let a = underlying.next()?;

        Some(Sample::new(b, i, a))
    }
}

fn matching_ops(sample: &Sample) -> HashSet<OpCode> {
    let Sample { before: b, insn: i, after: a } = sample;

    let mut matches = HashSet::new();
    let mut regs = [b.r0, b.r1, b.r2, b.r3];
    let expected = [a.r0, a.r1, a.r2, a.r3];

    let clobber = regs[i.out];

    for op in &ALL_OPS {
        exec(*op, i.in1, i.in2, i.out, &mut regs);

        if regs == expected {
            matches.insert(*op);
        }

        regs[i.out] = clobber;
    }

    matches
}

fn resolve_op_codes(mut possible_ops: PossibleOps) -> Translation {
    let mut resolved = HashSet::new();
    let mut trn = [ADDR; 16];
    while resolved.len() != 16 {
        for (&i, ops) in possible_ops.iter_mut() {
            *ops = ops.iter()
                .cloned()
                .filter(|op| !resolved.contains(op))
                .collect();

            if ops.len() == 1 {
                let op = ops.drain().next().unwrap();
                resolved.insert(op);
                trn[i] = op;
            }
        }
    };

    trn
}

fn read_program(ix: usize) -> io::Result<std::vec::IntoIter<Insn>>
{
    let fname = env::args().nth(ix).unwrap();
    let file = File::open(fname)?;
    let reader = BufReader::new(file);

    let prog_buf = reader.lines()
        .map(|l| Insn::new(&l?))
        .collect::<io::Result<Vec<_>>>()?;

    Ok(prog_buf.into_iter())
}

fn main() -> io::Result<()> {
    let samples = {
        let fname = env::args().nth(1).unwrap();
        let file = File::open(fname)?;
        let reader = BufReader::new(file);

        Samples::new(reader.lines())
    };

    let mut part1 = 0;
    let mut possible_ops: PossibleOps = (0 .. 16)
        .map(|i| (i, ALL_OPS.iter().cloned().collect()))
        .collect();

    for res in samples {
        let sample = res?;
        let sample_ops = matching_ops(&sample);

        if sample_ops.len() >= 3 {
            part1 += 1;
        }

        let ops = possible_ops.get_mut(&sample.insn.op).unwrap();
        *ops = ops.intersection(&sample_ops).cloned().collect();
    }

    println!("Part 1: {}", part1);

    let trn = resolve_op_codes(possible_ops);
    let part2 = eval(&trn, read_program(2)?);
    println!("Part 2: {}", part2);

    Ok(())
}
