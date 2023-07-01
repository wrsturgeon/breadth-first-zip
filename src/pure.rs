/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */

//! When an iterator will always return the same value at the same index.

extern crate alloc;

use alloc::{vec, vec::Vec};

/// Run through the whole iterator once, caching its values, then read them back in subsequent calls.
struct Reiterator<Iter: Iterator> {
    /// The iterator that this struct lazily wraps.
    iter: Iter,
    /// Cache of all previously computed elements.
    cache: Vec<Iter::Item>,
    /// Index that `current` will return.
    index: usize,
}

/// Recursive implementation of a breadth-first exhaustive `zip`.
pub struct BreadthFirstZipped<Head: Iterator, Tail: crate::BreadthFirst> {
    /// Enumerated caching iterator for this current "index" in the recursive scheme.
    iter: Reiterator<Head>,
    /// Implementations for the rest of the list.
    tail: Tail,
}

impl<Head: Iterator, Tail: crate::BreadthFirst> BreadthFirstZipped<Head, Tail>
where
    Head::Item: Clone,
{
    /// Initialize a new recursive node of a breadth-first zip implementation.
    #[inline(always)]
    pub fn new(head: Head, tail: Tail) -> Result<Self, &'static str> {
        Reiterator::new(head).map(|iter| Self { iter, tail })
    }
}

impl<Iter: Iterator> Reiterator<Iter>
where
    Iter::Item: Clone,
{
    /// Set up to return the first element but don't compute it yet.
    /// TODO: edit macro to allow eliding the `Result`.
    #[allow(clippy::unnecessary_wraps)]
    #[inline(always)]
    pub const fn new(iter: Iter) -> Result<Self, &'static str> {
        Ok(Self {
            iter,
            cache: vec![],
            index: 0,
        })
    }
    /// Lazily calculate the current value or read it from a cache if it's already been computed.
    #[inline(always)]
    pub fn current(&mut self) -> Option<Iter::Item> {
        self.cache.get(self.index).cloned().or_else(|| {
            self.iter.next().map(|v| {
                self.cache.push(v.clone());
                v
            })
        })
    }
    /// Advance to the next item. Note that this does _not_ calculate the next item; that's done lazily in `current`.
    #[inline(always)]
    pub fn next(&mut self) -> Option<()> {
        self.index.checked_add(1).map(|incr| self.index = incr)
    }
    /// Restart the iterator so that the next `current` call returns element #0.
    #[inline(always)]
    pub fn rewind(&mut self) {
        self.index = 0;
    }
}

impl<Head: Iterator, Tail: crate::BreadthFirst> crate::BreadthFirst
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
            if let Some(tail) = self.tail.advance(index_sum.checked_sub(self.iter.index)?) {
                return self.iter.current().map(|v| (v, tail));
            }
            (self.iter.index < index_sum).then(|| self.iter.next())??; // Comparison is just an optimization, not logically necessary
            self.tail.rewind();
        }
    }
    #[inline(always)]
    fn rewind(&mut self) {
        self.iter.rewind();
        self.tail.rewind();
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

breadth_first_zip_macros::implement!(true); // Implement traits for (A,), (A, B), (A, B, C), (A, B, C, D), ...
