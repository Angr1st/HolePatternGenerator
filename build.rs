use std::env;

fn main() {  
    println!("From Build rs");
    // Re-runs script if any files in res are changed  
    println!("cargo:rerun-if-changed=res/*");  
    copy_to_output::copy_to_output("res", &env::var("PROFILE").unwrap()).expect("Could not copy");
    //copy_to_output::copy_to_output("res/secondPart.FCMacro", &env::var("PROFILE").unwrap()).expect("Could not copy");  
}