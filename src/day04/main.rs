#[macro_use] extern crate lib;

use std::collections::{BTreeMap, HashMap};
use std::env;
use std::fs::File;
use std::io::{self, BufRead, BufReader};

input! {
    enum Log {
        #["[{d}-{d}-{d} {d}:{d}] Guard #{d}"; "begins shift"]
        ShiftStart { y: usize, m: usize, d: usize, hr: usize, min: usize, id: usize },

        #["[{d}-{d}-{d} {d}:{d}]"; "wakes up"]
        Wake { y: usize, m: usize, d: usize, hr: usize, min: usize },

        #["[{d}-{d}-{d} {d}:{d}]"; "falls asleep"]
        Sleep { y: usize, m: usize, d: usize, hr: usize, min: usize }
    }
}

enum Transition {
    Wake, Sleep
}

struct SleepRecord {
    total_mins: usize,
    transitions: BTreeMap<usize, Transition>,
}

impl SleepRecord {
    fn new() -> Self {
        SleepRecord {
            total_mins: 0,
            transitions: BTreeMap::new(),
        }
    }
}

type Input = Vec<Log>;
fn parse_input() -> io::Result<Input> {
    let fname = env::args().nth(1).unwrap();
    let file = File::open(fname)?;
    let reader = BufReader::new(file);
    let mut logs = vec![];

    for line in reader.lines() {
        logs.push(Log::new(&line?)?);
    }

    logs.sort_unstable_by_key(|log| match log {
        Log::ShiftStart{ y, m, d, hr, min, ..}
            | Log::Sleep{ y, m, d, hr, min }
            | Log::Wake{ y, m, d, hr, min }
        => (*y, *m, *d, *hr, *min),
    });

    Ok(logs)
}

fn main() -> io::Result<()> {
    let inputs = parse_input()?;

    let mut mins_asleep: HashMap<usize, SleepRecord> = HashMap::new();
    let mut on_duty = 0;
    let mut ts = 0;
    let mut asleep = false;
    for log in inputs {
        match log {
            Log::ShiftStart { id, min, .. } => {
                asleep = false;
                on_duty = id;
                ts = min;
            },

            Log::Wake { min,.. } => {
                assert!(on_duty != 0, "No guard on duty");
                assert!(asleep);

                let mut record = mins_asleep.get_mut(&on_duty).unwrap();
                record.total_mins += min - ts;
                record.transitions.insert(min, Transition::Wake);

                asleep = false;
                ts = min;
            },

            Log::Sleep { min, .. } => {
                assert!(on_duty != 0, "No guard on duty");

                let mut record = mins_asleep
                    .entry(on_duty)
                    .or_insert_with(SleepRecord::new);

                record.transitions.insert(min, Transition::Sleep);

                asleep = true;
                ts = min;
            },
        }
    }

    let (sleepiest, record) = mins_asleep
        .iter()
        .max_by_key(|(_, rec)| rec.total_mins)
        .unwrap();

    println!("Sleepiest Guard: {}", sleepiest);

    let mut days = 0;
    let mut max_days = 0;
    let mut start = 0;
    let mut end = 0;
    for (min, transition) in &record.transitions {
        match transition {
            Transition::Sleep => {
                days += 1;

                if days >= max_days {
                    max_days = days;
                    start = *min;
                }
            },
            Transition::Wake => {
                if days == max_days {
                    end = *min;
                }

                days -= 1;
            },
        }
    }

    println!("Sleepiest Range: {} -- {}", start, end);

    Ok(())
}
