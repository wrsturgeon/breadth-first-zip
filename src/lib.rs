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

use ::core::{cell::Cell, convert::Infallible, marker::PhantomData};
use reiterator::{Reiterate, Reiterator};

#[cfg(test)]
mod test;

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

breadth_first_zip_macros::implement_flatten!();

/// End of a recursive implementation of a breadth-first exhaustive `zip`.
#[derive(Clone, Debug, Default, Eq, Ord, PartialEq, PartialOrd)]
pub struct BaseCase(Cell<bool>);

/// Sealed traits.
mod sealed {
    /// Either `BaseCase` or a sequence `BreadthFirst<Whatever, ...>` ending in `BaseCase` on the right-hand side.
    pub trait BreadthFirst {}
    impl BreadthFirst for super::BaseCase {}
    impl<'item, Head: Iterator, Tail: super::BreadthFirst<'item>> BreadthFirst
        for super::BreadthFirstZipped<'item, Head, Tail>
    {
    }
}

/// Helper trait returning a nested list that will be turned into a flat list for a huge but finite range of tuple sizes.
pub trait BreadthFirst<'item>: sealed::BreadthFirst {
    /// Depth of recursion.
    const DEPTH: usize;
    /// Output of `advance` if successful.
    type Advance: Flatten;
    /// Fallibly choose the next output.
    #[must_use]
    fn next(&'item self, index_sum: usize) -> Option<Self::Advance>;
    /// Rewind the iterator back to its starting point
    fn rewind(&self);
}

impl<'item> BreadthFirst<'item> for BaseCase {
    const DEPTH: usize = 0;
    type Advance = ();
    #[inline(always)]
    #[must_use]
    fn next(&self, index_sum: usize) -> Option<Self::Advance> {
        (index_sum == 0 && self.0.get()).then(|| {
            self.0.set(false);
        })
    }
    #[inline(always)]
    fn rewind(&self) {
        self.0.set(true);
    }
}

/// Recursive implementation of a breadth-first exhaustive `zip`.
pub struct BreadthFirstZipped<'item, Head: Iterator, Tail: BreadthFirst<'item>> {
    /// Enumerated caching iterator for this current "index" in the recursive scheme.
    iter: Reiterator<Head>,
    /// Implementations for the rest of the list.
    tail: Tail,
    /// Representation of this struct's lifetime.
    lifetime: PhantomData<&'item Infallible>,
}

impl<'item, Head: Iterator, Tail: BreadthFirst<'item>> BreadthFirstZipped<'item, Head, Tail> {
    /// Initialize a new recursive node of a breadth-first zip implementation.
    #[inline(always)]
    pub fn new(head: Head, tail: Tail) -> Self {
        Self {
            iter: head.reiterate(),
            tail,
            lifetime: PhantomData,
        }
    }
}

impl<'item, Head: Iterator, Tail: BreadthFirst<'item>> BreadthFirst<'item>
    for BreadthFirstZipped<'item, Head, Tail>
where
    Head::Item: 'item,
    (&'item Head::Item, Tail::Advance): Flatten,
{
    const DEPTH: usize = Tail::DEPTH + 1;
    type Advance = (&'item Head::Item, Tail::Advance);
    #[inline(always)]
    #[must_use]
    fn next(&'item self, index_sum: usize) -> Option<Self::Advance> {
        loop {
            if let Some(tail) = self
                .tail
                .next(index_sum.checked_sub(self.iter.index.get())?)
            {
                return self.iter.get().map(|indexed| (indexed.value, tail));
            }
            (self.iter.index.get() < index_sum).then(|| self.iter.next())??; // Comparison is just an optimization, not logically necessary
            self.tail.rewind();
        }
    }
    #[inline(always)]
    fn rewind(&self) {
        self.iter.restart();
        self.tail.rewind();
    }
}

/// Helper struct for a breadth-first zip: a counter controlling the maximum index sum of the internal recursive implementation.
#[derive(Clone, Debug, Default, Eq, Ord, PartialEq, PartialOrd)]
pub struct BreadthFirstManager<'item, Tail: BreadthFirst<'item>> {
    /// Recursive implementation.
    tail: Tail,
    /// "Global" counter to allow the maximum possible sum of indices.
    index_sum: Cell<usize>,
    /// Representation of this struct's lifetime.
    lifetime: PhantomData<&'item Infallible>,
}

impl<'item, Tail: BreadthFirst<'item>> BreadthFirstManager<'item, Tail> {
    /// Initialize a new breadth-first algorithm.
    #[inline(always)]
    #[must_use]
    pub const fn new(tail: Tail) -> Self {
        Self {
            tail,
            index_sum: Cell::new(0),
            lifetime: PhantomData,
        }
    }
    /// Like `Iterator::next` but with a generic lifetime.
    /// Why not implement `Iterator`? <https://stackoverflow.com/questions/68606470/how-to-return-a-reference-when-implementing-an-iterator>
    #[allow(clippy::should_implement_trait)]
    #[inline(always)]
    #[must_use]
    pub fn next(&'item self) -> Option<<Tail::Advance as Flatten>::Flattened> {
        self.tail
            .next(self.index_sum.get())
            .map_or_else(
                || {
                    self.index_sum.set(self.index_sum.get().checked_add(1)?);
                    self.tail.rewind();
                    self.tail.next(self.index_sum.get())
                },
                Some,
            )
            .map(Flatten::flatten)
    }
}

/// Zip a tuple into a lazy breadth-first traversal of each possible combination with a monotonically increasing sum of indices.
pub trait BreadthFirstZip<'item> {
    /// Rearrangement of input into a nested tuple.
    type Nested: BreadthFirst<'item>;
    /// Lazy breadth-first exhaustive `zip` that guarantees a monotonically increasing sum of indices.
    fn breadth_first(self) -> BreadthFirstManager<'item, Self::Nested>;
    /// Unflatten a tuple like `(A, B, C)` to `BreadthFirstZipped<A, BreadthFirstZipped<B, BreadthFirstZipped<C, BaseCase>>>`.
    /// # Errors
    /// If any iterator is empty.
    fn unflatten(self) -> Self::Nested;
}

impl<'item> BreadthFirstZip<'item> for () {
    type Nested = BaseCase;
    #[inline(always)]
    fn breadth_first(self) -> BreadthFirstManager<'item, Self::Nested> {
        BreadthFirstManager::new(self.unflatten())
    }
    #[inline(always)]
    fn unflatten(self) -> Self::Nested {
        BaseCase(Cell::new(true))
    }
}

breadth_first_zip_macros::implement!(); // Implement traits for (A,), (A, B), (A, B, C), (A, B, C, D), ...
