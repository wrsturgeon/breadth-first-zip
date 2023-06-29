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
    clippy::expect_used,
    clippy::implicit_return,
    clippy::needless_borrowed_reference,
    clippy::panic,
    clippy::question_mark_used,
    clippy::string_add
)]
#![allow(unreachable_code)] // FIXME

#[cfg(test)]
mod test;

/// End of a recursive implementation of a breadth-first exhaustive `zip`.
#[derive(Clone)]
pub struct BaseCase;

/// Recursive implementation of a breadth-first exhaustive `zip`.
#[derive(Clone)]
pub struct BreadthFirstZipped<Head: Iterator + Clone, Tail: BreadthFirst>
where
    Head::Item: Clone,
{
    head: ::core::iter::Enumerate<Head>,
    tail: Tail,
    orig_tail: Tail,
    current: (usize, Head::Item),
}

impl<Head: Iterator + Clone, Tail: BreadthFirst> BreadthFirstZipped<Head, Tail>
where
    Head::Item: Clone,
{
    /// Initialize a new recursive node of a breadth-first zip implementation.
    #[inline(always)]
    #[must_use]
    pub fn new(head: Head, tail: Tail) -> Result<Self, &'static str> {
        let mut head = head.enumerate();
        let current = head
            .next()
            .ok_or("Tried to breadth-first zip an empty iterator")?;
        Ok(Self {
            head,
            tail: tail.clone(),
            orig_tail: tail,
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
    fn flatten(self) -> Self::Flattened {
        ()
    }
}

mod sealed {
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
    /// Output of `advance` if successful.
    type Advance: Flatten;
    /// Fallibly choose the next output.
    #[must_use]
    fn advance(&mut self, index_sum: usize) -> Option<Self::Advance>;
}

impl BreadthFirst for BaseCase {
    type Advance = ();
    #[inline(always)]
    #[must_use]
    fn advance(&mut self, index_sum: usize) -> Option<Self::Advance> {
        (index_sum == 0).then_some(())
    }
}

impl<Head: Iterator + Clone, Tail: BreadthFirst> BreadthFirst for BreadthFirstZipped<Head, Tail>
where
    Head::Item: Clone,
    (Head::Item, Tail::Advance): Flatten,
{
    type Advance = (Head::Item, Tail::Advance);
    #[inline(always)]
    #[must_use]
    fn advance(&mut self, index_sum: usize) -> Option<Self::Advance> {
        self.tail
            .advance(index_sum.checked_sub(self.current.0)?)
            .map_or_else(
                || {
                    self.current = self.head.next()?;
                    self.tail = self.orig_tail.clone();
                    self.tail.advance(index_sum.checked_sub(self.current.0)?)
                },
                Some,
            )
            .map(|tail| (self.current.1.clone(), tail))
    }
}

/// Unflatten a tuple like `(A, B, C)` to `BreadthFirstZipped<A, BreadthFirstZipped<B, BreadthFirstZipped<C, BaseCase>>>`.
trait Unflatten {
    /// E.g. `BreadthFirstZipped<A, BreadthFirstZipped<B, BreadthFirstZipped<C, BaseCase>>>`, not `(A, B, C)`.
    type Unflattened: BreadthFirst;
    /// Unflatten a tuple like `(A, B, C)` to `BreadthFirstZipped<A, BreadthFirstZipped<B, BreadthFirstZipped<C, BaseCase>>>`.
    #[must_use]
    fn unflatten(self) -> Result<Self::Unflattened, &'static str>;
}

impl Unflatten for () {
    type Unflattened = BaseCase;
    #[inline(always)]
    #[must_use]
    fn unflatten(self) -> Result<Self::Unflattened, &'static str> {
        Ok(BaseCase)
    }
}

// TODO: the above is a bit wasteful--maybe lazily calculate the entire vector then save it for the second, third, ... times
// but also note that this assumes pure functions

/// Helper struct for a breadth-first zip: a counter controlling the maximum index sum of the internal recursive implementation.
pub struct BreadthFirstManager<Bf: BreadthFirst>(Bf, usize);

impl<Bf: BreadthFirst> BreadthFirstManager<Bf> {
    /// Initialize a new breadth-first manager at an index sum of zero.
    #[inline(always)]
    #[must_use]
    pub const fn new(bf: Bf) -> Self {
        Self(bf, 0)
    }
}

/// Zip a tuple into a lazy breadth-first traversal of each possible combination with a monotonically increasing sum of indices.
pub trait BreadthFirstZip {
    /// Rearrangement of input into a nested tuple.
    type Nested: BreadthFirst;
    /// Lazy breadth-first exhaustive `zip` that guarantees a monotonically increasing sum of indices.
    #[must_use]
    fn breadth_first_zip(self) -> Result<BreadthFirstManager<Self::Nested>, &'static str>;
}

impl BreadthFirstZip for () {
    type Nested = BaseCase;
    #[inline(always)]
    #[must_use]
    fn breadth_first_zip(self) -> Result<BreadthFirstManager<Self::Nested>, &'static str> {
        Ok(BreadthFirstManager::new(BaseCase))
    }
}

impl<Bf: BreadthFirst> Iterator for BreadthFirstManager<Bf> {
    type Item = <Bf::Advance as Flatten>::Flattened;
    #[inline(always)]
    #[must_use]
    fn next(&mut self) -> Option<Self::Item> {
        self.0
            .advance(self.1)
            .map_or_else(
                || {
                    self.1 = self.1.checked_add(1)?;
                    self.0.advance(self.1)
                },
                Some,
            )
            .map(Flatten::flatten)
    }
}

bfzip_util::implement_up_to!(); // Implement traits for (A,), (A, B), (A, B, C), (A, B, C, D), ...
