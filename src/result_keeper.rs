use std::collections::VecDeque;
use std::iter;

pub struct ResultKeeper<T>(usize, VecDeque<(f64, T)>);

impl<T> ResultKeeper<T> {
    pub fn new(size: usize) -> Self {
        Self(size, VecDeque::with_capacity(size + 1))
    }

    pub fn add(&mut self, fitness: f64, result: T) {
        self.1.push_back((fitness, result));
        self.1
            .make_contiguous()
            .sort_unstable_by(|(fitness_one, _), (fitness_two, _)| {
                fitness_one.partial_cmp(fitness_two).unwrap()
            });
        self.1.truncate(self.0);
    }

    pub fn best(&self) -> Option<&T> {
        self.1.front().map(|pair| &pair.1)
    }
}

impl<T> Default for ResultKeeper<T> {
    fn default() -> Self {
        Self::new(1)
    }
}

impl<T> iter::Iterator for ResultKeeper<T> {
    type Item = T;

    fn next(&mut self) -> Option<Self::Item> {
        self.1.pop_front().map(|pair| pair.1)
    }
}
