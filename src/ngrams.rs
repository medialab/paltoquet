use std::collections::VecDeque;

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
}

pub trait NgramsIteratorExt<I: Iterator> {
    fn ngrams(self, n: usize) -> NGrams<I>;
}

impl<I: Iterator> NgramsIteratorExt<I> for I
where
    I::Item: Clone,
{
    fn ngrams(self, n: usize) -> NGrams<I> {
        NGrams::new(n, self)
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
        let no_grams = Vec::<Vec<String>>::new();

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
}
