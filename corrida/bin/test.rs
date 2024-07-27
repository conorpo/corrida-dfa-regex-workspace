use corrida::r#final::*;

pub fn main() {
    use std::time::*;
    // Each fighter is 4*16, 64 bytes
    let start = Instant::now();
    let arena = Corrida::new();
    for i in 0..5_000_000 {
        let _my_ref = arena.alloc([i,i,i,i,i,i,i,i,i,i,i,i,i,i,i,i]);
    }
    dbg!(start.elapsed());
    //assert!(start.elapsed() < Duration::from_millis(500))
}