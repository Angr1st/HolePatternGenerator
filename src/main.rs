use std::{fs::File, io::{BufReader, LineWriter, Write, BufRead}, path::Path, error::Error, fmt::Display};

use clap::{Parser, command, ValueEnum};
use serde::{Serialize, Deserialize};

#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, ValueEnum, Debug)]
enum Quadrants {
    One = 1,
    Two = 2,
    Three = 3,
    Four = 4
}

impl Quadrants {
    fn determine(x: f64, z: f64) -> Option<Quadrants> {
        // x 0 -> n && z 0 -> n
        if x >= 0.0 && z >= 0.0 {
            Some(Quadrants::One)
        }
        // x -0.1 -> -n & z 0.1 -> n
        else if x < 0.0 && z > 0.0 {
            Some(Quadrants::Two)
        }
        // x -0.1 -> -n & z -0.1 -> -n
        else if x <= 0.0 && z <= 0.0 {
            Some(Quadrants::Three)
        }
        // x 0.1 -> n & z -0.1 -> -n
        else if x > 0.0 && z < 0.0 {
            Some(Quadrants::Four)
        }
        else {
            None
        }
    }
}

#[cfg(test)]
mod quadrants_tests {
    use crate::Quadrants;

    #[test]
    fn first_quadrant() {
        let quadrant_opt = Quadrants::determine(1.0, 1.0);
        assert_eq!(quadrant_opt, Some(Quadrants::One))
    }

    #[test]
    fn first_quadrant_both_zero() {
        let quadrant_opt = Quadrants::determine(0.0, 0.0);
        assert_eq!(quadrant_opt, Some(Quadrants::One))
    }

    #[test]
    fn first_quadrant_one_zero() {
        let quadrant_opt = Quadrants::determine(1.0, 0.0);
        assert_eq!(quadrant_opt, Some(Quadrants::One))
    }

    #[test]
    fn second_quadrant() {
        let quadrant_opt = Quadrants::determine(-1.0, 1.0);
        assert_eq!(quadrant_opt, Some(Quadrants::Two))
    }

    #[test]
    fn third_quadrant() {
        let quadrant_opt = Quadrants::determine(-1.0, -1.0);
        assert_eq!(quadrant_opt, Some(Quadrants::Three))
    }

    #[test]
    fn third_quadrant_one_minus() {
        let quadrant_opt = Quadrants::determine(-1.0, 0.0);
        assert_eq!(quadrant_opt, Some(Quadrants::Three))
    }

    #[test]
    fn fourth_quadrant() {
        let quadrant_opt = Quadrants::determine(1.0, -1.0);
        assert_eq!(quadrant_opt, Some(Quadrants::Four))
    }
}

#[derive(Parser)]
#[command(author, version, about, long_about=None)]
struct Cli {
    #[arg(short)]
    config_path:String,
    /// How many quadrants to compute
    #[arg(value_enum,short)]
    quadrants:Quadrants
}

#[derive(Serialize,Deserialize)]
struct Config {
    hole_distance:f64,
    hole_diameter:f64,
    plate_diameter:f64,
    distance_from_edge:f64,
    target_file_name:String,
    first_part_of_macro_file: String,
    second_part_of_macro_file: String
}

struct HoleDefinition {
    total_depth:f64, 
    inner_diameter:f64, 
    straight_depth:f64, 
    hole_angle:f64
}

impl Display for HoleDefinition {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "basicHoleDefinition = HoleDefinition({}, {}, {}, {})", self.total_depth, self.inner_diameter, self.straight_depth, self.hole_angle)
    }
}

#[derive(PartialEq, Eq, Clone)]
enum HoleType {
    Center,
    Axes(Quadrants),
    Area(Quadrants)
}

impl Display for HoleType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Center => write!(f, "CENTER"),
            Self::Area(quadrant) =>
                match quadrant {
                    Quadrants::One => write!(f,"RIGHTTOP"),
                    Quadrants::Two => write!(f,"RIGHTBOTTOM"),
                    Quadrants::Three => write!(f,"LEFTBOTTOM"),
                    Quadrants::Four => write!(f, "LEFTTOP")
                },
            Self::Axes(quadrant) =>
                match quadrant {
                    Quadrants::One => write!(f,"TOP"),
                    Quadrants::Two => write!(f,"RIGHT"),
                    Quadrants::Three => write!(f,"BOTTOM"),
                    Quadrants::Four => write!(f, "LEFT")
                }
        }
    }
}

