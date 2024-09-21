use std::collections::VecDeque;
use std::fmt;

#[derive(Debug, Clone)]
pub(crate) struct BWTString {
    pub(crate) inner: VecDeque<u8>,
    pub(crate) sentinal_index: usize,
}

impl BWTString {
    pub fn new(inner: impl Into<VecDeque<u8>>) -> Self {
        let inner = inner.into();
        let len = inner.len();
        Self {
            inner,
            sentinal_index: len,
        }
    }

    pub fn new_with_sentinal(inner: impl Into<VecDeque<u8>>, sentinal_index: usize) -> Self {
        let inner = inner.into();
        Self {
            inner,
            sentinal_index,
        }
    }

    pub(crate) fn rotate(&mut self) {
        if self.inner.is_empty() {
            return;
        }

        // Perform rotation
        let front = self.inner.pop_front().unwrap();
        self.inner.push_back(front);

        // Update sentinal_index
        self.sentinal_index = (self.sentinal_index + self.len() - 1) % self.len();
    }

    pub(crate) fn all_rotations_sorted(&self) -> Vec<BWTString> {
        let mut rotations = self.all_rotations();
        Self::lex_sort(&mut rotations);
        rotations
    }

    pub(crate) fn all_rotations(&self) -> Vec<BWTString> {
        let mut rotations = Vec::new();
        let mut cur = self.clone();

        rotations.push(cur.clone());
        for _ in 0..self.len() {
            cur.rotate();
            rotations.push(cur.clone());
        }
        rotations
    }

    pub fn forward_transform(&self) -> Self {
        let rotations = self.all_rotations_sorted();

        let inner = rotations
            .iter()
            .filter_map(|rotation| {
                if rotation.sentinal_index == self.len() {
                    None
                } else {
                    Some(rotation.inner.iter().last().unwrap().clone())
                }
            })
            .collect();

        let sentinal_index = rotations
            .iter()
            .position(|rotation| rotation.sentinal_index == self.len())
            .unwrap();

        Self {
            inner,
            sentinal_index,
        }
    }

    pub(crate) fn lex_sort(bwt_string_vec: &mut Vec<BWTString>) {
        use std::cmp::Ordering::*;

        bwt_string_vec.sort_by(|a, b| {
            let a_bytes = a.inner.iter().enumerate();
            let b_bytes = b.inner.iter().enumerate();

            let a_sentinal_pos = a.sentinal_index;
            let b_sentinal_pos = b.sentinal_index;

            for ((a_idx, a_byte), (b_idx, b_byte)) in a_bytes.zip(b_bytes) {
                let a_char_is_sentinal = a_idx == a_sentinal_pos;
                let b_char_is_sentinal = b_idx == b_sentinal_pos;

                match (a_char_is_sentinal, b_char_is_sentinal) {
                    (true, true) => continue,                     // Both characters are sentinals, continue to next character
                    (true, false) => return Less,                 // a should be sorted before than b. a had sentinal and b had another byte
                    (false, true) => return Greater,              // Same thing as above, just inverted
                    (false, false) => return a_byte.cmp(&b_byte), // Both are regular bytes, compare normally
                }
            }

            // If we've reached here, then all bytes of the BWTString's (so far) were equal.
            // Fallback to string length
            a.inner.len().cmp(&b.inner.len())
        })
    }

    pub fn len(&self) -> usize {
        self.inner.len()
    }
}

impl fmt::Display for BWTString {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut queue = self
            .inner
            .clone()
            .iter()
            .map(|u8| u8.to_string())
            .collect::<Vec<_>>();
        queue.insert(self.sentinal_index, "$".into());

        write!(f, "{}", queue.join(", "))
    }
}
