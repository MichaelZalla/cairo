#[derive(Debug, Clone)]
pub(crate) struct GeneratedDistribution<const N: usize> {
    pub values: [f32; N],
    next: usize,
}

impl<const N: usize> Default for GeneratedDistribution<N> {
    fn default() -> Self {
        Self {
            values: [0.0; N],
            next: 0,
        }
    }
}

impl<const N: usize> GeneratedDistribution<N> {
    pub fn sample(&mut self) -> f32 {
        if self.next == self.values.len() {
            self.next = 0;
        }

        let value = self.values[self.next];

        self.next += 1;

        value
    }
}
