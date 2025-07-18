use std::process::{Command, Stdio};
use std::io::Write;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("Testing Trust AI local model integration...");
    
    // Start Trust AI process
    let mut child = Command::new("./target/release/forge")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()?;
    
    // Send model list command
    if let Some(stdin) = child.stdin.as_mut() {
        stdin.write_all(b"/model list\n")?;
        stdin.write_all(b"/exit\n")?;
    }
    
    // Wait for the process to complete
    let output = child.wait_with_output()?;
    
    println!("Exit status: {}", output.status);
    println!("Stdout:\n{}", String::from_utf8_lossy(&output.stdout));
    println!("Stderr:\n{}", String::from_utf8_lossy(&output.stderr));
    
    Ok(())
}