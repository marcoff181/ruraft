use std::fmt::Debug;

/// Each log entry consists of a term number and an item
#[derive(Clone, Debug, PartialEq, Eq, Default)]
pub struct LogEntry<T: Sized + Clone + PartialEq + Eq + Default + Debug> {
    pub term: i128,
    pub item: T,
}

/// Adds one or more entries to the log and returns a true/false value to indicate success.
///
/// It has the following attributes:
/// 1. Add the first entry onto an empty log always works.
/// # Examples
/// ```
/// # use ::raftmodel::*;
/// let mut log = vec![];
/// assert!(append_entries(
///            &mut log,
///            -1,
///            -1,
///            vec![LogEntry { term: 1, item: "a" }]
///        ));
/// ```
/// 2. The log is not allowed to have holes in it.
/// # Examples
/// ```
/// # use ::raftmodel::*;
/// let mut log = vec![LogEntry { term: 1, item: "a" }];
/// assert!(!append_entries(
///            &mut log,
///            1,
///            1,
///            vec![LogEntry { term: 1, item: "c" }]
///        ));
/// assert_eq!(log, vec![LogEntry { term: 1, item: "a" }]);
/// ```
/// 3. Adding a new entry to the end should work provided prev_term matches
/// # Examples
/// ```
/// # use ::raftmodel::*;
/// let mut log = vec![LogEntry { term: 1, item: "a" }];
/// assert!(append_entries(
///            &mut log,
///            0,
///            1,
///            vec![LogEntry { term: 1, item: "b" }]
///        ));
/// assert_eq!(
///            log,
///            vec![
///                LogEntry { term: 1, item: "a" },
///                LogEntry { term: 1, item: "b" },
///            ],
///        );
/// ```
/// 4. Overwriting an existing entry with the same entry should work and not alter other parts of the log.
/// # Examples
/// ```
/// # use ::raftmodel::*;
/// let mut log = vec![LogEntry { term: 1, item: "a" },LogEntry { term: 1, item: "b" } ];
/// assert!(append_entries(
///            &mut log,
///            -1,
///            -1,
///            vec![LogEntry { term: 1, item: "a" }]
///        ));
/// assert_eq!(
///            log,
///            vec![
///                LogEntry { term: 1, item: "a" },
///                LogEntry { term: 1, item: "b" }
///            ]
///        );
/// ```
/// 5. Overwrite ann existing entry with a different entry (different term). It should work and delete all entries afterwards.
/// # Examples
/// ```
/// # use ::raftmodel::*;
/// let mut log = vec![LogEntry { term: 1, item: "a" },LogEntry { term: 1, item: "b" } ];
/// assert!(append_entries(
///            &mut log,
///            -1,
///            -1,
///            vec![LogEntry { term: 2, item: "c" }]
///        ));
/// assert_eq!(
///            log,
///            vec![
///                LogEntry { term: 2, item: "c" }
///            ]
///        );
/// ```
/// 6. Append empty entries at the end. It should report success if the prev_term matches.
/// # Examples
/// ```
/// # use ::raftmodel::*;
/// let mut log = vec![LogEntry { term: 2, item: "c" } ];
/// assert!(append_entries(
///            &mut log,
///            0,
///            2,
///            vec![]
///        ));
/// assert_eq!(
///            log,
///            vec![
///                LogEntry { term: 2, item: "c" }
///            ]
///        );
/// ```
/// 7. Appending empty entries at the end with a mismatched term number should fail and do nothing.
/// # Examples
/// ```
/// # use ::raftmodel::*;
/// let mut log = vec![LogEntry { term: 2, item: "c" } ];
/// assert!(!append_entries(
///            &mut log,
///            0,
///            3,
///            vec![]
///        ));
/// assert_eq!(
///            log,
///            vec![
///                LogEntry { term: 2, item: "c" }
///            ]
///        );
/// ```

pub fn append_entries<T: Sized + Clone + PartialEq + Eq + Default + Debug>(
    log: &mut Vec<LogEntry<T>>,
    prev_index: i128,
    prev_term: i128,
    mut entries: Vec<LogEntry<T>>,
) -> bool {
    dbg!(prev_index);
    dbg!(log.len());
    if prev_index != -1 && prev_index > (log.len() as i128 - 1) as i128 {
        return false;
    }

    if prev_index != -1 && log[prev_index as usize].term != prev_term {
        return false;
    }

    // dbg!(log.clone());
    // dbg!(entries.clone());
    for (i, (x, y)) in entries
        .iter()
        .zip(log[(prev_index + 1) as usize..].iter())
        .enumerate()
    {
        if y.term != x.term {
            // dbg!(i);
            log.drain((prev_index + 1) as usize + i..);
            break;
        }
    }
    if entries.len() > log.len() - (prev_index + 1) as usize {
        log.resize_with(entries.len() + (prev_index + 1) as usize, Default::default);
    }

    for i in 0..entries.len() {
        log[(prev_index + 1) as usize + i] = entries.remove(i);
    }
    //log[(prev_index + 1) as usize..(prev_index + 1) as usize + entries.len()] = entries[..];
    true
}

