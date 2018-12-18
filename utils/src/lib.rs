#[macro_export]
macro_rules! getLine {
 ($($msg : expr)*) => {
    println!("[DEBUG] Execution hit line: {}", line!());
 }
}

/// strip surround quotes for the given string
pub fn strip(s: String) -> String {
    let mut t = s.clone();
    t.remove(0);
    t.remove(t.len()-1);
    t
}