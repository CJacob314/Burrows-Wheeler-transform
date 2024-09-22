use std::cmp;
use std::collections::VecDeque;
use std::fmt;
use std::io;

#[derive(Debug, Clone, PartialEq, Eq)]
enum BWTByte {
    Byte(u8),
    Sentinel,
}

#[derive(Debug, Clone)]
pub(crate) struct BWTStr {
    inner: VecDeque<BWTByte>,
    sentinel_index: usize,
}

impl BWTStr {
    pub fn new(inner: impl Into<VecDeque<u8>>) -> Self {
        let mut inner = inner
            .into()
            .iter()
            .map(|b| BWTByte::Byte(*b))
            .collect::<VecDeque<_>>();
        inner.push_back(BWTByte::Sentinel);
        let len = inner.len();

        Self {
            inner,
            sentinel_index: len,
        }
    }

    pub fn new_with_sentinal(inner: impl Into<VecDeque<u8>>, sentinal_index: usize) -> Self {
        let mut inner = inner
            .into()
            .iter()
            .map(|b| BWTByte::Byte(*b))
            .collect::<VecDeque<_>>();
        inner.insert(sentinal_index, BWTByte::Sentinel);

        Self {
            inner,
            sentinel_index: sentinal_index,
        }
    }

    pub fn forward_transform(&self) -> Self {
        let rotations = self.all_rotations_sorted();

        let inner = rotations
            .iter()
            .filter_map(|rotation| {
                if rotation.sentinel_index == self.len() {
                    None
                } else {
                    Some(rotation.inner.iter().last().unwrap().clone())
                }
            })
            .collect();

        let sentinal_index = rotations
            .iter()
            .position(|rotation| rotation.sentinel_index == self.len())
            .unwrap();

        Self {
            inner,
            sentinel_index: sentinal_index,
        }
    }

    pub fn reverse_transform(&self) -> Self {
        enum Column {
            Left,
            Right,
        }
        use BWTByte::*;
        use Column::*;

        let right = self.clone();
        let left = right.as_sorted();
        let ranks = self.rank_vec();

        let mut inner = VecDeque::new();

        let mut col = Left;
        let mut i = 0;
        loop {
            match (&col, &right.inner[i]) {
                (_, Sentinel) => {
                    break;
                }
                (Left, _) => {
                    col = Right;
                }
                (Right, Byte(b)) => {
                    inner.push_front(Byte(*b));

                    let rank = ranks[i];

                    i = left
                        .inner
                        .iter()
                        .enumerate()
                        .filter(|(_, ib)| Byte(*b) == **ib)
                        .nth(rank)
                        .unwrap()
                        .0;

                    col = Left;
                }
            }
        }

        let sentinal_index = inner.len();
        Self {
            inner,
            sentinel_index: sentinal_index,
        }
    }

    pub fn rle_write<F: io::Write>(&self, f: &mut F) -> io::Result<()> {
        use io::{BufWriter, Write};
        use BWTByte::*;

        // First, create a BufWriter
        let mut writer = BufWriter::new(f);

        // Write first the position of the sentinal character
        writer.write(self.sentinel_index.to_le_bytes().as_slice())?;

        // Now, the run-length encoding
        let mut iter = self.inner.iter().peekable();
        while let Some(b) = iter.peek() {
            match **b {
                Byte(b) => {
                    let mut cnt = 1_u16;
                    iter.next(); // Consume this occurrence of b

                    while let Some(ibwt) = iter.peek() {
                        if ibwt.is_byte_and(|byte| byte == &b) {
                            cnt += 1;
                            iter.next();
                        } else {
                            break;
                        }

                        // We only write two bytes for the run-length
                        if cnt == u16::MAX {
                            writer.write(&[b])?;
                            writer.write(cnt.to_le_bytes().as_slice())?;
                            cnt = 0;
                        }
                    }

                    // Byte b occurred cnt times in a row before we got to some other byte
                    // Write the byte first, then two bytes for the number of times we saw it
                    writer.write(&[b])?;
                    writer.write(cnt.to_le_bytes().as_slice())?;
                }
                Sentinel => {
                    iter.next();
                    continue;
                }
            }
        }

        writer.flush()?;
        Ok(())
    }

    fn rotate(&mut self) {
        if self.inner.is_empty() {
            return;
        }

        // Perform rotation
        let front = self.inner.pop_front().unwrap();
        self.inner.push_back(front);

        // Update sentinal_index
        self.sentinel_index = (self.sentinel_index + self.len() - 1) % self.len();
    }

    fn all_rotations_sorted(&self) -> Vec<BWTStr> {
        let mut rotations = self.all_rotations();
        Self::lex_sort(&mut rotations);
        rotations
    }

    fn all_rotations(&self) -> Vec<BWTStr> {
        let mut rotations = Vec::new();
        let mut cur = self.clone();

        rotations.push(cur.clone());
        for _ in 0..self.len() {
            cur.rotate();
            rotations.push(cur.clone());
        }
        rotations
    }

    fn lex_sort(bwt_string_vec: &mut Vec<BWTStr>) {
        bwt_string_vec.sort_by(|a, b| a.inner.iter().cmp(b.inner.iter()));
    }

    fn as_sorted(&self) -> Self {
        let mut inner = self.inner.clone();
        inner.make_contiguous().sort_by(|a, b| a.cmp(&b));

        Self {
            inner,
            sentinel_index: 0,
        }
    }

    fn rank_vec(&self) -> Vec<usize> {
        use BWTByte::*;

        let mut num_occurrences = [0_usize; Self::BYTE_RANGE];
        let mut ranks = Vec::with_capacity(self.len());

        for bwt_byte in &self.inner {
            match bwt_byte {
                Sentinel => ranks.push(0), // We only ever have one sentinel
                Byte(b) => {
                    let rank = num_occurrences[*b as usize];
                    ranks.push(rank);
                    num_occurrences[*b as usize] += 1;
                }
            }
        }

        ranks
    }

    pub fn len(&self) -> usize {
        self.inner.len()
    }

    const BYTE_RANGE: usize = 256;
}

impl fmt::Display for BWTStr {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let queue = self
            .inner
            .clone()
            .iter()
            .map(|bwt_byte| match bwt_byte {
                BWTByte::Sentinel => "$".into(),
                BWTByte::Byte(b) => b.to_string(),
            })
            .collect::<Vec<_>>();

        write!(f, "{}", queue.join(", "))
    }
}

impl cmp::PartialOrd for BWTByte {
    fn partial_cmp(&self, other: &Self) -> Option<cmp::Ordering> {
        use cmp::Ordering::*;
        use BWTByte::*;

        Some(match (self, other) {
            (Sentinel, Sentinel) => Equal,
            (Sentinel, Byte(_)) => Less,
            (Byte(_), Sentinel) => Greater,
            (Byte(a), Byte(b)) => a.cmp(&b),
        })
    }
}

impl cmp::Ord for BWTByte {
    fn cmp(&self, other: &Self) -> cmp::Ordering {
        self.partial_cmp(other).unwrap()
    }
}

impl BWTByte {
    fn is_sentinel(&self) -> bool {
        use BWTByte::*;

        match self {
            Byte(_) => false,
            Sentinel => true,
        }
    }

    fn is_byte_and<P: FnOnce(&u8) -> bool>(&self, predicate: P) -> bool {
        use BWTByte::*;

        match self {
            Sentinel => false,
            Byte(b) => predicate(b),
        }
    }
}
