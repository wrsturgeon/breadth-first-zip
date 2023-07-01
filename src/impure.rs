/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */

//! When an iterator might not always return the same value at the same index (e.g. `next` having side effects).

/// Recursive implementation of a breadth-first exhaustive `zip`.
pub struct BreadthFirstZipped<Head: Iterator + Clone, Tail: crate::BreadthFirst> {
    /// Enumerated iterator for this current "index" in the recursive scheme.
    head: ::core::iter::Enumerate<Head>,
    /// Copy of the original iterator to allow rewinding later.
    orig_head: ::core::iter::Enumerate<Head>,
    /// Implementations for the rest of the list.
    tail: Tail,
    /// Current value: we don't always advance every call.
    current: crate::Indexed<Head::Item>,
}

impl<Head: Iterator + Clone, Tail: crate::BreadthFirst> BreadthFirstZipped<Head, Tail> {
    /// Initialize a new recursive node of a breadth-first zip implementation.
    /// # Errors
    /// If any iterator is empty.
    #[inline(always)]
    pub fn new(head: Head, tail: Tail) -> Result<Self, &'static str> {
        #![allow(clippy::shadow_unrelated)]
        let orig_head = head.enumerate();
        let mut head = orig_head.clone();
        let current = head
            .next()
            .ok_or("Tried to breadth-first zip an empty iterator")
            .map(crate::indexed)?;
        Ok(Self {
            head,
            orig_head,
            tail,
            current,
        })
    }
}

impl<Head: Iterator + Clone, Tail: crate::BreadthFirst> crate::BreadthFirst
    for BreadthFirstZipped<Head, Tail>
where
    Head::Item: Clone,
    (Head::Item, Tail::Advance): crate::Flatten,
{
    const DEPTH: usize = Tail::DEPTH + 1;
    type Advance = (Head::Item, Tail::Advance);
    #[inline(always)]
    #[must_use]
    fn advance(&mut self, index_sum: usize) -> Option<Self::Advance> {
        loop {
            if let Some(tail) = self
                .tail
                .advance(index_sum.checked_sub(self.current.index)?)
            {
                return Some((self.current.value.clone(), tail));
            }
            self.current = (self.current.index < index_sum) // Comparison is just an optimization, not logically necessary
                .then(|| self.head.next().map(crate::indexed))??;
            self.tail.rewind();
        }
    }
    #[inline(always)]
    fn rewind(&mut self) {
        self.head = self.orig_head.clone();
        self.tail.rewind();
        self.current = self.head.next().map(crate::indexed).unwrap();
    }
}

/// Helper struct for a breadth-first zip: a counter controlling the maximum index sum of the internal recursive implementation.
pub struct BreadthFirstManager<Tail: crate::BreadthFirst> {
    /// Recursive implementation.
    tail: Tail,
    /// "Global" counter to allow the maximum possible sum of indices.
    index_sum: usize,
}

impl<Tail: crate::BreadthFirst> BreadthFirstManager<Tail> {
    /// Initialize a new breadth-first algorithm.
    #[inline(always)]
    #[must_use]
    pub const fn new(tail: Tail) -> Self {
        Self { tail, index_sum: 0 }
    }
}

/// Zip a tuple into a lazy breadth-first traversal of each possible combination with a monotonically increasing sum of indices.
pub trait BreadthFirstZip {
    /// Rearrangement of input into a nested tuple.
    type Nested: crate::BreadthFirst;
    /// Lazy breadth-first exhaustive `zip` that guarantees a monotonically increasing sum of indices.
    /// # Errors
    /// If any iterator is empty.
    fn breadth_first_zip(self) -> Result<BreadthFirstManager<Self::Nested>, &'static str>;
    /// Unflatten a tuple like `(A, B, C)` to `BreadthFirstZipped<A, BreadthFirstZipped<B, BreadthFirstZipped<C, BaseCase>>>`.
    fn unflatten(self) -> Result<Self::Nested, &'static str>;
}

impl BreadthFirstZip for () {
    type Nested = crate::BaseCase;
    #[inline(always)]
    fn breadth_first_zip(self) -> Result<BreadthFirstManager<Self::Nested>, &'static str> {
        self.unflatten().map(BreadthFirstManager::new)
    }
    #[inline(always)]
    fn unflatten(self) -> Result<Self::Nested, &'static str> {
        Ok(crate::BaseCase(true))
    }
}

// TODO: https://stackoverflow.com/questions/68606470/how-to-return-a-reference-when-implementing-an-iterator
#[allow(clippy::missing_trait_methods)]
impl<Tail: crate::BreadthFirst> Iterator for BreadthFirstManager<Tail> {
    type Item = <Tail::Advance as crate::Flatten>::Flattened;
    #[inline(always)]
    #[must_use]
    fn next(&mut self) -> Option<Self::Item> {
        self.tail
            .advance(self.index_sum)
            .map_or_else(
                || {
                    self.index_sum = self.index_sum.checked_add(1)?;
                    self.tail.rewind();
                    self.tail.advance(self.index_sum)
                },
                Some,
            )
            .map(crate::Flatten::flatten)
    }
}

breadth_first_zip_macros::implement!(false); // Implement traits for (A,), (A, B), (A, B, C), (A, B, C, D), ...
