//! Tune word comparisons via Hamming distance

/// Step 0: Baseline computation --- nothing clever here
pub mod basic_str {

    /// Naive computation of Hamming distance between two strings
    pub fn dist(s1: &str, s2: &str) -> isize {
        s1.chars()
            .zip(s2.chars())
            .map(|(c1, c2)| if c1 == c2 { 0 } else { 1 })
            .sum::<isize>()
            + ((s1.len() as isize) - (s2.len() as isize)).abs()
    }

    /// Compute vector of mean distances for all strings
    pub fn mean_dists(dict: &[String]) -> Vec<f64> {
        let mut dists: Vec<isize> = vec![0; dict.len()];
        for (i, w1) in dict.iter().enumerate() {
            for w2 in dict.iter() {
                dists[i] += dist(w1, w2);
            }
        }
        dists
            .iter()
            .map(|d| (*d as f64) / (dict.len() as f64))
            .collect()
    }

    /// Standardized interface
    pub fn mean_dists_dict(dict: &[String]) -> Vec<f64> {
        mean_dists(dict)
    }
}

/// Step 1: Blocked computation
pub mod block_str {

    use super::basic_str::dist; // We can use the naive string compare
    const BSIZE: usize = 500; // Probably want a constant block size param

    /// Compute vector of mean distances for all strings (you may change
    /// the interface if you want)
    fn block_updates(d1: &[String], d2: &[String], counts: &mut [isize]) {
        for (w1, count) in d1.iter().zip(counts.iter_mut()) {
            for w2 in d2 {
                *count += dist(w1, w2);
            }
        }
    }

    /// Blocked computation
    pub fn mean_dists(dict: &[String]) -> Vec<f64> {
        let mut counts = vec![0; dict.len()];
        // Reuse each small pair of dictionary blocks before moving on.
        // chunks() also handles a final block with fewer than BSIZE words.
        for (d1, block_counts) in dict.chunks(BSIZE).zip(counts.chunks_mut(BSIZE)) {
            for d2 in dict.chunks(BSIZE) {
                block_updates(d1, d2, block_counts);
            }
        }
        counts
            .iter()
            .map(|count| (*count as f64) / (dict.len() as f64))
            .collect()
    }

    pub fn mean_dists_dict(dict: &[String]) -> Vec<f64> {
        mean_dists(dict)
    }
}

/// Step 2: Removing indirection
pub mod basic_word {

    const WSIZE: usize = 28;
    const BSIZE: usize = 500;

    /// Word storage
    pub struct Word([u8; WSIZE]);

    impl Word {
        /// Create a Word from a string
        pub fn new(s: &str) -> Self {
            let bytes = s.as_bytes();
            let mut w = [0u8; WSIZE];
            w[..bytes.len()].copy_from_slice(bytes);
            Word(w)
        }
    }

    /// Re-pack the dictionary in more condensed form
    fn pack_dict(dict: &[String]) -> Vec<Word> {
        dict.iter().map(|s| Word::new(s)).collect()
    }

    /// Compute the Hamming distance between two Words
    pub(crate) fn dist(w1: &Word, w2: &Word) -> isize {
        w1.0.iter()
            .zip(w2.0.iter())
            .map(|(c1, c2)| (c1 != c2) as isize)
            .sum()
    }

    fn block_updates(d1: &[Word], d2: &[Word], counts: &mut [isize]) { // i think we can factor this out
        for (w1, count) in d1.iter().zip(counts.iter_mut()) {
            for w2 in d2 {
                *count += dist(w1, w2);
            }
        }
    }

    /// Compute vector of mean distances for all words (pre-packed)
    pub fn mean_dists(dict: &[Word]) -> Vec<f64> {
        let mut counts = vec![0; dict.len()];
        for (d1, block_counts) in dict.chunks(BSIZE).zip(counts.chunks_mut(BSIZE)) {
            for d2 in dict.chunks(BSIZE) {
                block_updates(d1, d2, block_counts);
            }
        }
        counts
            .iter()
            .map(|count| (*count as f64) / (dict.len() as f64))
            .collect()
    }

    /// Compute vector of mean distances for all words    
    pub fn mean_dists_dict(dict: &[String]) -> Vec<f64> {
        let pdict = pack_dict(dict);
        mean_dists(&pdict)
    }
}

/// Step 3: SIMD Within a Register
pub mod swar_word {

    /// Word storage
    pub struct Word([u32; 6]); // You may change the internals (eg [u64; 3])

    const BSIZE: usize = 500;

    const LOW: u32 = 0b011111_011111_011111_011111_011111;
    const HIGH: u32 = 0b100000_100000_100000_100000_100000;

