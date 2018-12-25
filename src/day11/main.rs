fn power(serial: isize, x: isize, y: isize) -> isize {
    let rack_id = x + 10;
    let mut level = rack_id * y + serial;
    level *= rack_id;
    level /= 100;
    level %= 10;
    level -= 5;
    level
}

static GRID_DIM: isize = 300;
static GRID: isize = GRID_DIM * GRID_DIM;

fn max_in_grid_nbyn(dim: isize, grid: &Vec<isize>) -> (isize, isize, isize) {
    let wndw = (GRID_DIM - dim + 1) * (GRID_DIM - dim + 1);
    let (i, p) = (0 .. wndw)
        .map(|i| {
            (0 .. dim).map(|j| {
                (0 .. dim).map(|k| {
                    grid[(i + j * GRID_DIM + k) as usize]
                }).sum::<isize>()
            }).sum::<isize>()
        })
        .enumerate()
        .max_by_key(|&(_, p)| p)
        .unwrap();

    let x = (i as isize) % GRID_DIM + 1;
    let y = (i as isize) / GRID_DIM + 1;

    (x, y, p)
}

fn main() {
    static SERIAL: isize = 7165;
    let grid: Vec<isize> = (0 .. GRID)
        .map(|i| power(SERIAL, i % GRID_DIM + 1, i / GRID_DIM + 1))
        .collect();

    {
        let (x, y, tot) = max_in_grid_nbyn(3, &grid);
        println!("Part 1: {},{} with {}", x, y, tot);
    }

    {
        let (dim, x, y, tot) = (1 ..= GRID_DIM).map(|dim| {
            let (x, y, tot) = max_in_grid_nbyn(dim, &grid);
            (dim, x, y, tot)
        }).max_by_key(|&(_, _, _, p)| p).unwrap();

        println!("Part 2: {},{},{} with {}", x, y, dim, tot);
    }
}