#[derive(PartialEq, Eq, Clone)]
enum HoleReplicationMethod {
    Mirror,
    MirrorX,
    MirrorXRotate,
    Rotate
}

#[derive(Clone,Debug)]
struct DistanceToEdge {
    iteration:i32,
    hole_distance:f64,
    distance_to_edge:f64
}

impl DistanceToEdge {
    fn new(iteration:i32, hole_distance:f64, distance_to_edge:f64) -> DistanceToEdge {
        DistanceToEdge { iteration, hole_distance, distance_to_edge }
    }
}

#[derive(Clone)]
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
        else {
            let quadrant = Quadrants::determine(x, z).expect("X and Z should not be both zero!");
            if x == 0.0 || z == 0.0 {
                HolePosition { x, z, hole_type: HoleType::Axes(quadrant) }
            }
            else {     
                HolePosition { x, z, hole_type: HoleType::Area(quadrant) }
            }
        }
    }

    fn get_hole_quadrant(&self) -> Option<Quadrants> {
        return match self.hole_type {
            HoleType::Center => None,
            HoleType::Area(q) => Some(q),
            HoleType::Axes(q) => Some(q)
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
            Some(HolePosition::new( self.x * - 1.0, self.z * - 1.0))
        }
    }

    fn mirror_x(&self) -> Option<HolePosition> {
        if self.hole_type == HoleType::Center {
            None
        }
        else {
            Some(HolePosition::new(self.x * - 1.0, self.z))
        }
    }

    fn mirror_x_rotate(&self) -> Option<HolePosition> {
        if self.hole_type == HoleType::Center {
            None
        }
        else {
            self.mirror_x().and_then(|hp| hp.rotate())
        }
    }

    fn rotate(&self) -> Option<HolePosition> {
        if self.hole_type == HoleType::Center {
            None
        }
        else {
            Some(HolePosition::new( self.z, self.x))
        }
    }
}

impl Display for HolePosition {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "HolePosition({},{},HoleType.{})", self.x, self.z, self.hole_type)
    }
}

#[cfg(test)]
mod hole_position_tests {
    use crate::HolePosition;

