// #![feature(allocate_api)]
// //! An Arena implemenation using Rust's Allocator trait, allowing it to be as an allocator for other structs.
// use core::alloc::*;
// struct ArenaAllocator {

// }

// unsafe impl Allocator for ArenaAllocator {
//     fn allocate(&self, layout: Layout) -> Result<NonNull<[u8]>, AllocError> {
//         todo!();
//     }

//     unsafe fn deallocate(&self, ptr: NonNull<u8>, layout: Layout) {
//         todo!();
//     }
// }