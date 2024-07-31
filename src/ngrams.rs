use std::collections::VecDeque;
use std::ops::RangeInclusive;

pub struct NGrams<I: Iterator> {
    deque: VecDeque<I::Item>,
    inner: I,
}

impl<I: Iterator> NGrams<I>
where
    I::Item: Clone,
{
    fn new(n: usize, inner: I) -> Self {
        if n < 1 {
            panic!("cannot compute ngrams when n < 1");
        }

        Self {
            deque: VecDeque::with_capacity(n),
            inner,
        }
    }

    fn rotate(&mut self, next_item: I::Item) -> Vec<I::Item> {
        let mut buffer = Vec::with_capacity(self.deque.len());

        for item in self.deque.iter() {
            buffer.push(item.clone());
        }

        self.deque.pop_front();
        self.deque.push_back(next_item);

        buffer
    }

    fn flush(&mut self) -> Option<Vec<I::Item>> {
        if self.deque.is_empty() {
            return None;
        }

        let mut buffer = Vec::with_capacity(self.deque.len());

        while let Some(item) = self.deque.pop_front() {
            buffer.push(item);
        }

        Some(buffer)
    }
}

impl<I: Iterator> Iterator for NGrams<I>
where
    I::Item: Clone,
{
    type Item = Vec<I::Item>;

    fn next(&mut self) -> Option<Self::Item> {
        loop {
            match self.inner.next() {
                None => return self.flush(),
                Some(item) => {
                    if self.deque.len() < self.deque.capacity() {
                        self.deque.push_back(item);
                    } else {
                        return Some(self.rotate(item));
                    }
                }
            }
        }
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        match self.deque.capacity() {
            0 => (0, Some(0)),
            1 => self.inner.size_hint(),
            n => {
                let (l, u) = self.inner.size_hint();
                (l.saturating_sub(n - 1), u.map(|x| x.saturating_sub(n - 1)))
            }
        }
    }
}

pub struct NGramsRange<T> {
    buffer: Vec<T>,
    next_n: usize,
    upper_bound: usize,
    // TODO: genericize this type somehow?
    current_iterator: Option<NGrams<std::vec::IntoIter<T>>>,
}

impl<T> NGramsRange<T> {
    fn new<I>(range: RangeInclusive<usize>, inner: I) -> Self
    where
        I: Iterator<Item = T>,
    {
        if range.start() < &1 {
            panic!("cannot compute ngrams when n < 1");
        }

        // NOTE: it must buffer the input into memory which is
        // hardly optimal...
        Self {
            buffer: inner.collect(),
            next_n: *range.start(),
            upper_bound: *range.end(),
            current_iterator: None,
        }
    }
}

impl<T: Clone> Iterator for NGramsRange<T> {
    type Item = Vec<T>;

    fn next(&mut self) -> Option<Self::Item> {
        loop {
            match &mut self.current_iterator {
                None => {
                    if self.next_n > self.upper_bound {
                        return None;
                    }

                    // TODO: not clone
                    self.current_iterator =
                        Some(NGrams::new(self.next_n, self.buffer.clone().into_iter()));

                    self.next_n += 1;

                    continue;
                }
                Some(inner) => match inner.next() {
                    Some(gram) => return Some(gram),
                    None => {
                        self.current_iterator = None;
                        continue;
                    }
                },
            }
        }
    }
}

pub trait NgramsIteratorExt<I: Iterator> {
    fn ngrams(self, n: usize) -> NGrams<I>;
    fn ngrams_range(self, range: RangeInclusive<usize>) -> NGramsRange<I::Item>;
}

impl<I: Iterator> NgramsIteratorExt<I> for I
where
    I::Item: Clone,
{
    fn ngrams(self, n: usize) -> NGrams<I> {
        NGrams::new(n, self)
    }
    fn ngrams_range(self, range: RangeInclusive<usize>) -> NGramsRange<I::Item> {
        NGramsRange::new(range, self)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn ngrams<'a>(target: Vec<&'a str>, n: usize) -> Vec<Vec<&'a str>> {
        target.into_iter().ngrams(n).collect()
    }

    #[test]
    fn test_empty_ngrams() {
        let empty = Vec::<&str>::new();
        let no_grams = Vec::<Vec<&str>>::new();

        assert_eq!(ngrams(empty, 2), no_grams);
    }

    #[test]
    #[should_panic]
    fn test_irrelvant_n() {
        ngrams(vec!["the", "cat"], 0);
    }

    #[test]
    fn test_ngrams() {
        let sentence = vec!["the", "cat", "eats", "the", "mouse"];

        let tests = vec![
            vec![
                vec!["the"],
                vec!["cat"],
                vec!["eats"],
                vec!["the"],
                vec!["mouse"],
            ],
            vec![
                vec!["the", "cat"],
                vec!["cat", "eats"],
                vec!["eats", "the"],
                vec!["the", "mouse"],
            ],
            vec![
                vec!["the", "cat", "eats"],
                vec!["cat", "eats", "the"],
                vec!["eats", "the", "mouse"],
            ],
            vec![
                vec!["the", "cat", "eats", "the"],
                vec!["cat", "eats", "the", "mouse"],
            ],
        ];

        for (i, grams) in tests.into_iter().enumerate() {
            assert_eq!(ngrams(sentence.clone(), i + 1), grams);
        }
    }

    #[test]
    fn test_ngrams_range() {
        let sentence = vec!["the", "cat", "eats", "the", "mouse"];

        let expected = vec![
            vec!["the"],
            vec!["cat"],
            vec!["eats"],
            vec!["the"],
            vec!["mouse"],
            vec!["the", "cat"],
            vec!["cat", "eats"],
            vec!["eats", "the"],
            vec!["the", "mouse"],
            vec!["the", "cat", "eats"],
            vec!["cat", "eats", "the"],
            vec!["eats", "the", "mouse"],
            vec!["the", "cat", "eats", "the"],
            vec!["cat", "eats", "the", "mouse"],
        ];

        assert_eq!(
            sentence.into_iter().ngrams_range(1..=4).collect::<Vec<_>>(),
            expected
        );
    }
}