    impl Word {
        /// Create packed word
        pub fn new(s: &str) -> Self {
            let mut w = [0u32; 6];
            for (i, b) in s.bytes().enumerate() {
                w[i / 5] |= ((b - b'a' + 1) as u32) << (6 * (i % 5));
            }
            Word(w)
        }
    }

    /// Compute the Hamming distance between two Words
    pub(crate) fn dist(w1: &Word, w2: &Word) -> isize {
        let mut flags = 0u32;
        for i in 0..6 {
            let x = w1.0[i] ^ w2.0[i];
            flags += ((x + LOW) & HIGH) >> 5;
        }
        (flags % 63) as isize
    }

    fn block_updates(d1: &[Word], d2: &[Word], counts: &mut [isize]) { // i think we can factor this out
        for (w1, count) in d1.iter().zip(counts.iter_mut()) {
            for w2 in d2 {
                *count += dist(w1, w2);
            }
        }
    }

    /// Compute vector of mean distances for all words (pre-packed)
    pub fn mean_dists(dict: &[Word]) -> Vec<f64> {
        let mut counts = vec![0; dict.len()];
        for (d1, block_counts) in dict.chunks(BSIZE).zip(counts.chunks_mut(BSIZE)) {
            for d2 in dict.chunks(BSIZE) {
                block_updates(d1, d2, block_counts);
            }
        }
        counts
            .iter()
            .map(|count| (*count as f64) / (dict.len() as f64))
            .collect()
    }

    /// Re-pack the dictionary in more condensed form
    fn pack_dict(dict: &[String]) -> Vec<Word> {
        dict.iter().map(|s| Word::new(s)).collect()
    }

    pub fn mean_dists_dict(dict: &[String]) -> Vec<f64> {
        let pdict = pack_dict(dict);
        mean_dists(&pdict)
    }
}

/// Step 4: Optimized version!
pub mod optimized {

    pub fn mean_dists_dict(dict: &[String]) -> Vec<f64> {
        todo!()
    }
}

#[cfg(test)]
mod test {

    #[test]
    fn test_naive_dist() {
        use super::basic_str::dist;
        assert_eq!(dist("aa", "aaaaa"), 3);
        assert_eq!(dist("aaaaa", "aa"), 3);
        assert_eq!(dist("test", "tilt"), 2);
    }

    // TODO: Add your own module tests!
    #[test]
    fn test_blocked_edge_cases() {
        use super::block_str::mean_dists;
        assert!(mean_dists(&[]).is_empty());
        assert_eq!(mean_dists(&["word".to_string()]), vec![0.0]);
        let dict: Vec<String> = ["", "a", "aa", "a"].iter().map(|s| s.to_string()).collect();
        assert_eq!(mean_dists(&dict), vec![1.0, 0.5, 1.0, 0.5]);
    }

    #[test]
    fn test_blocked_matches_naive_across_block_boundaries() {
        // Cover a partial block, an exact block, and multiple blocks with a tail.
        for len in [499, 500, 501, 1003] {
            let dict: Vec<String> = (0..len)
                .map(|i| {
                    (0..i % 29)
                        .map(|j| (b'a' + ((i * 7 + j * 11) % 26) as u8) as char)
                        .collect()
                })
                .collect();
            assert_eq!(
                super::block_str::mean_dists(&dict),
                super::basic_str::mean_dists(&dict),
                "dictionary length {len}"
            );
            assert_eq!(
                super::basic_word::mean_dists_dict(&dict),
                super::basic_str::mean_dists(&dict),
                "packed dictionary length {len}"
            );
            assert_eq!(
                super::swar_word::mean_dists_dict(&dict),
                super::basic_str::mean_dists(&dict),
                "swar dictionary length {len}"
            );
        }
    }

    #[test]
    fn test_packed_dist() {
        use super::basic_word::{Word, dist};
        assert_eq!(dist(&Word::new("aa"), &Word::new("aaaaa")), 3);
        assert_eq!(dist(&Word::new("aaaaa"), &Word::new("aa")), 3);
        assert_eq!(dist(&Word::new("test"), &Word::new("tilt")), 2);
        assert_eq!(dist(&Word::new("true"), &Word::new("truth")), 2);
    }

    #[test]
    fn test_swar_dist() {
        use super::swar_word::{Word, dist};
        assert_eq!(dist(&Word::new("aa"), &Word::new("aaaaa")), 3);
        assert_eq!(dist(&Word::new("test"), &Word::new("tilt")), 2);
        assert_eq!(dist(&Word::new("aaaaaa"), &Word::new("aaaabb")), 2);
        assert_eq!(dist(&Word::new("zzzzz"), &Word::new("aaaaa")), 5);
    }
}
