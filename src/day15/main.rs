extern crate lib;
extern crate termion;

use lib::grid::Grid;
use std::collections::{BTreeSet, HashMap, VecDeque};
use std::env;
use std::fmt;
use std::fs::File;
use std::io::{self, BufRead, BufReader, Read};
use std::thread;
use std::time::Duration;
use termion::clear;
use termion::cursor;

#[derive(Copy, Clone, Debug, Hash, PartialEq, Eq)]
enum Class { Goblin, Elf }
impl Class {
    fn from_byte(b: u8) -> Option<Class> {
        use Class::*;
        match b as char {
            'E' => Some(Elf),
            'G' => Some(Goblin),
            _   => None,
        }
    }

    fn to_char(&self) -> char {
        use Class::*;
        match self {
            Elf    => 'E',
            Goblin => 'G',
        }
    }

    fn enemy(&self) -> Class {
        use Class::*;
        match self {
            Goblin => Elf,
            Elf => Goblin,
        }
    }
}

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
enum Cell { Empty, Wall, Entity(usize) }
impl Cell {
    fn from_byte(b: u8) -> Cell {
        use Cell::*;
        match b as char {
            '.' => Empty,
            '#' => Wall,
            _   => Entity(0),
        }
    }

    fn to_char(&self) -> char {
        use Cell::*;
        match self {
            Empty => '.',
            Wall  => '#',
            Entity(_) => 'X',
        }
    }
}

#[derive(Debug)]
struct Entity {
    class: Class,
    x: usize,
    y: usize,
    hp: usize,
    ap: usize,
}

impl Entity {
    fn new(class: Class, ap: usize, x: usize, y: usize) -> Entity {
        Entity { class, x, y, ap, hp: 200 }
    }
}

enum Step {
    NotFound,
    Adjacent,
    Towards(usize, usize),
}

#[derive(Clone, Copy)]
enum Search {
    Unexplored,
    Explored(usize),
}

impl fmt::Debug for Search {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Search::Unexplored => write!(f, "."),
            Search::Explored(d) => write!(f, "{}", d),
        }
    }
}

enum SimResult {
    Win(Class),
    Ongoing(HashMap<Class, usize>),
}

struct Game {
    entities: Vec<Entity>,
    map: Grid<Cell>,
}

impl Game {
    fn new(init: String, aps: HashMap<Class, usize>) -> io::Result<Game> {
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
        let mut entities = Vec::new();

        {
            let r = BufReader::new(init.as_bytes());
            for (j, l) in r.lines().enumerate() {
                for (i, b) in l?.as_bytes().iter().enumerate() {
                    map[(i, j)] = Cell::from_byte(*b);

                    if let Cell::Entity(ref mut id) = map[(i, j)] {
                        *id = entities.len();
                        entities.push(
                            Class::from_byte(*b)
                                .map(|c| Entity::new(c, aps[&c], i, j))
                                .expect("Entity from class"));
                    }
                }
            }
        }


        Ok(Game { map, entities })
    }

    fn has_class(&self, class: Class) -> bool {
        self.entities.iter()
            .find(|e| e.class == class && e.hp > 0)
            .is_some()
    }

    fn hitpoint_sum(&self) -> usize {
        self.entities.iter().map(|e| e.hp).sum()
    }

    fn find_nearest(&self, i: usize, j: usize, class: Class) -> Step {
        let Game { entities, map } = self;

        struct Frontier {
            x: usize,
            y: usize,
            dist: usize,
        }

        let mut frontier = VecDeque::new();
        frontier.push_back(Frontier { x: i, y: j, dist: 0 });

        let mut frontier: VecDeque<_> = neighbours(i, j)
            .iter().cloned()
            .map(|(x, y)| Frontier { x, y, dist: 1 })
            .collect();

        let mut search =
            Grid::new(map.width(), map.height(), Search::Unexplored);

        // Initialise the starting point as already explored.
        search[(i, j)] = Search::Explored(0);

        let mut nearest = None;
        let mut candidates = vec![];
        while let Some(Frontier { x, y, dist }) = frontier.pop_front() {
            if let Some(ndist) = nearest {
                if ndist < dist {
                    // We found a nearer candidate, everything in the frontier
                    // after this will only be further away.
                    break;
                }
            }

            // If we have already explored this region, we need not do it
            // again.
            if let Search::Explored(sdist) = search[(x, y)] {
                if sdist > dist {
                    panic!("Ordering inversion");
                } else {
                    continue;
                }
            }

            match map[(x, y)] {
                // Walls stem the search.
                Cell::Wall => continue,

                // Empty cells warrant further exploration
                Cell::Empty => {
                    for &(ni, nj) in neighbours(x, y).iter() {
                        frontier.push_back(
                            Frontier { x: ni, y: nj, dist: dist + 1 }
                        );
                    }
                },

                Cell::Entity(id) => {
                    let Entity { class: c, .. } = entities[id];
                    if c != class {
                        // Entities of an incorrect class also stem the search.
                        continue
                    } else {
                        // Entities of the correct class are potential
                        // candidates.
                        let ndist = *nearest.get_or_insert(dist);
                        assert!(ndist == dist);

                        // Push with y co-ordinate first, to make it easy to
                        // search in "reading" order.
                        candidates.push((y, x));
                    }
                },
            };

            search[(x, y)] = Search::Explored(dist);
        }

        candidates.sort_unstable();
        candidates.first().map_or(
            Step::NotFound,
            |&(cj, ci)| chart_course(i, j, ci, cj, &search, &map),
        )
    }

