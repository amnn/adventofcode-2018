extern crate lib;
#[macro_use] extern crate num_derive;
extern crate num_traits;

use lib::grid::Grid;
use num_traits::cast::FromPrimitive;
use std::collections::BTreeMap;
use std::env;
use std::fs::File;
use std::io::{self, Read, BufRead, BufReader};

#[derive(Copy, Clone, Debug)]
enum CornerType {
    ULDR, /* Up <-> Left, Down <-> Right */
    URDL, /* Up <-> Right, Down <-> Left */
}

impl CornerType {
    fn turn(&self, inbnd: Dir) -> Turn {
        match self {
            CornerType::ULDR /* / */=>
                match inbnd {
                    Dir::Up   | Dir::Down  => Turn::Right,
                    Dir::Left | Dir::Right => Turn::Left,
                },
            CornerType::URDL /* \ */=>
                match inbnd {
                    Dir::Up   | Dir::Down  => Turn::Left,
                    Dir::Left | Dir::Right => Turn::Right,
                },
        }
    }
}

#[derive(Copy, Clone)]
enum Cell { Empty, Horiz, Vert, XSect, Corner(CornerType) }

#[derive(Copy, Clone, Debug)]
enum Turn { Straight = 0, Right = 1, Left = 3 }

impl Turn {
    fn next(&mut self) -> Turn {
        use Turn::*;

        let prev = *self;
        *self = match self {
            Left => Straight,
            Straight => Right,
            Right => Left,
        };

        prev
    }
}

#[derive(Copy, Clone, Debug, Eq, FromPrimitive, PartialEq)]
enum Dir { Up = 0, Right = 1, Down = 2, Left = 3 }

impl Dir {
    fn from_byte(b: u8) -> Option<Dir> {
        use Dir::*;

        match b as char {
            '>' => Some(Right),
            '<' => Some(Left),
            '^' => Some(Up),
            'v' => Some(Down),
            _   => None,
        }
    }

    fn turn(self, by: Turn) -> Dir {
        let d = self as u8;
        let t = by as u8;
        let e = (d + t) % 4;

        Dir::from_u8(e).unwrap()
    }

    fn travel_from(self, pos: (usize, usize)) -> (usize, usize) {
        use Dir::*;

        let (i, j) = pos;
        match self {
            Up    => (i, j - 1),
            Right => (i + 1, j),
            Down  => (i, j + 1),
            Left  => (i - 1, j),
        }
    }
}

struct Cart {
    dir: Dir,
    turn: Turn,
}

impl Cart {
    fn from_byte(b: u8) -> Option<Cart> {
        Dir::from_byte(b).map(|dir| Cart { dir, turn: Turn::Left })
    }
}

impl Cell {
    fn from_byte(b: u8) -> Cell {
        use Cell::*;

        match b as char {
            ' '  => Empty,
            '+'  => XSect,
            '/'  => Corner(CornerType::ULDR),
            '\\' => Corner(CornerType::URDL),

            '-' | '<' | '>' => Cell::Horiz,
            '|' | '^' | 'v' => Cell::Vert,
            _ => panic!("Unexpected character"),
        }
    }

    fn turn(self, cart: &mut Cart) {
        use Cell::*;
        let turn = match self {
            Empty => panic!("Off the tracks!"),

            Horiz | Vert => Turn::Straight,
            XSect => cart.turn.next(),
            Corner(ctype) => ctype.turn(cart.dir),
        };

        cart.dir = cart.dir.turn(turn);
    }
}

struct Tracks(Grid<Cell>);
type Carts = BTreeMap<(usize, usize), Cart>;

impl Tracks {
    fn new(init: String) -> io::Result<(Tracks, Carts)> {
        let (width, height) = {
            let mut r = BufReader::new(init.as_bytes());
            let mut w = 0;
            let mut h = 0;

            for l in r.lines() {
                h += 1;
                w = w.max(l?.len());
            }

            (w, h)
        };

        let mut map = Grid::new(width, height, Cell::Empty);
        let mut carts = Carts::new();

        {
            let r = BufReader::new(init.as_bytes());
            for (j, l) in r.lines().enumerate() {
                for (i, b) in l?.as_bytes().iter().enumerate() {
                    map[(i, j)] = Cell::from_byte(*b);

                    if let Some(c) = Cart::from_byte(*b) {
                        // Y-coord first to order top-bottom, left-right.
                        carts.insert((j, i), c);
                    }
                }
            }
        }

        Ok((Tracks(map), carts))
    }

    fn tick(&self, mut posns: Carts) -> Carts {
        let Tracks(ref grid) = self;

        let coords: Vec<_> = posns.keys().cloned().collect();
        for (j, i) in coords {
            // Try and remove the cart at the co-ord of interest.
            // It might not exist because it could have been crashed into.
            if let Some(mut cart) = posns.remove(&(j, i)) {
                // Move in direction cart is facing
                let (k, l) = cart.dir.travel_from((i, j));

                // Find new cell under cart
                let cell = grid[(k, l)];

                // Turn cart appropriately
                cell.turn(&mut cart);

                if posns.insert((l, k), cart).is_some() {
                    // We overwrote an existing cart, meaning we crashed.
                    println!("Crash at {},{}", k, l);
                    posns.remove(&(l, k));
                };
            }
        }

        posns
    }
}

fn main() -> io::Result<()> {
    let (tracks, mut carts) = {
        let fname = env::args().nth(1).unwrap();
        let mut file = File::open(fname)?;
        let mut buf = String::new();

        file.read_to_string(&mut buf)?;
        Tracks::new(buf)?
    };

    println!("#Carts = {}", carts.len());
    while carts.len() > 1 {
        carts = tracks.tick(carts);
    }

    assert_eq!(carts.len(), 1);
    let (j, i) = carts.keys().nth(0).unwrap();
    println!("Last Cart at {},{}", i, j);

    Ok(())
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn dir_turn() {
        let turns = vec![
            Turn::Left, Turn::Straight,
            Turn::Left, Turn::Straight,
            Turn::Left, Turn::Straight,
            Turn::Left, Turn::Straight,
            Turn::Right,
            Turn::Right,
            Turn::Right,
            Turn::Right,
        ];

        let expected = vec![
            Dir::Left, Dir::Left,
            Dir::Down, Dir::Down,
            Dir::Right, Dir::Right,
            Dir::Up, Dir::Up,
            Dir::Right,
            Dir::Down,
            Dir::Left,
            Dir::Up
        ];

        let actual: Vec<_> = turns.iter()
            .scan(Dir::Up, |d, &t| { *d = d.turn(t); Some(*d) })
            .collect();

        assert_eq!(expected, actual);
    }
}
