use std::collections::VecDeque;
use std::env;

static TAIL: usize = 10;

struct Recipes {
    scores: Vec<u8>,
    elf1: usize,
    elf2: usize,
    next: usize,
}

impl Recipes {
    fn new() -> Recipes {
        Recipes { scores: vec![3, 7], elf1: 0, elf2: 1, next: 0 }
    }

    fn tick(&mut self) {
        let Recipes { scores, elf1, elf2, .. } = self;

        let r1 = scores[*elf1];
        let r2 = scores[*elf2];
        let digits = r1 + r2;

        let tens = digits / 10;
        if tens != 0 {
            scores.push(tens);
        }

        scores.push(digits % 10);

        *elf1 += 1 + r1 as usize;
        *elf1 %= scores.len();

        *elf2 += 1 + r2 as usize;
        *elf2 %= scores.len();
    }
}

impl Iterator for Recipes {
    type Item = u8;

    fn next(&mut self) -> Option<u8> {
        while self.next >= self.scores.len() {
            self.tick();
        }

        let res = self.scores[self.next];
        self.next += 1;
        Some(res)
    }
}

fn main() {
    let count_str = env::args().nth(1).unwrap();
    let count: usize = count_str.parse().unwrap();

    {
        print!("Part 1: ");
        let recipes = Recipes::new();
        for score in recipes.skip(count).take(TAIL) {
            print!("{}", score);
        }
        println!("");
    }

    {
        let sub_seq: VecDeque<u8> = count_str.chars()
            .map(|c| c.to_digit(10).unwrap() as u8)
            .collect();

        let mut recipes = Recipes::new();
        let mut window: VecDeque<_> = (&mut recipes).take(sub_seq.len())
            .collect();

        let mut i = 0;
        loop {
            if sub_seq == window {
                break;
            }

            window.pop_front();
            window.push_back(recipes.next().unwrap());
            i += 1
        }

        println!("Part 2: {}", i);

    }
}
