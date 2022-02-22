use std::cmp::{Ord, Ordering, PartialOrd, Reverse};
use std::collections::{BTreeSet, BinaryHeap, HashMap, VecDeque};
use std::hash::Hash;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Interval<T>(pub usize, pub usize, pub T)
where
    T: Clone + Eq + Hash;

impl<T> Ord for Interval<T>
where
    T: Clone + Eq + Hash,
{
    fn cmp(&self, other: &Self) -> Ordering {
        self.0.cmp(&other.0)
    }
}

impl<T> PartialOrd for Interval<T>
where
    T: Clone + Eq + Hash,
{
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

#[derive(Debug)]
pub struct Merge<T>
where
    T: Clone + Eq + Hash,
{
    last: usize,
    ends: HashMap<T, BinaryHeap<Reverse<usize>>>,
    queue: VecDeque<Interval<T>>,
}

impl<T> Merge<T>
where
    T: Clone + Eq + Hash + std::fmt::Debug,
{
    pub fn from_iter<I>(iter: I) -> Merge<T>
    where
        I: IntoIterator<Item = Interval<T>>,
    {
        let mut iter = iter.into_iter();
        match iter.next() {
            Some(Interval(start, end, key)) => {
                let last = start;
                let ends = HashMap::from_iter([(key, BinaryHeap::from([Reverse(end)]))]);
                let queue = VecDeque::from({
                    let mut vec = iter.collect::<Vec<_>>();
                    vec.sort();
                    vec
                });
                Merge { last, ends, queue }
            }
            None => Merge {
                last: 0,
                ends: HashMap::new(),
                queue: VecDeque::new(),
            },
        }
    }

    fn peek(&self) -> Option<usize> {
        println!("{:?}", self.ends);
        match (
            self.ends
                .iter()
                .map(|(_, ends)| ends.peek().unwrap().0)
                .min(),
            self.queue.front().map(|&Interval(s, _, _)| s),
        ) {
            (None, None) => None,
            (None, Some(m)) => Some(m),
            (Some(m), None) => Some(m),
            (Some(me), Some(ms)) => Some(usize::min(me, ms)),
        }
    }
}

impl<T> Iterator for Merge<T>
where
    T: Clone + Eq + Hash + Ord + std::fmt::Debug,
{
    type Item = Interval<BTreeSet<T>>;

    fn next(&mut self) -> Option<Self::Item> {
        self.peek().map(|min| {
            println!("{:?}", self);
            let classes = BTreeSet::from_iter(self.ends.keys().cloned());
            // pop all min ends by retaining those whose tops are >min
            self.ends.retain(|key, ends| {
                // if top of heap >min, skip..
                let end = ends.peek().unwrap().0;
                assert!(end >= min, "end={} < min={}", end, min);
                if end != min {
                    return true;
                }

                // pop every heap entry >min
                while let Some(end) = ends.peek() {
                    if end.0 > min {
                        return true;
                    }

                    ends.pop();
                }

                return false;
            });

            // pop all min queued entries
            while let Some(&Interval(start, end, _)) = self.queue.front() {
                assert!(start >= min, "start={} < min={}", start, min);
                if start != min {
                    break;
                }

                let Interval(_, _, key) = self.queue.pop_front().unwrap();
                self.ends
                    .entry(key)
                    .or_insert(BinaryHeap::new())
                    .push(Reverse(end));
            }

            let last = self.last;
            self.last = min;

            Interval(last, min, classes)
        })
    }
}

#[test]
fn test_merge_peek() {
    assert_eq!(Merge::<()>::from_iter([]).peek(), None);
    assert_eq!(Merge::from_iter([Interval(1, 2, ())]).peek(), Some(2));
}

#[test]
fn test_merge_iter() {
    assert_eq!(Merge::<()>::from_iter([]).collect::<Vec<_>>(), vec![]);
    assert_eq!(
        Merge::from_iter([Interval(1, 2, ())]).collect::<Vec<_>>(),
        vec![Interval(1, 2, BTreeSet::from([()]))],
    );
    assert_eq!(
        Merge::from_iter([Interval(1, 2, ()), Interval(2, 3, ())]).collect::<Vec<_>>(),
        vec![
            Interval(1, 2, BTreeSet::from([()])),
            Interval(2, 3, BTreeSet::from([()])),
        ],
    );
    assert_eq!(
        Merge::from_iter([Interval(1, 2, ()), Interval(3, 4, ())]).collect::<Vec<_>>(),
        vec![
            Interval(1, 2, BTreeSet::from([()])),
            Interval(2, 3, BTreeSet::from([])),
            Interval(3, 4, BTreeSet::from([()])),
        ],
    );
    assert_eq!(
        Merge::from_iter([Interval(1, 4, ()), Interval(2, 3, ())]).collect::<Vec<_>>(),
        vec![
            Interval(1, 2, BTreeSet::from([()])),
            Interval(2, 3, BTreeSet::from([()])),
            Interval(3, 4, BTreeSet::from([()])),
        ],
    );
    assert_eq!(
        Merge::from_iter([Interval(1, 2, "a")]).collect::<Vec<_>>(),
        vec![Interval(1, 2, BTreeSet::from(["a"]))],
    );
    assert_eq!(
        Merge::from_iter([Interval(1, 2, "a"), Interval(2, 3, "b")]).collect::<Vec<_>>(),
        vec![
            Interval(1, 2, BTreeSet::from(["a"])),
            Interval(2, 3, BTreeSet::from(["b"])),
        ],
    );
    assert_eq!(
        Merge::from_iter([Interval(1, 2, "a"), Interval(3, 4, "b")]).collect::<Vec<_>>(),
        vec![
            Interval(1, 2, BTreeSet::from(["a"])),
            Interval(2, 3, BTreeSet::from([])),
            Interval(3, 4, BTreeSet::from(["b"])),
        ],
    );
    assert_eq!(
        Merge::from_iter([Interval(1, 4, "a"), Interval(2, 3, "b")]).collect::<Vec<_>>(),
        vec![
            Interval(1, 2, BTreeSet::from(["a"])),
            Interval(2, 3, BTreeSet::from(["a", "b"])),
            Interval(3, 4, BTreeSet::from(["a"])),
        ],
    );
}
