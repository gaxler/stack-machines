#![allow(dead_code)]

use std::ptr::write;

use crate::memory::block::Block;

pub const BLOCK_SIZE_BITS: usize = 16;
pub const BLOCK_SIZE: usize = 1 << (BLOCK_SIZE_BITS - 1);

pub const LINE_SIZE_BITS: usize = 8;
pub const LINE_SIZE: usize = 1 << (LINE_SIZE_BITS - 1);
pub const LINE_COUNT: usize = BLOCK_SIZE / LINE_SIZE;

pub struct BlockMeta {
    line: [bool; LINE_COUNT],
    black_mark: bool,
}

impl BlockMeta {
    fn find_available_hole(&self, start_at_loc: usize) -> Option<(usize, usize)> {
        let mut free_lines = 0usize;
        let mut start: Option<usize> = None;
        let mut stop = 0usize;

        let start_at_line = start_at_loc / LINE_SIZE;

        for (idx, &full_line) in self.line[start_at_line..].iter().enumerate() {
            let abs_idx = start_at_line + idx;

            if !full_line {
                free_lines += 1;
                // skip first not full line, as it might be partially filled
                if (free_lines == 1) && (abs_idx > 0) {
                    continue;
                }

                // identify the second free line
                if start.is_none() {
                    start = Some(abs_idx);
                }

                // Keep extending out available space while we can
                stop = abs_idx + 1;
            }

            // we found a full line after a free line,
            // or we found a full line that is the last line in a block
            // |first valid free| ---------hole--------- |full or end|
            if free_lines > 0 && (full_line || stop > LINE_COUNT) {
                if let Some(start) = start {
                    let cursor = start * LINE_SIZE;
                    let limit = stop * LINE_SIZE;
                    return Some((cursor, limit));
                }
            }

            // we found the first free line, skipped it and found a full line?
            // Not good, the next free line is going to be "the first" again.
            // So we need to reset the whole count
            if full_line {
                free_lines = 0;
                start = None;
            }
        }
        None
    }
}

unsafe fn write_obj<T>(ptr: *const u8, obj: T) {
    write(ptr as *mut T, obj);
}

pub struct BumpBlock {
    cursor: usize,
    limit: usize,
    block: Block,
    meta: Box<BlockMeta>,
}

impl BumpBlock {
    fn inner_alloc(&mut self, alloc_size: usize) -> Option<*const u8> {
        let next_bump = self.cursor + alloc_size;

        if next_bump > self.limit {
            if self.limit < BLOCK_SIZE {
                if let Some((cursor, limit)) = self.meta.find_available_hole(self.limit) {
                    self.cursor = cursor;
                    self.limit = limit;
                    // When does this recursion stops?
                    // Either hole is big enough to fit alloc_size or we reached the end of the block
                    return self.inner_alloc(alloc_size);
                }
            }
            None
        } else {
            let offset = self.cursor;
            self.cursor = next_bump;
            unsafe { Some(self.block.as_ptr().add(offset)) }
        }
    }
}
