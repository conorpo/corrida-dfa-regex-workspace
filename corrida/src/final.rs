use std::{alloc::{AllocError, Allocator, Global, GlobalAlloc, Layout}, cell::{Cell, UnsafeCell}, intrinsics::size_of, marker::{PhantomData, PhantomPinned}, mem::{self, MaybeUninit}, ops::Index, ptr::NonNull, slice::{self, from_mut_ptr_range, from_raw_parts_mut}, sync::atomic::{AtomicI32, AtomicU32, Ordering}};

enum Assert<const CHECK: bool> {}
trait IsTrue {}
impl IsTrue for Assert<true> {}

static block_count: AtomicU32 = AtomicU32::new(0);

#[repr(align(256))]
struct Block
{   
    prev: Option<Box<Block>>,
    cur_ptr: NonNull<u8>
}

const BLOCK_DATA_SIZE: usize = 1 << 20;

impl Block
{
    const LAYOUT: Layout = unsafe { 
        Layout::from_size_align_unchecked(BLOCK_DATA_SIZE + size_of::<Self>(), 256)
    };
    fn new(prev: Option<Box<Block>>) -> NonNull<Self> {
        //SAFETY, we are about to immedieatly initialize prev, data will always be initalized
        println!("NEW_BLOCK");

        unsafe {
            let ptr = (Global {}).allocate(Self::LAYOUT).expect("Failed to allocate").as_mut_ptr();
            
            let footer = {
                NonNull::new_unchecked(ptr.add(BLOCK_DATA_SIZE) as *mut Self)
            };

            footer.write (Block {
                prev,
                cur_ptr: NonNull::new_unchecked(ptr)
            });

            footer
        }
    }

    fn alloc(&mut self, layout: Layout) -> Result<&mut [u8], AllocError> {
        unsafe {
            let align_offset = self.cur_ptr.align_offset(layout.align());
            let actual_size = layout.size() + layout.size() % layout.align();
            
            if self.cur_ptr.add(align_offset + actual_size).as_ptr() as *const u8 > (self as *const Block as *const u8) {
                Err(AllocError)
            } else {
                let slot_ptr = self.cur_ptr.add(align_offset);
                self.cur_ptr = slot_ptr.add(actual_size);
                Ok(from_raw_parts_mut(slot_ptr.as_ptr(), actual_size))
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
    cur_offset: Cell<usize>,
    _boo: PhantomData<Block>
}



impl Corrida
{
    pub fn new() -> Self {
        Self {
            cur_block: Cell::new(Block::new(None)),
            cur_offset: Cell::new(0),
            _boo: PhantomData
        }
    }

    fn alloc_block(&self) {
        
    }

    pub fn alloc<F>(&self, fighter: F) -> &mut F
    {
        //let mut cur_offset = self.cur_offset.get();
        let layout: Layout = Layout::new::<F>();
        
        unsafe {
            // SAFETY
            // self.cur_block will always have a valid pointer to a Block
            let mut slot = match (*self.cur_block.get().as_ptr()).alloc(layout) {
                Ok(slot) => {
                    slot
                },
                Err(_) => {
                    let old_block = Box::from_raw(self.cur_block.get().as_ptr());
                    let mut new_block = Block::new(Some(old_block));

                    let old = block_count.fetch_add(1, Ordering::Relaxed);
                    if old % 10 == 0 {
                        println!("{old} BLOCKS");
                    }

                    self.cur_block.set(new_block);
                    new_block.as_mut().alloc(layout).unwrap()
                }
            };

            let slot = slot.as_mut_ptr() as *mut F;
            slot.write(fighter);
            &mut *slot
        }

    }
}


#[cfg(test)]
mod test {
    use std::time::{Duration, Instant};

    use super::Corrida;

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
    #[test]
    fn test_large() {
        use std::time::*;
        // Each fighter is 4*16, 64 bytes
        let start = Instant::now();
        let arena = Corrida::new();
        for i in 0..10_000 {
            if i % 1000 == 0 {
                println!("{i}");
            }
            let _my_ref = arena.alloc([i,i,i,i,i,i,i,i,i,i,i,i,i,i,i,i]);
        }
        dbg!(start.elapsed());
        assert!(start.elapsed() < Duration::from_millis(500))
    }

    #[test]
    fn test_large2() {
        let start = Instant::now();
        let arena = Corrida::new();
        for i in 0..5_000_000 {
            let _my_ref = Box::new([i,i,i,i,i,i,i,i,i,i,i,i,i,i,i,i]);
            unsafe {
                Box::into_raw(_my_ref);
            }
        }
        dbg!(start.elapsed());
        assert!(start.elapsed() < Duration::from_millis(500))
    }

}