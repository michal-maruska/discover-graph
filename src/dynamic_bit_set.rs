use petgraph::visit::VisitMap;
use petgraph::stable_graph::{IndexType};
use fixedbitset::FixedBitSet;

// Implement VisitMap trait for a new type - DynamicBitSet



/// Dynamic bit set that wraps FixedBitSet and can grow as needed
#[derive(Clone)]
pub struct DynamicBitSet {
    inner: FixedBitSet,
}

impl DynamicBitSet {
    pub fn new() -> Self {
        Self {
            inner: FixedBitSet::new(),
        }
    }

    pub fn with_capacity(capacity: usize) -> Self {
        Self {
            inner: FixedBitSet::with_capacity(capacity),
        }
    }

    pub fn clear(&mut self) {
        self.inner.clear();
    }

    pub fn reset(&mut self) {
        self.inner.clear();
    }

    pub fn len(&self) -> usize {
        self.inner.len()
    }
}

impl<Ix> VisitMap<Ix> for DynamicBitSet
where
    Ix: IndexType,
{
    fn visit(&mut self, node_idx: Ix) -> bool {
        let idx = node_idx.index();

        // Grow the bit set if needed using FixedBitSet's grow method
        if idx >= self.inner.len() {
            self.inner.grow(idx + 1);
        }

        !self.inner.put(idx)
    }

    fn is_visited(&self, node_idx: &Ix) -> bool {
        let idx = node_idx.index();
        self.inner.contains(idx)
    }

    fn unvisit(&mut self, x: Ix) -> bool {
        if self.is_visited(&x) {
            self.inner.toggle(x.index());
            return true;
        }
        false
    }
}



#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_grow_bitset() {
        let mut b = DynamicBitSet::new();
        b.reset();
        b.clear();

        let base: usize = 2;
        for i in 1..8 {
            b.visit(base.pow(i) );
        }
        assert_eq!(b.len(), 129);

        let mut visited: Vec<usize> = vec![];
        for j in 1..b.len() {
            if b.is_visited(&j) {
                visited.push(j)
            }
        }
        assert_eq!(visited, [2,4,8,16,32,64,128]);

        b.reset(); // still:
        // b.clear();
        assert_eq!(b.len(), 129);
    }
}
