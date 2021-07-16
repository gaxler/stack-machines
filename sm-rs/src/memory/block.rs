use anyhow::{anyhow, Result};
use std::alloc::{alloc, dealloc, Layout};
use std::ptr::NonNull;

pub type BlockPtr = NonNull<u8>;
pub type BlockSize = usize;

pub fn alloc_block(size: BlockSize) -> Result<BlockPtr> {
    let layout = Layout::from_size_align(size, size)?;
    unsafe {
        let p = alloc(layout);
        if p.is_null() {
            return Err(anyhow!("OOM"));
        }
        Ok(BlockPtr::new_unchecked(p))
    }
}

pub fn dealloc_block(ptr: BlockPtr, size: BlockSize) {
    unsafe {
        let layout = Layout::from_size_align_unchecked(size, size);
        dealloc(ptr.as_ptr(), layout);
    }
}

pub struct Block {
    ptr: BlockPtr,
    size: BlockSize,
}

impl Block {
    pub fn new(size: BlockSize) -> Result<Block> {
        if size.is_power_of_two() {
            let ptr = alloc_block(size)?;
            return Ok(Self { ptr, size });
        }
        Err(anyhow!("Not a power of 2 {}", size))
    }

    pub fn as_ptr(&self) -> *const u8 {
        self.ptr.as_ptr()
    }
}

impl Drop for Block {
    fn drop(&mut self) {
        dealloc_block(self.ptr, self.size);
    }
}

#[cfg(test)]
mod tests {

    use super::*;
    use std::mem::size_of;

    fn poly_alloc<T>() {
        let size = size_of::<T>() * 2;
        let mask = size - 1;

        let block = Block::new(size);
        match block {
            Ok(b) => {
                let p = dbg!(b.as_ptr() as usize);
                assert!((p & mask) ^ mask == mask)
            }
            _ => panic!(),
        }
    }

    #[test]
    fn multi_size_alloc() {
        poly_alloc::<u8>();
        poly_alloc::<u16>();
        poly_alloc::<u32>();
        poly_alloc::<u64>();
    }
}
