//! The top sorting algorithm; that is, the modified merge sort we keep
//! talking about.

#[cfg(test)]
mod tests;

use crate::find_run::get_run;
use crate::insort;
use crate::merge::merge;
use crate::Comparator;
use alloc::vec::Vec;
use core::cmp::min;

/// Minimum run length to merge; anything shorter will be lengthend and
/// sorted using `insort::sort`.
const MIN_MERGE: usize = 64;

/// Compute the actual minimum merge size for a particular list.
fn calc_min_merge(mut len: usize) -> usize {
    if len < MIN_MERGE {
        len
    } else {
        let mut r: usize = 0;
        while len >= MIN_MERGE {
            r |= len & 1;
            len >>= 1;
        }
        len + r
    }
}

/// Represents a known-sorted sublist.
#[derive(Copy, Clone, Debug)]
struct Run {
    pos: usize,
    len: usize,
}

/// All the ongoing state of the sort.
struct SortState<'a, T, C: Comparator<T>> {
    /// The list that is being sorted.
    list: &'a mut [T],
    /// The comparator function. Should return true if the first argument is
    /// greater than the second.
    cmp: &'a C,
    /// The list of known-sorted sections of the list that can be merged.
    /// To keep the size of this list down, this invariant is preserved:
    ///  - `runs.len < 3 || runs[i-2].len > runs[i-1].len + runs[i].len`
    ///  - `runs.len < 2 || runs[i-1].len > runs[i].len`
    runs: Vec<Run>,
    /// The current position in the list. When `pos == list.len()`, we can now
    /// merge the last of the runs, and we're done.
    pos: usize,
}

impl<'a, T, C: Comparator<T>> SortState<'a, T, C> {
    #[inline]
    fn new(list: &'a mut [T], cmp: &'a C) -> SortState<'a, T, C> {
        SortState {
            list,
            cmp,
            runs: Vec::new(),
            pos: 0,
        }
    }

    /// The outer loop. Find runs, and move forward.
    fn sort(&mut self) -> Result<(), C::Error> {
        let list_len = self.list.len();
        // Minimum run size to use merge sort on. Any sorted sections of the
        // list that are shorter than this are lengthened using `insort::sort`.
        let min_run = calc_min_merge(list_len);
        while self.pos < list_len {
            let pos = self.pos;
            let mut run_len = get_run(&mut self.list[pos..], self.cmp)?;
            let run_min_len = min(min_run, list_len - pos);
            if run_len < run_min_len {
                run_len = run_min_len;
                let l = &mut self.list[pos..][..run_len];
                insort::sort(l, self.cmp)?;
            }
            self.runs.push(Run { pos, len: run_len });
            self.pos += run_len;
            self.merge_collapse()?;
        }
        self.merge_force_collapse()?;
        Ok(())
    }

    /// Merge the runs if they're too big.
    /// Copied almost verbatim from
    /// http://envisage-project.eu/proving-android-java-and-python-sorting-algorithm-is-broken-and-how-to-fix-it/#sec3.2
    fn merge_collapse(&mut self) -> Result<(), C::Error> {
        let runs = &mut self.runs;
        while runs.len() > 1 {
            let l = runs.len();
            let (pos1, pos2) = if (l >= 3 && runs[l - 3].len <= runs[l - 2].len + runs[l - 1].len)
                || (l >= 4 && runs[l - 4].len <= runs[l - 2].len + runs[l - 3].len)
            {
                if runs[l - 3].len < runs[l - 1].len {
                    (l - 3, l - 2)
                } else {
                    (l - 2, l - 1)
                }
            } else if runs[l - 2].len <= runs[l - 1].len {
                (l - 2, l - 1)
            } else {
                break; // Invariant established.
            };
            let (run1, run2) = (runs[pos1], runs[pos2]);
            debug_assert_eq!(run1.pos + run1.len, run2.pos);
            runs.remove(pos2);
            runs[pos1] = Run {
                pos: run1.pos,
                len: run1.len + run2.len,
            };
            let l = &mut self.list[run1.pos..][..run1.len + run2.len];
            merge(l, run1.len, self.cmp)?;
        }
        Ok(())
    }

    /// Merge any outstanding runs, at the end.
    fn merge_force_collapse(&mut self) -> Result<(), C::Error> {
        let runs = &mut self.runs;
        while runs.len() > 1 {
            let (mut pos1, mut pos2) = (runs.len() - 2, runs.len() - 1);
            if runs.len() > 2 && runs[runs.len() - 3].len < runs[runs.len() - 1].len {
                pos1 -= 1;
                pos2 -= 1;
            }
            let (run1, run2) = (runs[pos1], runs[pos2]);
            debug_assert_eq!(run1.pos + run1.len, run2.pos);
            runs.remove(pos2);
            runs[pos1] = Run {
                pos: run1.pos,
                len: run1.len + run2.len,
            };
            let l = &mut self.list[run1.pos..][..run1.len + run2.len];
            merge(l, run1.len, self.cmp)?;
        }
        Ok(())
    }
}

/// Sorts the list using merge sort.
pub(crate) fn try_sort_by<T, C: Comparator<T>>(list: &mut [T], cmp: C) -> Result<(), C::Error> {
    if list.len() < MIN_MERGE {
        insort::sort(list, &cmp)
    } else {
        SortState::new(list, &cmp).sort()
    }
}
