use std::{fs::File, io::BufReader, path::Path, error::Error};

use clap::{Parser, command};
use serde::{Serialize, Deserialize};

#[derive(Parser)]
#[command(author, version, about, long_about=None)]
struct Cli {
    config_path:String
}

#[derive(Serialize,Deserialize)]
struct Config {
    hole_distance:f64
}

fn read_config_from_file<P: AsRef<Path>>(path: P) -> Result<Config, Box<dyn Error>> {
    // Open the file in read-only mode with buffer.
    let file = File::open(path)?;
    let reader = BufReader::new(file);

    // Read the JSON contents of the file as an instance of `User`.
    let u = serde_json::from_reader(reader)?;

    // Return the `User`.
    Ok(u)
}

fn main() -> Result<(),Box<dyn Error>> {
    let cli = Cli::parse();

    println!("{}",cli.config_path);

    let config = read_config_from_file(cli.config_path)?;
    
    println!("{}",config.hole_distance);
    Ok(())
}
