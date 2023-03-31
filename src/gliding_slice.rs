use std::iter;
use std::slice;

pub struct GlidingSlice<'a, T>(iter::Peekable<slice::ChunksExact<'a, T>>);

impl<'a, T> GlidingSlice<'a, T> {
    pub fn new(slice: &'a [T], chunk_size: usize) -> Self {
        Self(slice.chunks_exact(chunk_size).peekable())
    }
}

impl<'a, T> iter::Iterator for GlidingSlice<'a, T> {
    type Item = (&'a [T], &'a [T]);

    fn next(&mut self) -> Option<Self::Item> {
        if let Some(head) = self.0.next() {
            if let Some(&tail) = self.0.peek() {
                Some((head, tail))
            } else {
                None
            }
        } else {
            None
        }
    }
}
