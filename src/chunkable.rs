use std::iter::Peekable;

/// An iterator that yields the same elements as its underlying iterator,
/// inserting `None` when the value of the classifier function applied at the
/// underlying iterator's next element changes.
///
/// Constructed by a call to [`chunk_by`].
///
/// [`chunk_by`]: trait.Chunkable.html#method.chunk_by
pub struct Chunked<I: Iterator, K: Sized, C> {
    prev: Option<K>,
    iter: Peekable<I>,
    class: C
}

impl <I, K, C> Iterator for Chunked<I, K, C>
    where I: Iterator,
          K: Sized + PartialEq,
          C: FnMut(&I::Item) -> K
{
    type Item = I::Item;

    fn next(&mut self) -> Option<Self::Item> {
        let key = (self.class)(self.iter.peek()?);
        match self.prev.as_ref().map(|prev| *prev == key) {
            None => {
                self.prev = Some(key);
                self.iter.next()
            },

            Some(true) => {
                self.iter.next()
            },

            Some(false) => {
                self.prev = None;
                None
            }
        }
    }
}

/// Extension trait for iterators that allows them to be chunked according to
/// a classifier function.
pub trait Chunkable
    where Self: Iterator + Sized
{
    /// Creates an iterator which stops (but can be resumed) every time the
    /// value of `class` at the next item changes (compared to its value at the
    /// previously yielded item, if any).
    fn chunk_by<K, C>(self, class: C) -> Chunked<Self, K, C>
        where K: Sized,
              C: FnMut(&<Self as Iterator>::Item) -> K;
}

/// All iterators are chunkable.
impl <I> Chunkable for I where I: Iterator {
    fn chunk_by<K ,C> (self, class: C) -> Chunked<I, K, C>
        where K: Sized,
              C: FnMut(&I::Item) -> K
    {
        Chunked { prev: None, iter: self.peekable(), class }
    }
}

#[cfg(test)]
mod tests {
    use chunkable::Chunkable;

    #[test]
    fn unit_key() {
        let v: Vec<usize> = (0..4)
            .into_iter()
            .chunk_by(|_| ())
            .collect();

        assert_eq!(v, [0, 1, 2, 3]);
    }

    #[test]
    fn split_half() {
        let mut i = (0..4).into_iter().chunk_by(|n| *n < 2);
        let v: Vec<usize> = (&mut i).collect();
        let w: Vec<usize> = (&mut i).collect();

        assert_eq!(v, [0, 1]);
        assert_eq!(w, [2, 3]);
    }

    #[test]
    fn many_chunks() {
        let mut i = (0..9).into_iter().chunk_by(|n| *n / 3);

        let s1: usize = (&mut i).sum();
        let s2: usize = (&mut i).sum();
        let s3: usize = (&mut i).sum();

        assert_eq!(0 + 1 + 2, s1);
        assert_eq!(3 + 4 + 5, s2);
        assert_eq!(6 + 7 + 8, s3);
    }
}
