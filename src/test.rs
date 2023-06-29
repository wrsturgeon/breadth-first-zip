/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */

//! Super simple test suite.

use crate::BreadthFirstZip; // trait

#[test]
fn triples() {
    let indices = 0..3_u8;
    let mut iter = (indices.clone(), indices.clone(), indices)
        .breadth_first_zip()
        .unwrap();
    // index sum = 0
    assert_eq!(iter.next(), Some((0, 0, 0))); /* 1 item */
    // index sum = 1
    assert_eq!(iter.next(), Some((0, 0, 1)));
    assert_eq!(iter.next(), Some((0, 1, 0)));
    assert_eq!(iter.next(), Some((1, 0, 0))); /* 3 items */
    // index sum = 2
    assert_eq!(iter.next(), Some((0, 0, 2)));
    assert_eq!(iter.next(), Some((0, 1, 1)));
    assert_eq!(iter.next(), Some((0, 2, 0)));
    assert_eq!(iter.next(), Some((1, 0, 1)));
    assert_eq!(iter.next(), Some((1, 1, 0)));
    assert_eq!(iter.next(), Some((2, 0, 0))); /* 6 items */
    // index sum = 3
    assert_eq!(iter.next(), Some((0, 1, 2)));
    assert_eq!(iter.next(), Some((0, 2, 1)));
    assert_eq!(iter.next(), Some((1, 0, 2)));
    assert_eq!(iter.next(), Some((1, 1, 1)));
    assert_eq!(iter.next(), Some((1, 2, 0)));
    assert_eq!(iter.next(), Some((2, 0, 1)));
    assert_eq!(iter.next(), Some((2, 1, 0))); /* 7 items */
    // index sum = 4
    assert_eq!(iter.next(), Some((0, 2, 2)));
    assert_eq!(iter.next(), Some((1, 1, 2)));
    assert_eq!(iter.next(), Some((1, 2, 1)));
    assert_eq!(iter.next(), Some((2, 0, 2)));
    assert_eq!(iter.next(), Some((2, 1, 1)));
    assert_eq!(iter.next(), Some((2, 2, 0))); /* 6 items */
    // index sum = 5
    assert_eq!(iter.next(), Some((1, 2, 2)));
    assert_eq!(iter.next(), Some((2, 1, 2)));
    assert_eq!(iter.next(), Some((2, 2, 1))); /* 3 items */
    // index sum = 6
    assert_eq!(iter.next(), Some((2, 2, 2))); /* 1 item */
    // index sum too high
    assert_eq!(iter.next(), None);
}

mod qc {
    #![allow(warnings)]
    use super::*;

    type A = usize;
    type B = (usize,);
    type C = ((usize,),);

    quickcheck::quickcheck! {
        fn prop_everything(va: Vec<A>, vb: Vec<B>, vc: Vec<C>, a0: A, b0: B, c0: C) -> bool {
            let va = { let mut va = va; va.push(a0); va.sort_unstable(); va.dedup(); va };
            let vb = { let mut vb = vb; vb.push(b0); vb.sort_unstable(); vb.dedup(); vb };
            let vc = { let mut vc = vc; vc.push(c0); vc.sort_unstable(); vc.dedup(); vc };
            let total_elements = va.len() * vb.len() * vc.len();
            let mut seen = ::std::collections::HashSet::new();
            let mut iter = (va.iter(), vb.iter(), vc.iter()).breadth_first_zip().unwrap();
            for _ in 0..total_elements {
                let Some((a, b, c)) = iter.next() else { panic!("Returned `None` too soon"); return false; };
                if seen.contains(&(a, b, c)) { panic!("Returned an element already seen"); return false; }
                seen.insert((a, b, c));
                if !va.contains(&a) { panic!("`a` not in `A`"); return false; }
                if !vb.contains(&b) { panic!("`b` not in `B`"); return false; }
                if !vc.contains(&c) { panic!("`c` not in `C`"); return false; }
            }
            if iter.next().is_some() { panic!("Kept returning after should have returned `None`"); return false; }
            true
        }
    }
}
