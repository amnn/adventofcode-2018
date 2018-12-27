#[macro_use] extern crate lib;
#[macro_use] extern crate scan_fmt;

use std::env;
use std::fmt;
use std::fs::File;
use std::io::{self, BufRead, BufReader};

input! {
    #["{d} players; last marble is worth {d}"; " points"]
    struct GameParams {
        players: usize,
        marbles: usize
    }
}


struct LinkedList<T> {
    head: Option<NodeRef>,
    free: Option<NodeRef>,

    storage: Vec<Node<T>>,
}

type NodeRef = usize;

struct Node<T> {
    datum: T,
    left: NodeRef,
    right: NodeRef,
}

impl <T> LinkedList<T> {
    fn new(first: T) -> Self {
        LinkedList {
            head: Some(0),
            free: None,
            storage: vec![
                Node { left: 0, right: 0, datum: first },
            ],
        }
    }

    fn datum(&self, nr: NodeRef) -> &T {
        &self.storage[nr].datum
    }

    fn insert(&mut self, datum: T, after: NodeRef) -> NodeRef {
        let left = after;
        let right = self.storage[after].right;

        let nr = match self.free.take() {
            Some(nr) => {
                self.storage[nr] = Node { left, right, datum };
                nr
            },

            None => {
                let nr = self.storage.len();
                self.storage.push(Node { left, right, datum });
                nr
            }
        };

        self.storage[left].right = nr;
        self.storage[right].left = nr;

        nr
    }

    fn remove(&mut self, nr: NodeRef) -> Option<NodeRef> {
        match self.free.take() {
            Some(_) => panic!("Unexpected consecutive deletions"),
            None => {
                self.dislocate(nr);
                self.free = Some(nr);

                let right = self.storage[nr].right;
                if right == nr { None } else { Some(right) }
            }
        }
    }

    fn left(&self, pos: NodeRef, num: usize) -> NodeRef {
        (0 .. num).fold(pos, |i, _| self.storage[i].left)
    }

    fn right(&self, pos: NodeRef, num: usize) -> NodeRef {
        (0 .. num).fold(pos, |i, _| self.storage[i].right)
    }

    fn dislocate(&mut self, nr: NodeRef) {
        let lr = self.storage[nr].left;
        let rr = self.storage[nr].right;

        self.storage[lr].right = rr;
        self.storage[rr].left = lr;
    }
}

struct Game {
    marbles: LinkedList<usize>,
    current: NodeRef,
    next_marble: usize,
}

impl Game {
    fn new() -> Self {
        Game {
            marbles: LinkedList::new(0),
            current: 0,
            next_marble: 1,
        }
    }

    fn next_move(&mut self) -> usize {
        let score = if self.next_marble % 23 == 0 {
            let rpos = self.marbles.left(self.current, 7);
            let pred = self.marbles.remove(rpos);

            self.current = match pred {
                Some(pr) => pr,
                None => panic!("Can't set current marble"),
            };

            self.next_marble + self.marbles.datum(rpos)
        } else {
            let ipos = self.marbles.right(self.current, 1);
            self.current = self.marbles.insert(self.next_marble, ipos);
            0
        };

        self.next_marble += 1;
        score
    }
}

impl <T> fmt::Debug for LinkedList<T>
    where T: fmt::Display {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        if let Some(hr) = self.head {
            let mut nr = hr;
            loop {
                if nr != hr {
                    write!(f, " ")?;
                }

                let node = &self.storage[nr];
                write!(f, "{: >2}", node.datum)?;
                nr = node.right;

                if nr == hr {
                    break
                }
            }
        }

        Ok(())
    }
}

fn main() -> io::Result<()> {
    let params = {
        let fname = env::args().nth(1).unwrap();
        let file = File::open(fname)?;
        let mut reader = BufReader::new(file);
        let mut line = String::new();
        reader.read_line(&mut line)?;

        GameParams::new(line.trim())?
    };

    let mut game = Game::new();
    let mut scores = vec![0; params.players];

    for i in 0 .. params.marbles {
        let player = i % params.players;
        let score = game.next_move();
        scores[player] += score
    }

    println!("Max Score: {}", scores.iter().max().unwrap());

    Ok(())
}
