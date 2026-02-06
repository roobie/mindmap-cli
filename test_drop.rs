struct Guard<'a> {
    depth: &'a mut usize,
}

impl<'a> Drop for Guard<'a> {
    fn drop(&mut self) {
        println!("Guard dropped! Decrementing {} -> {}", *self.depth, *self.depth - 1);
        *self.depth = self.depth.saturating_sub(1);
    }
}

fn main() {
    let mut depth = 0;
    
    {
        *&mut depth = 1;
        let g = Guard { depth: &mut depth };
        println!("After creating guard: depth = {}", depth);
        drop(g);
        println!("After dropping guard: depth = {}", depth);
    }
}
