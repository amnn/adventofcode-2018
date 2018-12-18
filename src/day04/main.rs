#[macro_use] extern crate lib;

use lib::chunkable::Chunkable;
use std::collections::{HashMap, HashSet};
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

#[derive(Clone, Debug, Eq, PartialEq)]
enum Desc { Sleep, Wake }
use self::Desc::*;

#[derive(Clone, Debug)]
struct Event {
    desc: Desc,
    min: usize,
    guard: usize,
}

struct SleepRange {
    start: usize,
    end: usize,
    guard: usize,
}

/// Returns the minute that `guard` is most often spent asleep at, and
/// the number of days `guard` was asleep at that minute in.
///
/// Pre-condition: Assumes there is a "sleepiest" minute.
fn sleepiest_minute(guard: usize, events: &Vec<Event>) -> (usize, usize) {
    let mut guard_events: Vec<Event> = events
        .iter()
        .filter(|event| event.guard == guard)
        .cloned()
        .collect();

    guard_events.sort_unstable_by_key(|event| event.min);
    let mut days_by_min = guard_events
        .iter()
        .scan(0, |days, event| {
            let Event { desc, .. } = event;
            match desc {
                Sleep => *days += 1,
                Wake  => *days -= 1,
            }

            Some((*days, event.min))
        })
        .chunk_by(|(_, min)| *min)
        .peekable();

    let mut max_min = 0;
    let mut max_days = 0;
    while let Some((days, min)) = (&mut days_by_min).last() {
        if days > max_days {
            max_min = min;
            max_days = days;
        }
    }

    (max_min, max_days)
}

fn main() -> io::Result<()> {
    let inputs = parse_input()?;

    let sleep_events: Vec<Event> = inputs
        .into_iter()
        .scan(0, |on_duty, log| {
            if let Log::ShiftStart { id, .. } = &log {
                *on_duty = *id;
            }

            Some((*on_duty, log))
        }).filter_map(|(guard, log)| {
            match log {
                Log::ShiftStart { .. } => return None,
                Log::Sleep { min, .. } => return Some(Event { desc: Sleep, min, guard }),
                Log::Wake  { min, .. } => return Some(Event { desc: Wake,  min, guard }),
            }
        }).collect();

    let sleep_durations = sleep_events
        .iter()
        .scan(0, |asleep_from, Event { desc, min, guard }| {
            match desc {
                Sleep => {
                    *asleep_from = *min;
                    return Some(None)
                },

                Wake => {
                    return Some(Some(
                            SleepRange {
                                start: *asleep_from,
                                end: *min,
                                guard: *guard
                            }))
                }
            }
        }).filter_map(|v| v);

    let mut sleep_totals = HashMap::new();
    for SleepRange { start, end, guard } in sleep_durations {
        let mut counter = sleep_totals.entry(guard).or_insert(0);
        *counter += end - start;
    }

    {
        let (&sleepiest, _) = sleep_totals
            .iter()
            .max_by_key(|(_, total)| *total)
            .unwrap();

        let (min, count) = sleepiest_minute(sleepiest, &sleep_events);

        println!("Sleepiest Guard: {} @ {} x{}", sleepiest, min, count);
        println!("Part 1: {}", sleepiest * min);
    }

    let guards: HashSet<usize> = sleep_events
        .iter()
        .map(|event| event.guard)
        .collect();

    {
        let (sleepiest, min, count) = guards
            .iter()
            .map(|g| {
                let (min, count) = sleepiest_minute(*g, &sleep_events);
                (*g, min, count)
            })
            .max_by_key(|(_, _, count)| *count)
            .unwrap();

        println!("Sleepiest Guard: {} @ {} x{}", sleepiest, min, count);
        println!("Part 2: {}", sleepiest * min);
    }

    Ok(())
}
