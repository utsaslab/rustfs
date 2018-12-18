#[macro_export]
macro_rules! getLine {
 ($($msg : expr)*) => {
    println!("[DEBUG] Execution hit line: {}", line!());
 }
}
