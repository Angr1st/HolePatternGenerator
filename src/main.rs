use std::{fs::File, io::{BufReader, LineWriter, Write, BufRead}, path::Path, error::Error, fmt::Display};

use clap::{Parser, command};
use serde::{Serialize, Deserialize};

#[derive(Parser)]
#[command(author, version, about, long_about=None)]
struct Cli {
    config_path:String
}

#[derive(Serialize,Deserialize)]
struct Config {
    hole_distance:f64,
    hole_diameter:f64,
    plate_diameter:f64,
    distance_from_edge:f64,
    padding_distance_from_edge:f64,
    target_file_name:String,
    first_part_of_macro_file: String,
    second_part_of_macro_file: String
}

#[derive(PartialEq, Eq, Clone)]
enum HoleType {
    Center,
    Axes,
    Area
}

struct HolePosition {
    x:f64,
    z:f64,
    hole_type:HoleType
}

impl HolePosition {
    fn new(x:f64, z:f64) -> HolePosition {
        if x == 0.0 && z == 0.0 {
            HolePosition::create_center()
        }
        else if x == 0.0 || z == 0.0 {
            HolePosition { x, z, hole_type: HoleType::Axes }
        }
        else {
            HolePosition { x, z, hole_type: HoleType::Area }
        }
    }

    fn create_center() -> HolePosition {
        HolePosition { x: 0.0, z: 0.0, hole_type: HoleType::Center }
    }

    fn mirror(&self) -> Option<HolePosition> {
        if self.hole_type == HoleType::Center {
            None
        }
        else {
            Some(HolePosition { x: self.x * - 1.0, z: self.z * - 1.0, hole_type: self.hole_type.clone() })
        }
       
    }

    fn rotate(&self) -> Option<HolePosition> {
        if self.hole_type == HoleType::Center {
            None
        }
        else {
            Some(HolePosition { x: self.z, z: self.x, hole_type: self.hole_type.clone() })
        }
    }
}

impl Display for HolePosition {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "HolePosition({},{})", self.x, self.z)
    }
}

fn read_config_from_file<P: AsRef<Path>>(path: P) -> Result<Config, Box<dyn Error>> {
    // Open the file in read-only mode with buffer.
    let file = File::open(path)?;
    let reader = BufReader::new(file);

    // Read the JSON contents of the file as an instance of `Config`.
    let u = serde_json::from_reader(reader)?;

    // Return the `Config`.
    Ok(u)
}

fn insert_hole(i: i32, j: i32, hole_distance: f64, holes: &mut Vec<HolePosition>) {
    let holeposition = HolePosition::new(i as f64 * hole_distance ,j as f64 * hole_distance);
    let mirrored_opt = holeposition.mirror();
    let rotated_opt = holeposition.rotate();
    let mirrored_rotated_opt = holeposition.mirror().and_then(|m| m.rotate());
    holes.push(holeposition);
    if let Some(mirrored) = mirrored_opt {
        holes.push(mirrored);
    }
    if let Some(rotated) = rotated_opt {
        holes.push(rotated);
    }
    if let Some(mirrored_rotated) = mirrored_rotated_opt {
        holes.push(mirrored_rotated);
    }
}

fn create_line_writer(file_name:String) -> Result<LineWriter<File>, Box<dyn Error>> {
    let file = File::options().read(false).write(true).create(true).open(file_name)?;
    Ok(LineWriter::new(file))
}

fn load_file_content(file_name:String) -> Result<BufReader<File>,Box<dyn Error>> {
    let file = File::open(file_name)?;

    Ok(BufReader::new(file))
}

fn main() -> Result<(),Box<dyn Error>> {
    let cli = Cli::parse();

    let config = read_config_from_file(cli.config_path)?;
    
    let mut holes = Vec::new();

    //Calculate radius of plate 29
    let plate_radius = config.plate_diameter / 2.0;
    //and subtract the distance from the edege 28.15
    let padded_plate_radius = plate_radius - config.padding_distance_from_edge;
    //compute distance between 2 holes center points
    let hole_distance = config.hole_distance + config.hole_diameter;
    //compute the amount of holes 
    let hole_amount = padded_plate_radius / hole_distance;

    let r_hole_amount = hole_amount.floor() as i32;

    for i in 0..=r_hole_amount {
        //insert_hole(i,0, hole_distance, &mut holes);
        for j in 0..=r_hole_amount {
            insert_hole(i, j, hole_distance, &mut holes)
        }
    }

    let mut line_writer = create_line_writer(config.target_file_name)?;

    let first_part_file = load_file_content(config.first_part_of_macro_file)?;

    for line in first_part_file.lines() {
        if let Ok(l) = line {
            writeln!(line_writer,"{}",l)?;
        }
    }

    for hp in holes.iter().enumerate() {
        writeln!(line_writer,"holeList.append(({},{}))",hp.0, hp.1)?;
    }

    let second_part_file = load_file_content(config.second_part_of_macro_file)?;

    for line in second_part_file.lines() {
        if let Ok(l) = line {
            writeln!(line_writer,"{}",l)?;
        }
    }

    Ok(())
}