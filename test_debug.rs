fn main() {
    let path = std::path::Path::new("/etc/passwd");
    println!("is_absolute: {}", path.is_absolute());
    
    for component in path.components() {
        println!("component: {:?}", component);
    }
}
