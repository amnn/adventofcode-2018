#[macro_use] extern crate lib;

use std::cmp::{Ord, Ordering};
use std::collections::{VecDeque, HashMap, HashSet};
use std::env;
use std::isize;
use std::fs::File;
use std::io::{self, BufRead, BufReader};
use std::ops::{Index, IndexMut};

input! {
    #["{d}, {d}"; ""]
    struct Coord { x: isize, y: isize }
}

impl Coord {
    fn neighbours(&self) -> [Coord; 4] {
        let &Coord { x, y } = self;

        [
            Coord { x: x - 1, y },
            Coord { x: x + 1, y },
            Coord { x, y: y - 1 },
            Coord { x, y: y + 1 },
        ]
    }

    fn dist(&self, other: &Coord) -> usize {
        ((self.x - other.x).abs() + (self.y - other.y).abs()) as usize
    }
}

struct Boundary {
    top: isize,
    left: isize,
    bottom: isize,
    right: isize
}

impl Boundary {
    fn around<'a, Coords>(coords: Coords) -> Boundary
        where Coords: IntoIterator<Item = &'a Coord>
    {
        let mut top = isize::MAX;
        let mut bottom = isize::MIN;

        let mut left = isize::MAX;
        let mut right = isize::MIN;

        for Coord { x, y } in coords.into_iter() {
            top = top.min(*y);
            bottom = bottom.max(*y);
            left = left.min(*x);
            right = right.max(*x);
        }

        Boundary { top, left, bottom, right }
    }

    fn contains(&self, p: &Coord) -> bool {
        self.top <= p.y
            && p.y <= self.bottom
            && self.left <= p.x
            && p.x <= self.right
    }

    fn width(&self) -> usize {
        let &Boundary { left, right, .. } = self;

        assert!(left <= right);
        (right - left + 1) as usize
    }

    fn height(&self) -> usize {
        let &Boundary { top, bottom, .. } = self;

        assert!(top <= bottom);
        (bottom - top + 1) as usize
    }

    fn area(&self) -> usize {
        self.width() * self.height()
    }
}

#[derive(Clone, Copy, PartialEq)]
enum Cell {
    Unexplored,
    Contested,
    Owned { owner: usize, dist: usize }
}

struct Grid {
    bounds: Boundary,
    contents: Box<[Cell]>,
}

impl Grid {
    fn new(bounds: Boundary) -> Grid {
        let area = bounds.area();
        let contents = vec![Cell::Unexplored; area].into_boxed_slice();
        Grid { bounds, contents }
    }

    fn ix(&self, p: &Coord) -> usize {
        assert!(self.bounds.contains(&p));
        let Boundary { top, left, .. } = self.bounds;
        let Coord { x, y } = p;

        let i = (x - left) as usize;
        let j = (y - top) as usize;

        j * self.bounds.width() + i
    }

    fn contains(&self, p: &Coord) -> bool {
        self.bounds.contains(p)
    }
}

impl Index<&Coord> for Grid {
    type Output = Cell;
    fn index<'a>(&'a self, index: &Coord) -> &'a Cell {
        &self.contents[self.ix(index)]
    }
}

impl IndexMut<&Coord> for Grid {
    fn index_mut<'a>(&'a mut self, index: &Coord) -> &'a mut Cell {
        &mut self.contents[self.ix(index)]
    }
}

fn finite_areas_surrounding(coords: &Vec<Coord>) -> HashMap<usize, usize> {
    let mut grid = Grid::new(Boundary::around(coords));
    let mut frontier = VecDeque::new();

    // Seed the grid with the initial frontier
    for (owner, coord) in coords.into_iter().enumerate() {
        assert!(grid[&coord] == Cell::Unexplored);
        grid[&coord] = Cell::Owned { owner, dist: 0 };
        frontier.push_back(coord.clone());
    }

    let mut area = HashMap::new();
    let mut infinite = HashSet::new();

    while !frontier.is_empty() {
        let p = frontier.pop_front().unwrap();

        match grid[&p] {
            Cell::Contested => continue,
            Cell::Unexplored => panic!("Unexplored cell in frontier!"),

            Cell::Owned { owner, dist } => {
                // The cell did not get contested whilst in the frontier, so
                // attribute its area to its owner.
                *area.entry(owner).or_insert(0) += 1;

                for nbr in p.neighbours().iter().cloned() {
                    if !grid.contains(&nbr) {
                        infinite.insert(owner);
                    } else if grid[&nbr] == Cell::Unexplored {
                        grid[&nbr] = Cell::Owned { owner, dist: dist + 1 };
                        frontier.push_back(nbr);
                    } else if let Cell::Owned { owner: other, dist: odist } = grid[&nbr] {
                        match (dist + 1).cmp(&odist) {
                            Ordering::Greater => { /* nop */ },
                            Ordering::Less    => panic!("Ordering inversion!"),
                            Ordering::Equal   =>
                                if other != owner {
                                    grid[&nbr] = Cell::Contested;
                                }
                        }
                    }
                }
            }
        }
    }

    area.retain(|k, _| !infinite.contains(k));
    area
}

fn median(coords: &Vec<Coord>) -> Coord {
    let mut x_coords: Vec<isize> = coords.iter().map(|c| c.x).collect();
    let mut y_coords: Vec<isize> = coords.iter().map(|c| c.y).collect();

    x_coords.sort_unstable();
    y_coords.sort_unstable();

    let midpoint = coords.len() / 2;

    Coord { x: x_coords[midpoint], y: y_coords[midpoint] }
}

fn area_within_bounded_distance(bound: usize, coords: &Vec<Coord>) -> usize {
    let mut visited = HashSet::new();
    let mut frontier = VecDeque::new();
    frontier.push_back(median(coords));

    let mut area = 0;
    'outer: while !frontier.is_empty() {
        let p = frontier.pop_front().unwrap();
        if visited.contains(&p) {
            continue
        } else {
            visited.insert(p.clone());
        }

        let mut abs_dev = 0;
        for c in coords.iter() {
            abs_dev += p.dist(c);
            if abs_dev >= bound {
                continue 'outer
            }
        }

        area += 1;
        for nbr in p.neighbours().iter().cloned() {
            frontier.push_back(nbr);
        }
    };

    area
}

fn main() -> io::Result<()> {
    let fname = env::args().nth(1).unwrap();
    let file = File::open(fname)?;
    let reader = BufReader::new(file);

    let coords = reader.lines()
        .map(|l| Coord::new(&l?))
        .collect::<io::Result<Vec<_>>>()?;

    let areas = finite_areas_surrounding(&coords);

    {
        let largest_area = areas.values().max().unwrap();
        println!("Part 1: {}", largest_area);
    }

    {
        let part2 = area_within_bounded_distance(10000, &coords);
        println!("Part 2: {}", part2);
    }

    Ok(())
}
