/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */

//! Breadth-first exhaustive `zip` for repeatable iterators.
//! Behavior matches the following pseudocode specification:
//! - Initialize a counter `i` at zero.
//! - When propmted, pull the first element from each iterator.
//!     - If any iterator is empty, return `None`.
//! - When prompted again, advance only the last iterator.
//! - Continue to do so until the last iterator terminates or reaches its `i`th element.
//!     - When it does so, reset it and pull the next element from the second-to-last iterator.
//! - Repeat this process until we exhaust the first iterator.
//!     - When you've done that, increase `i` and repeat.
//! - Once `i` exceeds the longest iterator's length, we're done: return `None`.

#![cfg_attr(not(test), no_std)]
#![deny(warnings)]
#![warn(
    clippy::all,
    clippy::missing_docs_in_private_items,
    clippy::nursery,
    clippy::pedantic,
    clippy::restriction,
    clippy::cargo,
    missing_docs,
    rustdoc::all
)]
#![allow(
    clippy::blanket_clippy_restriction_lints,
    clippy::cognitive_complexity,
    clippy::expect_used,
    clippy::implicit_return,
    clippy::inline_always,
    clippy::needless_borrowed_reference,
    clippy::panic,
    clippy::question_mark_used,
    clippy::separated_literal_suffix,
    clippy::string_add,
    clippy::unwrap_used
)]

#[cfg(test)]
mod test;

/// End of a recursive implementation of a breadth-first exhaustive `zip`.
#[derive(Clone)]
pub struct BaseCase(bool);

/// Index and value.
#[derive(Clone, Debug)]
struct Indexed<Type> {
    /// Index.
    index: usize,
    /// Value.
    value: Type,
}

/// Recursive implementation of a breadth-first exhaustive `zip`.
#[derive(Clone)]
pub struct BreadthFirstZipped<Head: Iterator + Clone, Tail: BreadthFirst>
where
    Head::Item: Clone,
{
    /// Enumerated iterator for this current "index" in the recursive scheme.
    head: ::core::iter::Enumerate<Head>,
    /// Copy of the original iterator to allow rewinding later.
    orig_head: ::core::iter::Enumerate<Head>,
    /// Implementations for the rest of the list.
    tail: Tail,
    /// Current value: we don't always advance every call.
    current: Indexed<Head::Item>,
}

impl<Head: Iterator + Clone, Tail: BreadthFirst> BreadthFirstZipped<Head, Tail>
where
    Head::Item: Clone,
{
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
            .map(|(index, value)| Indexed { index, value })?;
        Ok(Self {
            head,
            orig_head,
            tail,
            current,
        })
    }
}

/// Flatten a nested tuple like `(A, (B, (C, ())))` to a flat one like `(A, B, C)`
pub trait Flatten {
    /// Flat tuple, e.g. `(A, B, C)`, not `(A, (B, (C, ())))`.
    type Flattened;
    /// Flatten e.g. `(A, (B, (C, ())))` into `(A, B, C)`.
    #[must_use]
    fn flatten(self) -> Self::Flattened;
}

impl Flatten for () {
    type Flattened = Self;
    #[inline(always)]
    #[must_use]
    fn flatten(self) -> Self::Flattened {}
}

/// Sealed traits.
mod sealed {
    /// Either `BaseCase` or `BreadthFirst<Whatever, ...>` eventually ending in `BaseCase` on the right-hand side.
    pub trait BreadthFirst {}
    impl BreadthFirst for super::BaseCase {}
    impl<Head: Iterator + Clone, Tail: super::BreadthFirst> BreadthFirst
        for super::BreadthFirstZipped<Head, Tail>
    where
        Head::Item: Clone,
    {
    }
}

/// Helper trait returning a nested list that will be turned into a flat list for a huge but finite range of tuple sizes.
pub trait BreadthFirst: Clone + sealed::BreadthFirst {
    /// Depth of recursion.
    const DEPTH: usize;
    /// Output of `advance` if successful.
    type Advance: Flatten;
    /// Fallibly choose the next output.
    #[must_use]
    fn advance(&mut self, index_sum: usize) -> Option<Self::Advance>;
    /// Rewind the iterator back to its starting point
    fn rewind(&mut self);
}

impl BreadthFirst for BaseCase {
    const DEPTH: usize = 0;
    type Advance = ();
    #[inline(always)]
    #[must_use]
    fn advance(&mut self, index_sum: usize) -> Option<Self::Advance> {
        (index_sum == 0 && self.0).then(|| {
            self.0 = false;
        })
    }
    #[inline(always)]
    fn rewind(&mut self) {
        self.0 = true;
    }
}

impl<Head: Iterator + Clone, Tail: BreadthFirst> BreadthFirst for BreadthFirstZipped<Head, Tail>
where
    Head::Item: Clone,
    (Head::Item, Tail::Advance): Flatten,
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
            self.current = (self.current.index < index_sum).then(|| {
                self.head
                    .next()
                    .map(|(index, value)| Indexed { index, value })
            })??;
            self.tail.rewind();
        }
    }
    #[inline(always)]
    fn rewind(&mut self) {
        self.head = self.orig_head.clone();
        self.tail.rewind();
        self.current = self
            .head
            .next()
            .map(|(index, value)| Indexed { index, value })
            .unwrap();
    }
}

/// Unflatten a tuple like `(A, B, C)` to `BreadthFirstZipped<A, BreadthFirstZipped<B, BreadthFirstZipped<C, BaseCase>>>`.
trait Unflatten {
    /// E.g. `BreadthFirstZipped<A, BreadthFirstZipped<B, BreadthFirstZipped<C, BaseCase>>>`, not `(A, B, C)`.
    type Unflattened: BreadthFirst;
    /// Unflatten a tuple like `(A, B, C)` to `BreadthFirstZipped<A, BreadthFirstZipped<B, BreadthFirstZipped<C, BaseCase>>>`.
    fn unflatten(self) -> Result<Self::Unflattened, &'static str>;
}

impl Unflatten for () {
    type Unflattened = BaseCase;
    #[inline(always)]
    fn unflatten(self) -> Result<Self::Unflattened, &'static str> {
        Ok(BaseCase(true))
    }
}

/// Helper struct for a breadth-first zip: a counter controlling the maximum index sum of the internal recursive implementation.
pub struct BreadthFirstManager<Tail: BreadthFirst> {
    /// Recursive implementation.
    tail: Tail,
    /// "Global" counter to allow the maximum possible sum of indices.
    index_sum: usize,
}

impl<Tail: BreadthFirst> BreadthFirstManager<Tail> {
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
    type Nested: BreadthFirst;
    /// Lazy breadth-first exhaustive `zip` that guarantees a monotonically increasing sum of indices.
    /// # Errors
    /// If any iterator is empty.
    fn breadth_first_zip(self) -> Result<BreadthFirstManager<Self::Nested>, &'static str>;
}

impl BreadthFirstZip for () {
    type Nested = BaseCase;
    #[inline(always)]
    fn breadth_first_zip(self) -> Result<BreadthFirstManager<Self::Nested>, &'static str> {
        self.unflatten().map(BreadthFirstManager::new)
    }
}

#[allow(clippy::missing_trait_methods)]
impl<Tail: BreadthFirst> Iterator for BreadthFirstManager<Tail> {
    type Item = <Tail::Advance as Flatten>::Flattened;
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
            .map(Flatten::flatten)
    }
}

breadth_first_zip_macros::implement!(); // Implement traits for (A,), (A, B), (A, B, C), (A, B, C, D), ...