    /// Returns true if the attack caused a kill.
    fn try_attack(&mut self, i: usize, j: usize) -> bool {
        let Game { entities, map } = self;

        let (class, ap) = if let Cell::Entity(id) = map[(i, j)] {
            let etty = &entities[id];
            (etty.class, etty.ap)
        } else {
            panic!("Must attack from a cell containing an entity");
        };

        let enemy = class.enemy();
        let weakest = neighbours(i, j).iter()
            .filter_map(|&(k, l)| {
                if let Cell::Entity(id) = map[(k, l)] {
                    let e = &entities[id];
                    if e.class == enemy {
                        return Some(e.hp)
                    }
                }

                None
            }).min();

        if let Some(low_hp) = weakest {
            for &(k, l) in &neighbours(i, j) {
                if let Cell::Entity(id) = map[(k, l)] {
                    let e = &mut entities[id];
                    if e.class == enemy && e.hp == low_hp {
                        if e.hp > ap {
                            e.hp -= ap;
                            break
                        } else {
                            e.hp = 0;
                            map[(k, l)] = Cell::Empty;
                            return true
                        }
                    }
                }
            }
        }

        false
    }

    fn travel(&mut self, i: usize, j: usize, k: usize, l: usize) {
        let Game { map, entities } = self;

        assert_eq!(Cell::Empty, map[(k, l)], "Must move into empty cell");

        if let Cell::Entity(id) = map[(i, j)] {
            map[(i, j)] = Cell::Empty;
            map[(k, l)] = Cell::Entity(id);

            let e = &mut entities[id];
            assert_eq!(e.x, i, "Entities out of sync with Map");
            assert_eq!(e.y, j, "Entities out of sync with Map");

            e.x = k;
            e.y = l;
        } else {
            panic!("Only entities can travel");
        }
    }

    fn simulate_round(&mut self) -> SimResult {
        let play_order = {
            // Invert order to sort by Y coord first.
            let mut coords: Vec<_> = self.entities.iter()
                .filter(|e| e.hp > 0)
                .map(|e| (e.y, e.x))
                .collect();

            coords.sort_unstable();
            coords
        };

        let mut deaths = HashMap::new();
        for (j, i) in play_order {
            // Check there are some entities of each type.
            for c in &[Class::Goblin, Class::Elf] {
                if !self.has_class(*c) {
                    return SimResult::Win(c.enemy())
                }
            }

            match self.map[(i, j)] {
                // The entity died.
                Cell::Empty => continue,
                Cell::Wall => panic!("Wall"),

                Cell::Entity(id) => {
                    // println!("{}'s turn", id);
                    let enemy = self.entities[id].class.enemy();
                    let (k, l) = match self.find_nearest(i, j, enemy) {
                        // Nothing for this unit to do.
                        Step::NotFound => continue,
                        Step::Adjacent => (i, j),
                        Step::Towards(k, l) => {
                            self.travel(i, j, k, l);
                            (k, l)
                        }
                    };

                    if self.try_attack(k, l) {
                        let mut count = deaths.entry(enemy).or_insert(0);
                        *count += 1;
                    }
                }
            }
        }

        SimResult::Ongoing(deaths)
    }
}

impl fmt::Debug for Game {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let Game { map, entities } = self;
        for j in 0 .. map.height() {
            let mut health = vec![];
            for i in 0 .. map.width() {
                let cell = map[(i, j)];
                if let Cell::Entity(id) = cell {
                    let Entity { class, hp, .. } = entities[id];
                    write!(f, "{}", class.to_char())?;
                    health.push((class, hp));
                } else {
                    write!(f, "{}", cell.to_char())?;
                }
            }

            writeln!(
                f, " {}",
                health.iter()
                    .map(|(c, h)| format!("{}({})", c.to_char(), h))
                    .collect::<Vec<_>>()
                    .join(", "),
            )?;
        }

        Ok(())
    }
}

/// Neighbours to point (i, j), in "reading order" (top to bottom, left to
/// right).  Assumes all neighbours exist -- this assumption works because the
/// inputs all have bounding walls, which we do not take the neighbours of.
fn neighbours(i: usize, j: usize) -> [(usize, usize); 4] {
    [
        (i, j - 1),
        (i - 1, j),
        (i + 1, j),
        (i, j + 1),
    ]
}