    #[test]
    fn print_center_hole() {
        let hole = HolePosition::new(0.0, 0.0);
        let string_repsentation = format!("{}",hole);
        assert_eq!(string_repsentation, "HolePosition(0,0,HoleType.CENTER)");
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

fn check_quadrant(hole_position: HolePosition, quadrant_setting:Quadrants) -> Option<HolePosition> {
    if let Some(quadrant) = hole_position.get_hole_quadrant() {
        if quadrant <= quadrant_setting {
            return Some(hole_position);
        }
    }
    
    return None;
}

fn quadrants_check(hole_position: &HolePosition, quadrant_setting:Quadrants, hole_replication_method: HoleReplicationMethod) -> Option<HolePosition> {
    let new_hole_opt = match hole_replication_method {
        HoleReplicationMethod::Mirror => hole_position.mirror(),
        HoleReplicationMethod::Rotate => hole_position.rotate(),
        HoleReplicationMethod::MirrorX => hole_position.mirror_x(),
        HoleReplicationMethod::MirrorXRotate => hole_position.mirror_x_rotate() 
    };

    if let Some(quadrant) = new_hole_opt.as_ref().and_then(|hp| hp.get_hole_quadrant()) {
        if quadrant <= quadrant_setting {
            return new_hole_opt;
        }
    }
    
    return None;
}

fn insert_hole(distance: &DistanceToEdge, hole_distance: f64, distance_from_edge: f64, holes: &mut Vec<HolePosition>, distances : &Vec<DistanceToEdge>, quadrants:Quadrants) {
    if distance.iteration == 0 {
        holes.push(HolePosition::create_center());
        return;
    }
    
    let padded_distance_to_edge = distance.distance_to_edge - distance_from_edge;

    for j in 0..=distance.iteration {
        let z = j as f64 * hole_distance;
        //x is the hole position 
        //distance from edge is the minimum distance from the edge for a hole central point
        //distance_to_edge is the distance to the edge of the cirle at this x height
        if z  > padded_distance_to_edge {
            return;
        }

        compute_insert_holes(distance.hole_distance, z, distance.iteration, j, quadrants, holes);

        if j != 0 && j != distance.iteration {
            compute_insert_holes(distance.hole_distance, -z, distance.iteration, j, quadrants, holes);
        }
    }
}

fn round_two_places(number: f64) -> f64 {
    (number * 100.0).round() / 100.0
}

fn compute_insert_holes(x: f64, z: f64, i: i32, j: i32, quadrants: Quadrants, holes: &mut Vec<HolePosition>) {
    let x_rounded = round_two_places(x);
    let z_rounded = round_two_places(z);
    let hole_position = HolePosition::new(x_rounded,z_rounded);
    let mut mirrored_opt = None;
    let mut rotated_opt = None;
    if i == j {           
        mirrored_opt = quadrants_check(&hole_position, quadrants, HoleReplicationMethod::Mirror);
    }
    else {
        rotated_opt = quadrants_check(&hole_position, quadrants, HoleReplicationMethod::Rotate);
    }
    let mirrored_x_opt = quadrants_check(&hole_position, quadrants, HoleReplicationMethod::MirrorX);
    let mirrored_x_rotated_opt = quadrants_check(&hole_position, quadrants, HoleReplicationMethod::MirrorXRotate);
    if let Some(some_hole_position) = check_quadrant(hole_position, quadrants) {
        holes.push(some_hole_position);
    }
    if let Some(mirrored) = mirrored_opt {
        holes.push(mirrored);
    }
    if let Some(rotated) = rotated_opt {
        holes.push(rotated);
    }
    if let Some(mirrored_x) = mirrored_x_opt {
        holes.push(mirrored_x);
    }
    if let Some(mirrored_x_rotated) = mirrored_x_rotated_opt {
        holes.push(mirrored_x_rotated);
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

fn compute_distance_to_edge(plate_radius: f64, hole_distance: f64, hole_amount: i32) -> Vec<DistanceToEdge> {
    let mut collection = Vec::new();
    
    for i in 0..=hole_amount {
        let x = i as f64 * hole_distance;
        //calculate the distance to the edge of the circle at this x height
        //arccos(x/r) = radiants (Angle) -> radius * sin(angle)
        let angle_from_center = (x / plate_radius).acos();
        let distance_to_edge = (angle_from_center).sin() * plate_radius;

        println!("Angle from Center in radiants: {}; Distance to edge at x: {} is {}", angle_from_center, x, distance_to_edge);

        collection.push(DistanceToEdge::new(i,x,distance_to_edge));
    }

    return collection;
}

fn main() -> Result<(),Box<dyn Error>> {
    let cli = Cli::parse();

    let config = read_config_from_file(cli.config_path)?;
    
    let mut holes = Vec::new();

    //Calculate radius of plate 29
    let plate_radius = config.plate_diameter / 2.0;
    //Calculate hole radius
    let hole_radius:f64 = config.hole_diameter / 2.0;
    //Calculate the distance from edge plus one hole radius
    let distance_from_edge_plus_hole_radius = config.distance_from_edge + hole_radius;
    //and subtract the distance from the edege 28.15
    let padded_plate_radius = plate_radius - config.distance_from_edge;
    //compute distance between 2 holes center points
    let hole_distance = config.hole_distance + config.hole_diameter;
    //compute the amount of holes 
    let max_hole_amount = padded_plate_radius / hole_distance;

    let r_max_hole_amount = max_hole_amount.floor() as i32;

    let distances_to_edge = compute_distance_to_edge(plate_radius, hole_distance, r_max_hole_amount);

    for distance in distances_to_edge {
        insert_hole(distance.iteration, distance.hole_distance, hole_distance, distance_from_edge_plus_hole_radius,distance.distance_to_edge, &mut holes, cli.quadrants);
    }

    let mut line_writer = create_line_writer(config.target_file_name)?;

    let first_part_file = load_file_content(config.first_part_of_macro_file)?;

    let basic_hole_definition = HoleDefinition {
        total_depth : 1.0,
        inner_diameter : 0.3,
        straight_depth : 0.75,
        hole_angle : 25.0
    };

    for line in first_part_file.lines() {
        if let Ok(l) = line {
            writeln!(line_writer,"{}",l)?;
        }
    }

    writeln!(line_writer, "{}", basic_hole_definition)?;

    let mut debug_line_writer = create_line_writer(String::from("cords.csv"))?;

    for hp in holes.iter() {
        writeln!(debug_line_writer,"{},{}",hp.x, hp.z)?;
        writeln!(line_writer,"holeList.append({})",hp)?;
    }

    let second_part_file = load_file_content(config.second_part_of_macro_file)?;

    for line in second_part_file.lines() {
        if let Ok(l) = line {
            writeln!(line_writer,"{}",l)?;
        }
    }

    Ok(())
}