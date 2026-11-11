//! Bounded single-owner FIFO for interleaved PCM samples.
#[derive(Debug, Clone)]
pub struct AudioRingBuffer<T: Copy + Default> { storage: Vec<T>, read: usize, len: usize }
impl<T: Copy + Default> AudioRingBuffer<T> {
 pub fn new(capacity: usize) -> Self { assert!(capacity > 0); Self { storage: vec![T::default(); capacity], read: 0, len: 0 } }
 pub fn capacity(&self) -> usize { self.storage.len() }
 pub fn len(&self) -> usize { self.len }
 pub fn is_empty(&self) -> bool { self.len == 0 }
 pub fn available(&self) -> usize { self.capacity() - self.len }
 pub fn push(&mut self, input: &[T]) -> usize { let count=input.len().min(self.available()); let write=(self.read+self.len)%self.capacity(); let first=count.min(self.capacity()-write); self.storage[write..write+first].copy_from_slice(&input[..first]); if count>first {self.storage[..count-first].copy_from_slice(&input[first..count]);} self.len+=count; count }
 pub fn pop(&mut self, output: &mut [T]) -> usize { let count=output.len().min(self.len); let first=count.min(self.capacity()-self.read); output[..first].copy_from_slice(&self.storage[self.read..self.read+first]); if count>first {output[first..count].copy_from_slice(&self.storage[..count-first]);} self.read=(self.read+count)%self.capacity(); self.len-=count; count }
 pub fn clear(&mut self) { self.read=0; self.len=0; }
}