#[cfg(test)]
mod tests {
    use super::*;
    fn make_log(terms: Vec<i128>) -> Vec<LogEntry<String>> {
        let mut result = vec![];
        for x in terms {
            result.push(LogEntry {
                term: x,
                item: "a".to_string(),
            });
        }
        result
    }
    #[test]
    fn test_append_entries() {
        let mut log = vec![];
        // Add the first entry onto an empty log (this should always work)
        // For the first entry, information about the prior entry is meaningless. The prev_index is -1,
        // but the prev_term is ignored (there is no prior entry to compare it to).
        assert!(append_entries(
            &mut log,
            -1,
            -1,
            vec![LogEntry { term: 1, item: "a" }]
        ));

        //The log is not allowed to have holes in it.  This operation fails (no entries at [1])
        assert!(!append_entries(
            &mut log,
            1,
            1,
            vec![LogEntry { term: 1, item: "c" }]
        ));
        assert_eq!(log, vec![LogEntry { term: 1, item: "a" }]);

        // Adding a new entry to the end. It should work.
        assert!(append_entries(
            &mut log,
            0,
            1,
            vec![LogEntry { term: 1, item: "b" }]
        ));
        assert_eq!(
            log,
            vec![
                LogEntry { term: 1, item: "a" },
                LogEntry { term: 1, item: "b" },
            ],
        );

        // Overwriting an existing entry with the same entry. This should work and not alter other parts of the log
        assert!(append_entries(
            &mut log,
            -1,
            -1,
            vec![LogEntry { term: 1, item: "a" }]
        ));
        assert_eq!(
            log,
            vec![
                LogEntry { term: 1, item: "a" },
                LogEntry { term: 1, item: "b" }
            ]
        );

        // Overwrite an existing entry with a different entry (different term). This should work and delete all entries afterwards
        assert!(append_entries(
            &mut log,
            -1,
            -1,
            vec![LogEntry { term: 2, item: "c" }]
        ));
        assert_eq!(log, vec![LogEntry { term: 2, item: "c" }]);
    }
    #[test]
    fn test_figure_7() {
        let mut leader_log = make_log(vec![1, 1, 1, 4, 4, 5, 5, 6, 6, 6]);
        let mut log_a = make_log(vec![1, 1, 1, 4, 4, 5, 5, 6, 6]);
        let mut log_b = make_log(vec![1, 1, 1, 4]);
        let mut log_c = make_log(vec![1, 1, 1, 4, 4, 5, 5, 6, 6, 6, 6]);
        let mut log_d = make_log(vec![1, 1, 1, 4, 4, 5, 5, 6, 6, 6, 7, 7]);
        let mut log_e = make_log(vec![1, 1, 1, 4, 4, 4, 4]);
        let mut log_f = make_log(vec![1, 1, 1, 2, 2, 2, 3, 3, 3, 3, 3]);
        let prev_index = leader_log.len() - 1;
        let prev_term = leader_log[prev_index].term;

        // Leader. Should always work.
        assert!(append_entries(
            &mut leader_log,
            prev_index as i128,
            prev_term,
            vec![LogEntry {
                term: 8,
                item: "e".to_string()
            }]
        ));

        // (a) fails. Would cause a hole
        assert!(!append_entries(
            &mut log_a,
            prev_index as i128,
            prev_term,
            vec![LogEntry {
                term: 8,
                item: "e".to_string()
            }]
        ));

        // (b) fails. Would cause a hole
        assert!(!append_entries(
            &mut log_b,
            prev_index as i128,
            prev_term,
            vec![LogEntry {
                term: 8,
                item: "e".to_string()
            }]
        ));

        // (c) works. Overwrites the last entry
        assert!(append_entries(
            &mut log_c,
            prev_index as i128,
            prev_term,
            vec![LogEntry {
                term: 8,
                item: "e".to_string()
            }]
        ));
        assert!(leader_log == log_c);

        // (d) Works. Overwrites last two entries
        assert!(append_entries(
            &mut log_d,
            prev_index as i128,
            prev_term,
            vec![LogEntry {
                term: 8,
                item: "e".to_string()
            }]
        ));
        assert!(leader_log == log_d);

        // (e) Fails. Would cause a hole
        assert!(!append_entries(
            &mut log_e,
            prev_index as i128,
            prev_term,
            vec![LogEntry {
                term: 8,
                item: "e".to_string()
            }]
        ));

        // (f) Fails. Log continuity. prev_term doesn't match.
        assert!(!append_entries(
            &mut log_f,
            prev_index as i128,
            prev_term,
            vec![LogEntry {
                term: 8,
                item: "e".to_string()
            }]
        ));
    }
}
