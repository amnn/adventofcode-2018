use std::ops::{Index, IndexMut};

pub struct Grid<T> {
    width: usize,
    height: usize,
    storage: Box<[T]>,
}

impl <T> Grid<T> where T: Clone {
    pub fn new(width: usize, height: usize, init: T) -> Grid<T> {
        let storage = vec![init.clone(); width * height].into_boxed_slice();
        Grid { width, height, storage }
    }
}

impl <T> Grid<T> {
    pub fn new_with_mapping<F>(width: usize, height: usize, mut f: F) -> Grid<T>
        where F: FnMut(usize, usize) -> T {
        let storage = (0 .. width * height).map(|i| {
            let x = i % width;
            let y = i / width;
            f(x, y)
        }).collect::<Vec<T>>().into_boxed_slice();

        Grid { width, height, storage }
    }

    pub fn elems<'a>(&'a self) -> impl Iterator<Item = &'a T> {
        self.storage.iter()
    }

    pub fn coord(&self, i: usize) -> (usize, usize) {
        let x = i % self.width;
        let y = i / self.width;
        (x, y)
    }

    pub fn width(&self) -> usize {
        self.width
    }

    pub fn height(&self) -> usize {
        self.height
    }
}

impl <T> Index<(usize, usize)> for Grid<T> {
    type Output = T;

    fn index<'a>(&'a self, index: (usize, usize)) -> &'a T {
        let (x, y) = index;
        let i = y * self.width + x;
        &self.storage[i]
    }
}

impl <T> IndexMut<(usize, usize)> for Grid<T> {
    fn index_mut<'a>(&'a mut self, index: (usize, usize)) -> &'a mut T {
        let (x, y) = index;
        let i = y * self.width + x;
        &mut self.storage[i]
    }
}
