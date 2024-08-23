// NOTE: our ngrams are not padded, and return the sequence as
// a gram when n > l.
use std::collections::VecDeque;
use std::ops::{Range, RangeInclusive};

pub fn ngrams_len(tokens: usize, n: usize) -> usize {
    if n < 1 || tokens == 0 {
        return 0;
    }

    tokens.checked_sub(n - 1).unwrap_or(1)
}

pub fn ngrams_range_len(tokens: usize, range: RangeInclusive<usize>) -> usize {
    if tokens == 0 {
        return 0;
    }

    if tokens < *range.start() {
        return 1;
    }

    let mut v: usize = 0;

    for n in range {
        v += ngrams_len(tokens, n);
    }

    v
}

pub struct NGrams<I: Iterator> {
    n: usize,
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
            n,
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
                    if self.deque.len() < self.n {
                        self.deque.push_back(item);
                    } else {
                        return Some(self.rotate(item));
                    }
                }
            }
        }
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        let (lower_bound, upper_bound) = self.inner.size_hint();

        (
            ngrams_len(lower_bound, self.n),
            upper_bound.map(|v| ngrams_len(v, self.n)),
        )
    }
}

pub struct NGramsRange<I: Iterator> {
    deque: VecDeque<I::Item>,
    range: RangeInclusive<usize>,
    window: Option<Range<usize>>,
    first: bool,
    inner: I,
}

impl<I: Iterator> NGramsRange<I>
where
    I::Item: Clone,
{
    fn new(range: RangeInclusive<usize>, inner: I) -> Self {
        if range.start() < &1 {
            panic!("cannot compute ngrams when n < 1");
        }

        let iterator = Self {
            deque: VecDeque::with_capacity(*range.end()),
            range: range.clone(),
            window: Some(0..*range.start()),
            first: true,
            inner,
        };

        iterator
    }

    fn reset_window(&mut self) {
        if self.first {
            self.first = false;
        }

        let s = *self.range.start();
        let e = self.deque.len();

        self.window = Some(e - s..e);
    }

    fn flush(&mut self) -> Option<Vec<I::Item>> {
        if let Some(window) = &self.window {
            let s = window.start;
            let e = window.end;

            let n = e - s;

            let mut buffer = Vec::with_capacity(n);

            for i in s..e {
                buffer.push(self.deque[i].clone());
            }

            // We increase n and widen the window
            if e == self.deque.len() {
                let next_n = n + 1;

                // We reset
                if next_n == self.deque.len() + 1 {
                    self.window = None;
                }
                // We advance to next n in range
                else if self.first {
                    self.window = Some(0..next_n);
                } else {
                    let l = self.deque.len();
                    self.window = Some((l - next_n)..l);
                }
            }
            // We slide the window forward
            else {
                self.window = Some((s + 1)..(e + 1));
            }

            Some(buffer)
        } else {
            None
        }
    }

    fn flush_short(&mut self) -> Option<Vec<I::Item>> {
        if self.deque.is_empty() {
            return None;
        }

        self.window = None;

        let mut buffer = Vec::with_capacity(self.deque.len());

        while let Some(item) = self.deque.pop_front() {
            buffer.push(item);
        }

        Some(buffer)
    }

    fn rotate(&mut self, next_item: I::Item) {
        self.deque.pop_front();
        self.deque.push_back(next_item);
    }
}

impl<I: Iterator> Iterator for NGramsRange<I>
where
    I::Item: Clone,
{
    type Item = Vec<I::Item>;

    fn next(&mut self) -> Option<Self::Item> {
        loop {
            // Initial fill
            if self.deque.len() < *self.range.end() {
                match self.inner.next() {
                    None => {
                        if self.deque.len() < *self.range.start() {
                            return self.flush_short();
                        }

                        self.range = *self.range.start()..=self.deque.len();
                        return self.flush();
                    }
                    Some(item) => {
                        self.deque.push_back(item);
                        continue;
                    }
                };
            }

            // We first check the window
            if let Some(gram) = self.flush() {
                return Some(gram);
            }

            // The consume the inner iterator and we rotate
            match self.inner.next() {
                None => return self.flush(),
                Some(item) => {
                    self.reset_window();
                    self.rotate(item);

                    continue;
                }
            }
        }
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        let inner_size_hint = self.inner.size_hint();

        (
            ngrams_range_len(inner_size_hint.0, self.range.clone()),
            inner_size_hint
                .1
                .map(|v| ngrams_range_len(v, self.range.clone())),
        )
    }
}

pub trait NgramsIteratorExt<I: Iterator> {
    fn ngrams(self, n: usize) -> NGrams<I>;
    fn ngrams_range(self, range: RangeInclusive<usize>) -> NGramsRange<I>;
}

