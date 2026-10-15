//! A small, dependency-free parallel map for CPU-bound work such as diffing.

use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Mutex;

/// Applies `work` to every item on up to `workers` threads and returns the
/// results in the order of `items`.
///
/// Items are handed out one at a time, so a few slow items (large screenshots)
/// do not leave the other threads idle. With one worker, or one item, it runs
/// on the calling thread.
pub(crate) fn map<T, R, F>(items: &[T], workers: usize, work: F) -> Vec<R>
where
    T: Sync,
    R: Send,
    F: Fn(&T) -> R + Sync,
{
    let workers = workers.clamp(1, items.len().max(1));
    if workers == 1 {
        return items.iter().map(&work).collect();
    }

    let next = AtomicUsize::new(0);
    let slots: Vec<Mutex<Option<R>>> = items.iter().map(|_| Mutex::new(None)).collect();

    std::thread::scope(|scope| {
        for _ in 0..workers {
            scope.spawn(|| loop {
                let index = next.fetch_add(1, Ordering::Relaxed);
                let Some(item) = items.get(index) else {
                    break;
                };
                let value = work(item);
                *slots[index]
                    .lock()
                    .unwrap_or_else(|poison| poison.into_inner()) = Some(value);
            });
        }
    });

    slots
        .into_iter()
        .map(|slot| {
            slot.into_inner()
                .unwrap_or_else(|poison| poison.into_inner())
                .expect("every item is processed before the scope ends")
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_input_order() {
        let items: Vec<u64> = (0..200).collect();
        let squares = map(&items, 8, |n| n * n);
        assert_eq!(squares, items.iter().map(|n| n * n).collect::<Vec<_>>());
    }

    #[test]
    fn handles_empty_input_and_single_worker() {
        let empty: Vec<u8> = Vec::new();
        assert!(map(&empty, 4, |n| *n).is_empty());
        assert_eq!(map(&[1, 2, 3], 1, |n| n + 1), vec![2, 3, 4]);
    }
}
