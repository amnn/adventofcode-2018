extern crate lib;
use lib::grid::Grid;

type PwrGrid = Grid<isize>;

fn power(serial: usize, x: usize, y: usize) -> isize {
    let rack_id = x + 10;
    let mut level = (rack_id * y + serial) as isize;
    level *= rack_id as isize;
    level /= 100;
    level %= 10;
    level -= 5;
    level
}

fn max_in_grid_nbyn(dim: usize, grid: &PwrGrid) -> (usize, usize, isize) {
    let wndw = grid.width() - dim + 1;
    let sum_grid = Grid::new_with_mapping(
        wndw, wndw, |i, j| {
            (0 .. dim).map(|k| {
                (0 .. dim)
                    .map(|l| grid[(i + l, j + k)])
                    .sum::<isize>()
            }).sum::<isize>()
        });

    let (i, &p) = sum_grid.elems()
        .enumerate()
        .max_by_key(|&(_, p)| p)
        .unwrap();

    let (x, y) = sum_grid.coord(i);
    (x + 1, y + 1, p)
}

fn main() {
    static SERIAL: usize = 7165;
    static SIDE: usize = 300;

    let grid = Grid::new_with_mapping(
        SIDE, SIDE, |i, j| power(SERIAL, i + 1, j + 1));

    {
        let (x, y, tot) = max_in_grid_nbyn(3, &grid);
        println!("Part 1: {},{} with {}", x, y, tot);
    }

    let row_cum = {
        let mut cum = Grid::new(SIDE + 1, SIDE, 0);

        for j in 0 .. SIDE {
            for i in 1 ..= SIDE {
                cum[(i, j)] = cum[(i - 1, j)] + grid[(i - 1, j)];
            }
        }

        cum
    };

    let col_cum = {
        let mut cum = Grid::new(SIDE, SIDE + 1, 0);

        for j in 1 ..= SIDE {
            for i in 0 .. SIDE {
                cum[(i, j)] = cum[(i, j - 1)] + grid[(i, j - 1)];
            }
        }

        cum
    };

    {
        let mut max_power = isize::min_value();
        let mut max_coord = (0, 0, 0);

        let mut window_sums = Grid::new(SIDE, SIDE, 0);
        for d in 0 .. SIDE {
            let end = SIDE - d;
            for j in 0 .. end {
                for i in 0 .. end {
                    // Update sum with new column
                    window_sums[(i, j)] +=
                        col_cum[(i+d, j+d)] - col_cum[(i+d, j)];

                    // Update sum with new row
                    window_sums[(i, j)] +=
                        row_cum[(i+d, j+d)] - row_cum[(i, j+d)];

                    // Update sum with new corner piece.
                    window_sums[(i, j)] += grid[(i+d, j+d)];

                    // Check against previous max.
                    if window_sums[(i, j)] > max_power {
                        max_power = window_sums[(i, j)];
                        max_coord = (i + 1, j + 1, d + 1);
                    }
                }
            }
        }

        let (x, y, dim) = max_coord;
        println!("Part 2: {},{},{} with {}", x, y, dim, max_power);
    }
}
