//! Bounded single-owner FIFO for interleaved PCM samples.

/// A fixed-capacity ring buffer for audio samples. It never allocates while
/// pushing or popping and drops no samples implicitly: writes report overflow.
#[derive(Debug, Clone)]
pub struct AudioRingBuffer<T: Copy + Default> {
    storage: Vec<T>,
    read: usize,
    len: usize,
}

impl<T: Copy + Default> AudioRingBuffer<T> {
    /// Create a buffer with the requested capacity. Capacity must be non-zero.
    pub fn new(capacity: usize) -> Self {
        assert!(capacity > 0, "audio ring buffer capacity must be non-zero");
        Self {
            storage: vec![T::default(); capacity],
            read: 0,
            len: 0,
        }
    }

    /// Maximum number of samples this buffer can hold.
    pub fn capacity(&self) -> usize {
        self.storage.len()
    }
    /// Number of samples currently available to read.
    pub fn len(&self) -> usize {
        self.len
    }
    /// Whether the buffer contains no samples.
    pub fn is_empty(&self) -> bool {
        self.len == 0
    }
    /// Remaining writable sample slots.
    pub fn available(&self) -> usize {
        self.capacity() - self.len
    }

    /// Append as many samples as fit and return the number accepted.
    pub fn push(&mut self, input: &[T]) -> usize {
        let count = input.len().min(self.available());
        let write = (self.read + self.len) % self.capacity();
        let first = count.min(self.capacity() - write);
        self.storage[write..write + first].copy_from_slice(&input[..first]);
        if count > first {
            self.storage[..count - first].copy_from_slice(&input[first..count]);
        }
        self.len += count;
        count
    }

    /// Remove samples into `output` and return the number copied.
    pub fn pop(&mut self, output: &mut [T]) -> usize {
        let count = output.len().min(self.len);
        let first = count.min(self.capacity() - self.read);
        output[..first].copy_from_slice(&self.storage[self.read..self.read + first]);
        if count > first {
            output[first..count].copy_from_slice(&self.storage[..count - first]);
        }
        self.read = (self.read + count) % self.capacity();
        self.len -= count;
        count
    }

    /// Discard all buffered samples.
    pub fn clear(&mut self) {
        self.read = 0;
        self.len = 0;
    }
}

#[cfg(test)]
mod tests {
    use super::AudioRingBuffer;

    #[test]
    fn wraps_without_reordering_and_reports_capacity() {
        let mut queue = AudioRingBuffer::new(4);
        assert_eq!(queue.push(&[1, 2, 3]), 3);
        let mut first = [0; 2];
        assert_eq!(queue.pop(&mut first), 2);
        assert_eq!(first, [1, 2]);
        assert_eq!(queue.push(&[4, 5, 6, 7]), 3);
        let mut rest = [0; 5];
        assert_eq!(queue.pop(&mut rest), 4);
        assert_eq!(&rest[..4], &[3, 4, 5, 6]);
        assert!(queue.is_empty());
    }

    #[test]
    fn clear_resets_queue() {
        let mut queue = AudioRingBuffer::new(2);
        assert_eq!(queue.push(&[8, 9]), 2);
        queue.clear();
        assert_eq!(queue.available(), 2);
    }
}
