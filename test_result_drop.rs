struct Guard<'a> {
    depth: &'a mut usize,
}

impl<'a> Drop for Guard<'a> {
    fn drop(&mut self) {
        println!("Guard dropped!");
        *self.depth -= 1;
    }
}

fn create_guard(depth: &mut usize) -> Result<Guard, String> {
    *depth += 1;
    Ok(Guard { depth })
}

fn main() {
    let mut depth = 0;
    
    let g = create_guard(&mut depth).expect("should work");
    println!("After expect: depth = {}", depth);
    drop(g);
    println!("After dropping guard: depth = {}", depth);
}
