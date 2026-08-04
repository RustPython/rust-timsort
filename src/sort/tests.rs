//! The top sorting algorithm; that is, the modified merge sort we keep
//! talking about.

use crate::{never, ord_t_comparator};

/// Test the sort implementation with an empty list
#[test]
fn empty() {
    let mut list: Vec<u32> = vec![];
    sort(&mut list);
    assert!(list.is_empty());
}

/// Test the sort implementation with a single-element list
#[test]
fn single() {
    let mut list = vec![42];
    sort(&mut list);
    assert!(list[0] == 42);
}

/// Test the sort implementation with a short unsorted list
#[test]
fn unsorted() {
    let mut list = vec![3, 1, 0, 4];
    sort(&mut list);
    assert!(list[0] == 0);
    assert!(list[1] == 1);
    assert!(list[2] == 3);
    assert!(list[3] == 4);
}

/// Test the sort implementation with a short backward list
#[test]
fn reverse() {
    let mut list = vec![21, 18, 7, 1];
    sort(&mut list);
    assert!(list[0] == 1);
    assert!(list[1] == 7);
    assert!(list[2] == 18);
    assert!(list[3] == 21);
}

/// Test the sort implementation with a short unsorted list
#[test]
fn sorted() {
    let mut list = vec![0, 1, 2, 3];
    sort(&mut list);
    assert!(list[0] == 0);
    assert!(list[1] == 1);
    assert!(list[2] == 2);
    assert!(list[3] == 3);
}

/// Make sure the sort is stable.
#[test]
fn stable_npow2() {
    let len = 259;
    let mut key1: usize = 0;
    let mut key2: usize = 0;
    #[derive(Debug)]
    struct Item {
        key1: usize,
        key2: usize,
    }
    let mut list: Vec<Item> = (0..len)
        .map(|_| {
            key1 += 1;
            key1 %= 5;
            key2 += 1;
            Item { key1, key2 }
        })
        .collect();
    crate::sort_by_gt(&mut list, |a, b| a.key1 > b.key1);
    for i in 0..(len - 1) {
        assert!(list[i].key1 <= list[i + 1].key1);
        if list[i].key1 == list[i + 1].key1 {
            assert!(list[i].key2 <= list[i + 1].key2);
        }
    }
}
/// Make sure the sort is stable.
#[test]
fn stable() {
    let len = 256;
    let mut key1: usize = 0;
    let mut key2: usize = 0;
    #[derive(Debug)]
    struct Item {
        key1: usize,
        key2: usize,
    }
    let mut list: Vec<Item> = (0..len)
        .map(|_| {
            key1 += 1;
            key1 %= 5;
            key2 += 1;
            Item { key1, key2 }
        })
        .collect();
    crate::sort_by_gt(&mut list, |a, b| a.key1 > b.key1);
    for i in 0..(len - 1) {
        assert!(list[i].key1 <= list[i + 1].key1);
        if list[i].key1 == list[i + 1].key1 {
            assert!(list[i].key2 <= list[i + 1].key2);
        }
    }
}

/// Make sure the comparison count stays O(n log n) on random input. This
/// regresses if `merge_collapse` stops maintaining the run stack invariant,
/// which makes the merge pattern quadratic.
#[test]
fn merge_collapse_comparison_bound() {
    use core::cell::Cell;
    let n: usize = 100_000;
    // xorshift64* so the input is deterministic.
    let mut state: u64 = 0xC0FFEE ^ (n as u64) | 1;
    let mut list: Vec<u32> = (0..n)
        .map(|_| {
            state ^= state >> 12;
            state ^= state << 25;
            state ^= state >> 27;
            (state.wrapping_mul(0x2545F4914F6CDD1D) >> 33) as u32
        })
        .collect();
    let count = Cell::new(0u64);
    crate::sort_by(&mut list, |a, b| {
        count.set(count.get() + 1);
        a.cmp(b)
    });
    for pair in list.windows(2) {
        assert!(pair[0] <= pair[1]);
    }
    // 3 * n * ceil(log2(n)) is generous for a merge sort (measured ~1.7x
    // n log2 n here), while a quadratic merge pattern exceeds it by more
    // than an order of magnitude at this size.
    let log2_n = u64::from(usize::BITS - (n - 1).leading_zeros());
    let bound = 3 * (n as u64) * log2_n;
    assert!(
        count.get() <= bound,
        "{} comparisons at n={} exceeds bound {}",
        count.get(),
        n,
        bound
    );
}

/// Sort implementation convenience used for tests.
fn sort<T: Ord>(list: &mut [T]) {
    super::SortState::new(list, &ord_t_comparator())
        .sort()
        .unwrap_or_else(never)
}