impl<I: Iterator> NgramsIteratorExt<I> for I
where
    I::Item: Clone,
{
    fn ngrams(self, n: usize) -> NGrams<I> {
        NGrams::new(n, self)
    }
    fn ngrams_range(self, range: RangeInclusive<usize>) -> NGramsRange<I> {
        NGramsRange::new(range, self)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tokenizers::WordToken;

    fn collect_ngrams<'a>(target: Vec<&'a str>, n: usize) -> Vec<Vec<&'a str>> {
        target.into_iter().ngrams(n).collect()
    }

    #[test]
    fn test_empty_ngrams() {
        let empty = Vec::<&str>::new();
        let no_grams = Vec::<Vec<&str>>::new();

        assert_eq!(collect_ngrams(empty.clone(), 2), no_grams);
        assert_eq!(
            empty.clone().into_iter().ngrams(2).size_hint(),
            (0, Some(0))
        );
        assert_eq!(
            empty
                .clone()
                .into_iter()
                .ngrams_range(1..=5)
                .collect::<Vec<_>>(),
            no_grams
        );
        assert_eq!(
            empty.clone().into_iter().ngrams_range(1..=5).size_hint(),
            (0, Some(0))
        );
    }

    #[test]
    #[should_panic]
    fn test_irrelvant_n() {
        collect_ngrams(vec!["the", "cat"], 0);
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
            assert_eq!(
                sentence.iter().ngrams(i + 1).size_hint(),
                (grams.len(), Some(grams.len()))
            );
            assert_eq!(collect_ngrams(sentence.clone(), i + 1), grams);
        }
    }

    #[test]
    fn test_ngrams_word_tokens() {
        let sentence = vec![
            WordToken::word("the"),
            WordToken::word("cat"),
            WordToken::word("eats"),
        ];

        assert_eq!(
            sentence.iter().ngrams(2).collect::<Vec<_>>(),
            vec![
                vec![&WordToken::word("the"), &WordToken::word("cat")],
                vec![&WordToken::word("cat"), &WordToken::word("eats")]
            ]
        );
    }

    #[test]
    fn test_ngrams_range() {
        let sentence = vec!["the", "cat", "eats", "the", "mouse"];

        let expected = vec![
            vec!["the"],
            vec!["cat"],
            vec!["eats"],
            vec!["the"],
            vec!["the", "cat"],
            vec!["cat", "eats"],
            vec!["eats", "the"],
            vec!["the", "cat", "eats"],
            vec!["cat", "eats", "the"],
            vec!["the", "cat", "eats", "the"],
            vec!["mouse"],
            vec!["the", "mouse"],
            vec!["eats", "the", "mouse"],
            vec!["cat", "eats", "the", "mouse"],
        ];

        let grams = sentence
            .clone()
            .into_iter()
            .ngrams_range(1..=4)
            .collect::<Vec<_>>();

        assert_eq!(grams, expected);
        assert_eq!(
            sentence.clone().into_iter().ngrams_range(1..=4).size_hint(),
            (14, Some(14))
        );

        // Should produce same grams as non-range counterpart when range is one value
        for n in 1..=4 {
            assert_eq!(
                sentence.iter().ngrams(n).collect::<Vec<_>>(),
                sentence.iter().ngrams_range(n..=n).collect::<Vec<_>>()
            );
            assert_eq!(
                sentence.iter().ngrams(n).size_hint(),
                sentence.iter().ngrams_range(n..=n).size_hint()
            );
        }
    }

    #[test]
    fn test_less_tokens_than_n() {
        let sentence = vec!["the", "cat"];

        // Normal
        assert_eq!(
            sentence.iter().ngrams(5).collect::<Vec<_>>(),
            vec![vec![&"the", &"cat"]]
        );
        assert_eq!(sentence.iter().ngrams(5).size_hint(), (1, Some(1)));

        // Range
        assert_eq!(
            vec!["chat"].iter().ngrams_range(1..=2).collect::<Vec<_>>(),
            vec![vec![&"chat"]]
        );
        assert_eq!(
            vec!["chat"].iter().ngrams_range(1..=2).size_hint(),
            (1, Some(1))
        );

        assert_eq!(
            sentence.iter().ngrams_range(1..=2).collect::<Vec<_>>(),
            vec![vec![&"the"], vec![&"cat"], vec![&"the", &"cat"]]
        );
        assert_eq!(
            sentence.iter().ngrams_range(1..=2).size_hint(),
            (3, Some(3))
        );

        assert_eq!(
            sentence.iter().ngrams_range(1..=3).collect::<Vec<_>>(),
            vec![vec![&"the"], vec![&"cat"], vec![&"the", &"cat"]]
        );
        assert_eq!(
            sentence.iter().ngrams_range(1..=3).size_hint(),
            (3, Some(3))
        );

        assert_eq!(
            sentence.iter().ngrams_range(4..=5).collect::<Vec<_>>(),
            vec![vec![&"the", &"cat"]]
        );
        assert_eq!(
            sentence.iter().ngrams_range(4..=5).size_hint(),
            (1, Some(1))
        );
    }
}
