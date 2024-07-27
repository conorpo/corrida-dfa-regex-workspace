#![warn(missing_docs)]
#![feature(allocator_api)]
#![feature(generic_const_exprs)]
#![feature(const_trait_impl)]
#![feature(inline_const)]
#![feature(ptr_metadata)]
#![feature(slice_ptr_get)]
#![feature(adt_const_params)]
#![feature(new_uninit)]
#![feature(cell_update)]
#![feature(maybe_uninit_as_bytes)]
#![feature(core_intrinsics)]
#![feature(maybe_uninit_fill)]
#![feature(maybe_uninit_uninit_array)]
#![feature(slice_from_ptr_range)]
#![feature(alloc_layout_extra)]

//! Typed Bump Allocator
//! 
//! - Useful as for cache-friendly accesses of collection.
//! 


pub mod basic_structures;

use std::{alloc::{AllocError, Allocator, Global, GlobalAlloc, Layout}, cell::{Cell, UnsafeCell}, intrinsics::size_of, marker::{PhantomData, PhantomPinned}, mem::{self, MaybeUninit}, ops::Index, ptr::NonNull, slice::{self, from_mut_ptr_range, from_raw_parts_mut}, sync::atomic::{AtomicI32, AtomicU32, Ordering}};

enum Assert<const CHECK: bool> {}
trait IsTrue {}
impl IsTrue for Assert<true> {}

static block_count: AtomicU32 = AtomicU32::new(0);

#[repr(align(256))]
struct Block
{   
    prev: Option<Box<Block>>,
    cur_ptr: NonNull<u8>,
    block_end: NonNull<u8>
}

const BLOCK_DATA_SIZE: usize = 1 << 20;

impl Block
{
    const LAYOUT: Layout = unsafe { 
        Layout::from_size_align_unchecked(BLOCK_DATA_SIZE + size_of::<Self>(), 256)
    };

    fn new(prev: Option<Box<Block>>) -> NonNull<Self> {
        //SAFETY, we are about to immedieatly initialize prev, data will always be initalized
        unsafe {
            let ptr = Global.allocate(Self::LAYOUT).unwrap().as_mut_ptr();
            
            let footer = {
                NonNull::new_unchecked(ptr.add(BLOCK_DATA_SIZE) as *mut Self)
            };

            
            footer.write (Block {
                prev,
                cur_ptr: NonNull::new_unchecked(ptr),
                block_end: NonNull::new_unchecked(footer.as_ptr() as *mut u8)
            });

            footer
        }
    }

    fn alloc(&mut self, size: usize, align: usize) -> Result<NonNull<u8>, AllocError> {
        let align_offset = self.cur_ptr.align_offset(align);
        unsafe {
            let slot_start_ptr = self.cur_ptr.add(align_offset);

            if slot_start_ptr.add(size) > self.block_end {
                Err(AllocError)
            } else {
                self.cur_ptr = slot_start_ptr.add(size);
                Ok(slot_start_ptr)
            }
        }
    }
}


/// One time use bump allocator.
/// Useful for many allocations of the same type
/// Self references are easy because all lifetimes are garunteed to live for the whole arena.
pub struct Corrida 
where
{
    cur_block: Cell<NonNull<Block>>,
    _boo: PhantomData<Block>
}



impl Corrida
{
    pub fn new() -> Self {
        Self {
            cur_block: Cell::new(Block::new(None)),
            _boo: PhantomData
        }
    }

    pub fn alloc<F>(&self, fighter: F) -> &mut F
    {
        //let mut cur_offset = self.cur_offset.get();
        let size: usize = size_of::<F>();
        let layout = align_of::<F>();
        
        unsafe {
            let mut slot = match (*self.cur_block.get().as_ptr()).alloc(size, layout) {
                Ok(slot) => {
                    slot
                },
                Err(_) => {
                    let old_block = Box::from_raw(self.cur_block.get().as_ptr());
                    let mut new_block = Block::new(Some(old_block));

                    let old = block_count.fetch_add(1, Ordering::Relaxed);
                    if old % 100 == 0 {
                        println!("{} blocks", old);
                    }

                    self.cur_block.set(new_block);
                    // SAFETY, New Block is a valid Block
                    new_block.as_mut().alloc(size, layout).unwrap()
                }
            };

            let slot = slot.as_ptr() as *mut F;
            //SAFETY, garunteed to have space and align required for F.
            slot.write(fighter);
            &mut *slot
        }
        // }

    }
}


#[cfg(test)]
mod test {
    use std::{hint::black_box, time::{Duration, Instant}};

    
    use super::Corrida;
    
  
    pub static CORRIDA_ALLOC: fn(&Corrida, [i32; 16]) -> &mut [i32; 16] = Corrida::alloc;
    #[test]
    fn test_isolated_arena() {
        let arena = Corrida::new();
        {
            let a = arena.alloc(1);
            let b = arena.alloc(2);
            let c = arena.alloc(3);
            assert_eq!(*a, 1);
            assert_eq!(*b, 2);
            assert_eq!(*c, 3);
        }
    }

    // Well its slower than General Allocator...

    #[inline]
    #[test]
    fn test_large() {
        use std::time::*;
        // Each fighter is 4*16, 64 bytes
        let start = Instant::now();
        let arena = Corrida::new();
        for i in 0..5_000_000 {
            let _my_ref = CORRIDA_ALLOC(&arena, [i,i,i,i,i,i,i,i,i,i,i,i,i,i,i,i]);
            black_box(_my_ref);
        }
        println!("Arena: {}",start.elapsed().as_millis());
    }

    #[test]
    fn test_large2() {
        let start = Instant::now();
        let arena = Corrida::new();
        for i in 0..5_000_000 {
            let _my_ref = Box::new([i,i,i,i,i,i,i,i,i,i,i,i,i,i,i,i]);
            unsafe {
                let ptr = Box::into_raw(_my_ref);
                black_box(ptr);
            }
        }
        println!("Old Alloc {}", start.elapsed().as_millis());
    }

}