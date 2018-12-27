#[macro_use] extern crate lib;
#[macro_use] extern crate scan_fmt;

use std::env;
use std::fmt;
use std::fs::File;
use std::io::{self, BufRead, BufReader};
use std::str;
use std::thread;
use std::time::Duration;

input! {
    #["position=<{d}, {d}> velocity=<{d}, {d}>"; ""]
    struct Point {
        x: isize,  y: isize,
        dx: isize, dy: isize
    }
}

impl Point {
    fn at_time(&self, t: isize) -> (isize, isize) {
        (
            self.x + self.dx * t,
            self.y + self.dy * t,
        )
    }
}

struct Canvas {
    width: usize,
    height: usize,
    buffer: String,
}

impl Canvas {
    fn new(width: usize, height: usize) -> Canvas {
        // width + 1 to accommodate new-lines.
        let mut buffer = String::with_capacity(height * (width + 1));
        let line = {
            let mut prefix = str::repeat(" ", width);
            prefix.push('\n');
            prefix
        };

        for _ in 0 .. height {
            buffer.push_str(&line);
        }

        Canvas { width, height, buffer }
    }

    fn clear_screen() {
        print!("\x1B[2J");
    }

    fn paint(&mut self, x: usize, y: usize) {
        let i = (self.width + 1) * y + x;
        unsafe { self.buffer.as_mut_vec()[i] = '#' as u8; }
    }

    fn flush(&mut self) {
        println!("{}", self);
        unsafe {
            for px in self.buffer.as_mut_vec().iter_mut() {
                if *px != '\n' as u8 {
                    *px = ' ' as u8;
                }
            }
        }
    }
}

impl fmt::Display for Canvas {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.buffer)
    }
}

struct PointCloud {
    constituents: Vec<Point>,
}

impl PointCloud {
    fn new(constituents: Vec<Point>) -> PointCloud {
        PointCloud { constituents }
    }

    fn display_at_time(&self, t: isize, canvas: &mut Canvas) -> (isize, bool) {
        let posns: Vec<(isize, isize)> =
            self.constituents.iter().map(|p| p.at_time(t)).collect();

        let mut xlo = isize::max_value();
        let mut xhi = isize::min_value();
        let mut ylo = isize::max_value();
        let mut yhi = isize::min_value();

        for &(x, y) in &posns {
            xlo = xlo.min(x);
            xhi = xhi.max(x);

            ylo = ylo.min(y);
            yhi = yhi.max(y);
        }

        let width  = xhi - xlo;
        let height = yhi - ylo;

        fn scale(dim: isize, sf: usize) -> isize {
            0.max(dim - 1) / sf as isize + 1
        }

        let res = scale(width, canvas.width)
            .max(scale(height, canvas.height));

        if res > 1 {
            return (res, false)
        }

        for &(x, y) in &posns {
            let px = (x - xlo) / res;
            let py = (y - ylo) / res;

            canvas.paint(px as usize, py as usize);
        }

        Canvas::clear_screen();
        println!("t = {: <5}, res = {: <5}, origin = ({}, {})", t, res, xlo, ylo);

        canvas.flush();
        (res, true)
    }
}

fn main() -> io::Result<()> {
    let points = {
        let fname = env::args().nth(1).unwrap();
        let file = File::open(fname)?;
        let reader = BufReader::new(file);

        PointCloud::new(
            reader.lines()
                .map(|l| Point::new(&l?))
                .collect::<io::Result<Vec<_>>>()?
        )
    };

    static TICK: Duration = Duration::from_millis(1000);
    let mut time = 0;
    let mut canvas = Canvas::new(80, 60);
    loop {
        let (res, did_draw) = points.display_at_time(time, &mut canvas);

        time += if res < 10 {
            1
        } else if res < 1000 {
            10
        } else {
            100
        };

        if did_draw {
            thread::sleep(TICK);
        }
    }
}
