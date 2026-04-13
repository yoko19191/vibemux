use std::collections::VecDeque;

#[derive(Clone, Debug)]
pub struct RingBufferEntry {
    pub seq: u64,
    pub data: Vec<u8>,
}

pub struct OutputRingBuffer {
    entries: VecDeque<RingBufferEntry>,
    max_lines: usize,
    max_bytes: usize,
    current_bytes: usize,
    next_seq: u64,
}

impl OutputRingBuffer {
    pub fn new() -> Self {
        Self {
            entries: VecDeque::new(),
            max_lines: 10_000,
            max_bytes: 20 * 1024 * 1024, // 20MB
            current_bytes: 0,
            next_seq: 1,
        }
    }

    pub fn with_limits(max_lines: usize, max_bytes: usize) -> Self {
        Self {
            entries: VecDeque::new(),
            max_lines,
            max_bytes,
            current_bytes: 0,
            next_seq: 1,
        }
    }

    pub fn push(&mut self, data: Vec<u8>) -> u64 {
        let seq = self.next_seq;
        self.next_seq += 1;
        self.current_bytes += data.len();

        self.entries.push_back(RingBufferEntry { seq, data });

        while self.entries.len() > self.max_lines
            || (self.current_bytes > self.max_bytes && self.entries.len() > 1)
        {
            if let Some(evicted) = self.entries.pop_front() {
                self.current_bytes -= evicted.data.len();
            }
        }

        seq
    }

    pub fn get_range(&self, from_seq: u64, to_seq: u64) -> Vec<RingBufferEntry> {
        self.entries
            .iter()
            .filter(|e| e.seq >= from_seq && e.seq <= to_seq)
            .cloned()
            .collect()
    }

    pub fn get_all(&self) -> Vec<RingBufferEntry> {
        self.entries.iter().cloned().collect()
    }

    pub fn current_seq(&self) -> u64 {
        if self.next_seq > 1 {
            self.next_seq - 1
        } else {
            0
        }
    }
}

impl Default for OutputRingBuffer {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_push_and_get_all() {
        let mut buf = OutputRingBuffer::new();
        buf.push(b"hello".to_vec());
        buf.push(b"world".to_vec());

        let all = buf.get_all();
        assert_eq!(all.len(), 2);
        assert_eq!(all[0].seq, 1);
        assert_eq!(all[0].data, b"hello");
        assert_eq!(all[1].seq, 2);
        assert_eq!(all[1].data, b"world");
    }

    #[test]
    fn test_current_seq() {
        let mut buf = OutputRingBuffer::new();
        assert_eq!(buf.current_seq(), 0);

        buf.push(b"a".to_vec());
        assert_eq!(buf.current_seq(), 1);

        buf.push(b"b".to_vec());
        assert_eq!(buf.current_seq(), 2);
    }

    #[test]
    fn test_get_range() {
        let mut buf = OutputRingBuffer::new();
        for i in 0..5 {
            buf.push(format!("line{}", i).into_bytes());
        }

        let range = buf.get_range(2, 4);
        assert_eq!(range.len(), 3);
        assert_eq!(range[0].seq, 2);
        assert_eq!(range[1].seq, 3);
        assert_eq!(range[2].seq, 4);
    }

    #[test]
    fn test_eviction_by_max_lines() {
        let mut buf = OutputRingBuffer::with_limits(3, 20 * 1024 * 1024);
        for i in 0..5 {
            buf.push(format!("line{}", i).into_bytes());
        }

        let all = buf.get_all();
        assert_eq!(all.len(), 3);
        assert_eq!(all[0].seq, 3); // oldest surviving entry
        assert_eq!(all[2].seq, 5);
    }

    #[test]
    fn test_eviction_by_max_bytes() {
        // Each entry is 100 bytes, max_bytes is 250 so only ~2 entries fit
        let mut buf = OutputRingBuffer::with_limits(10_000, 250);
        for _ in 0..5 {
            buf.push(vec![0u8; 100]);
        }

        let all = buf.get_all();
        assert_eq!(all.len(), 2);
        assert_eq!(all[0].seq, 4);
        assert_eq!(all[1].seq, 5);
    }

    #[test]
    fn test_get_range_after_eviction() {
        let mut buf = OutputRingBuffer::with_limits(3, 20 * 1024 * 1024);
        for i in 0..10 {
            buf.push(format!("line{}", i).into_bytes());
        }

        // Only seqs 8, 9, 10 should remain
        let range = buf.get_range(1, 5);
        assert_eq!(range.len(), 0); // all evicted

        let range = buf.get_range(8, 10);
        assert_eq!(range.len(), 3);
    }

    #[test]
    fn test_monotonic_seq() {
        let mut buf = OutputRingBuffer::new();
        let s1 = buf.push(b"a".to_vec());
        let s2 = buf.push(b"b".to_vec());
        let s3 = buf.push(b"c".to_vec());
        assert!(s1 < s2);
        assert!(s2 < s3);
    }
}
