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

// #![cfg_attr(not(test), no_std)] // FIXME
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

mod impure;

#[cfg(feature = "alloc")]
mod pure;

#[cfg(test)]
mod test_impure;

#[cfg(all(test, feature = "alloc"))]
mod test_pure;

/// End of a recursive implementation of a breadth-first exhaustive `zip`.
#[derive(Clone)]
pub struct BaseCase(bool);

/// Index and value.
#[derive(Clone, Debug)]
pub struct Indexed<Type> {
    /// Index.
    index: usize,
    /// Value.
    value: Type,
}

/// Construct an `Indexed` from a tuple.
#[allow(clippy::missing_const_for_fn)]
#[inline(always)]
#[must_use]
pub fn indexed<Type>((index, value): (usize, Type)) -> Indexed<Type> {
    Indexed { index, value }
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

breadth_first_zip_macros::implement_flatten!();

/// Sealed traits.
mod sealed {
    /// Either `BaseCase` or `BreadthFirst<Whatever, ...>` eventually ending in `BaseCase` on the right-hand side.
    pub trait BreadthFirst {}
    impl BreadthFirst for crate::BaseCase {}
    #[cfg(feature = "alloc")]
    impl<Head: Iterator, Tail: crate::BreadthFirst> BreadthFirst
        for crate::pure::BreadthFirstZipped<Head, Tail>
    {
    }
    impl<Head: Iterator + Clone, Tail: crate::BreadthFirst> BreadthFirst
        for crate::impure::BreadthFirstZipped<Head, Tail>
    {
    }
}

/// Helper trait returning a nested list that will be turned into a flat list for a huge but finite range of tuple sizes.
pub trait BreadthFirst: sealed::BreadthFirst {
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