fn distance(i: usize, j: usize, k: usize, l: usize) -> usize {
    let si = i as isize;
    let sj = j as isize;
    let sk = k as isize;
    let sl = l as isize;

    ((sk - si).abs() + (sl - sj).abs()) as usize
}

fn chart_course(
    i: usize, j: usize, ci: usize, cj: usize, search: &Grid<Search>, map: &Grid<Cell>,
) -> Step {
    // println!("({}, {}) -> ({}, {})", i, j, ci, cj);
    // println!("{:?}", search);

    if distance(i, j, ci, cj) == 1 {
        // println!("Adjacent");
        return Step::Adjacent;
    }

    let mut frontier = VecDeque::new();
    let mut candidates = BTreeSet::new();

    frontier.push_back((ci, cj));

    while let Some((k, l)) = frontier.pop_front() {
        let s = search[(k, l)];
        match s {
            Search::Unexplored => {
                let err = format!("chart_course: Unexplored @ ({}, {})", k, l);
                panic!(err)
            },

            Search::Explored(0) => panic!("chart_course: Self"),

            // Inverted co-ords for reading-order sorting purposes.
            Search::Explored(1) => { candidates.insert((l, k)); },

            // Further away, expand the frontier.
            Search::Explored(d) => for &(ni, nj) in neighbours(k, l).iter() {
                if let Cell::Empty = map[(ni, nj)] {
                    if let Search::Explored(e) = search[(ni, nj)] {
                        if e == d - 1 {
                            // This neighbour is one step nearer to the target,
                            // explore it as an extension of the path.
                            frontier.push_back((ni, nj));
                        }
                    }
                }
            }
        }
    }

    // println!("Step: {:?}", candidates);
    candidates.iter().next().map_or(Step::NotFound, |&(l, k)| Step::Towards(k, l))
}

fn parse_input(aps: HashMap<Class, usize>) -> io::Result<Game> {
    let fname = env::args().nth(1).unwrap();
    let mut file = File::open(fname)?;
    let mut buf = String::new();
    file.read_to_string(&mut buf)?;

    Game::new(buf, aps)
}

fn part1() -> io::Result<usize> {
    let mut game = {
        let mut aps = HashMap::new();
        aps.insert(Class::Elf, 3);
        aps.insert(Class::Goblin, 3);

        parse_input(aps)?
    };

    println!("Initially:");
    println!("{:?}", game);
    let mut i = 0;
    loop {
        thread::sleep(Duration::from_millis(100));
        print!("{}{}", clear::All, cursor::Goto(0, 0));

        if let SimResult::Win(_) = game.simulate_round() {
            break;
        }

        i += 1;
        println!("Round {}:", i);
        println!("{:?}", game);
    }

    Ok(game.hitpoint_sum() * i)
}

fn try_elf_ap(eap: usize, print: bool) -> io::Result<Option<usize>> {
    let mut game = {
        let mut aps = HashMap::new();
        aps.insert(Class::Elf, eap);
        aps.insert(Class::Goblin, 3);

        parse_input(aps)?
    };

    if print {
        println!("Try {} ap", eap);
        println!("{:?}", game);
    }

    let mut i = 0;
    loop {
        if print {
            thread::sleep(Duration::from_millis(100));
            print!("{}{}", clear::All, cursor::Goto(0, 0));
            println!("Round {}:", i + 1);
        }

        match game.simulate_round() {
            SimResult::Win(Class::Goblin) => {
                if print { println!("[-] Goblin Win") }
                return Ok(None)
            },
            SimResult::Win(Class::Elf) => {
                if print { println!("[+] Elf Win, {}", eap) }
                return Ok(Some(game.hitpoint_sum() * i))
            },

            SimResult::Ongoing(deaths) =>
                if deaths.get(&Class::Elf).cloned().unwrap_or(0) > 0 {
                    if print { println!("[-] Elf Death") }
                    return Ok(None)
                } else {
                    if print { println!("{:?}", game) }
                    i += 1;
                },
        }
    }
}

fn part2() -> io::Result<usize> {
    let mut lo = 4;
    let mut hi = {
        let mut ub = 4;
        while let None = try_elf_ap(ub, false)? {
            ub *= 2;
        }
        ub + 1
    };

    while lo < hi {
        let m = lo + (hi - lo) / 2;

        if let None = try_elf_ap(m, false)? {
            lo = m + 1;
        } else {
            hi = m;
        }
    }

    println!("Elves win with {} ap", hi);
    try_elf_ap(hi, true).map(|res| res.expect("Upperbound of search should succeed"))
}

fn main() -> io::Result<()> {
    let p1 = part1()?;
    let p2 = part2()?;

    println!("Part 1: {}", p1);
    println!("Part 2: {}", p2);
    Ok(())
}
