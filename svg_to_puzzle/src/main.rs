use std::path::PathBuf;
use std::collections::HashMap;
use quick_xml::Reader;
use quick_xml::events::Event;
use structopt::StructOpt;

type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;

#[derive(Debug, StructOpt)]
struct Cli {
    #[structopt(short = "f", parse(from_os_str))]
    file: PathBuf,
    #[structopt(short = "w", default_value = "5.0")]
    width: f32,
    #[structopt(short = "h", default_value = "5.0")]
    height: f32,
}

#[derive(Copy, Clone, Debug)]
struct Vertex {
    x: f32,
    y: f32
}

impl PartialEq for Vertex {
    fn eq(&self, other: &Self) -> bool {
        (self.x - other.x).abs() < 1e-5 && (self.y - other.y).abs() < 1e-5
    }
}

impl Eq for Vertex {}

impl std::hash::Hash for Vertex {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        format!("{:.5}", self.x).hash(state);
        format!("{:.5}", self.y).hash(state);
    }
}

fn main() -> Result<()> {
    let opt = Cli::from_args();
    let (width, height) = (opt.width, opt.height);

    // Set up variables where we keep track of parsed geometry
    let mut dims: Option<(f32, f32)> = None;
    let mut colors: HashMap<(u8, u8, u8), u32> = HashMap::new();
    let mut vertices: HashMap<Vertex, u32> = HashMap::new();
    let mut triangles: Vec<[u32; 4]> = vec![];

    // Parse XML file
    let mut reader = Reader::from_file(opt.file)?;
    reader.trim_text(true);
    let mut buf = Vec::new();
    loop {
        match reader.read_event(&mut buf) {
            Ok(Event::Start(ref e)) => {
                match e.name() {
                    b"svg" => {
                        let (mut width, mut height) = (0., 0.);
                        for attribute in e.attributes() {
                            let a = attribute?;
                            match a.key {
                                b"width" => {
                                    width = reader.decode(a.unescaped_value()?.to_mut())?
                                        .trim_end_matches("px").parse::<f32>()?;
                                },
                                b"height" => {
                                    height = reader.decode(a.unescaped_value()?.to_mut())?
                                        .trim_end_matches("px").parse::<f32>()?;
                                },
                                _ => (),
                            }
                        }
                        dims = Some((width, height));
                    },
                    b"polygon" => {
                        let mut points: Vec<Vertex> = vec![];
                        let mut color: Option<(u8, u8, u8)> = None;
                        for attribute in e.attributes() {
                            let a = attribute?;
                            match a.key {
                                b"fill" => {
                                    let mut unescaped = a.unescaped_value()?;
                                    let raw = reader.decode(unescaped.to_mut())?.trim_start_matches("#");
                                    let parsed_from_hex = i32::from_str_radix(raw, 16)?;
                                    color = Some((
                                        ((parsed_from_hex >> 16) & 0xff) as u8,
                                        ((parsed_from_hex >> 8) & 0xff) as u8,
                                        (parsed_from_hex & 0xff) as u8,
                                    ));
                                },
                                b"points" => {
                                    let mut unescaped = a.unescaped_value()?;
                                    let split = reader.decode(unescaped.to_mut())?
                                        .split_whitespace().collect::<Vec<&str>>();
                                    if split.len() != 6 { return Err(From::from("Polygon was not a triangle")); }
                                    points = split.chunks(2).map(|c| {
                                        Ok(Vertex{ x: c[0].parse::<f32>()?, y: c[1].parse::<f32>()? })
                                    }).collect::<Result<Vec<Vertex>>>()?;
                                },
                                _ => (),
                            }
                        }

                        // Use existing color if we can find one, or add new color and use that one's index
                        let parsed_color = color.ok_or::<Box<dyn std::error::Error>>(From::from("No color parsed"))?;
                        let color_idx = if let Some(idx) = colors.get(&parsed_color) {
                            *idx
                        } else {
                            let idx = colors.len() as u32;
                            colors.insert(parsed_color, idx);
                            idx
                        };

                        // Use existing vertices if we can find them, or add new vertices and use their indices
                        let indices = points.iter().map(|p| {
                            if let Some(idx) = vertices.get(p) {
                                *idx
                            } else {
                                let idx = vertices.len() as u32;
                                vertices.insert(p.clone(), idx);
                                idx
                            }
                        }).collect::<Vec<u32>>();

                        triangles.push([indices[0], indices[1], indices[2], color_idx]);
                    },
                    _ => (),
                }
            },
            Ok(Event::Eof) => break,
            Err(e) => return Err(Box::new(e)),
            _ => (),
        }
    
        buf.clear();
    }

    let dims = dims.ok_or::<Box<dyn std::error::Error>>(From::from("Did not find dimensions"))?;

    // Turn string vertex hashmap into vec of float vertices ordered by index
    let mut float_vertices = vertices.into_iter().collect::<Vec<(Vertex, u32)>>();
    float_vertices.sort_by(|(_, idx1), (_, idx2)| idx1.cmp(idx2));
    let mut float_vertices = float_vertices.into_iter().map(|(v, _)| v).collect::<Vec<Vertex>>();

    // Turn color hashmap into vec of colors ordered by index
    let mut color_vec = colors.iter().map(|(&k, &v)| (k, v)).collect::<Vec<((u8, u8, u8), u32)>>();
    color_vec.sort_by(|(_, idx1), (_, idx2)| idx1.cmp(idx2));
    let color_vec = color_vec.iter().map(|&(color, _)| color).collect::<Vec<(u8, u8, u8)>>();

    // Order triangle indices in a counterclockwise direction
    triangles.iter_mut().for_each(|t| {
        // Get centroid so we can compare vertices to it by angle
        let mut centroid = t[0..3].iter().fold((0., 0.), |acc, idx| {
            let vertex = float_vertices[*idx as usize];
            (acc.0 + vertex.x, acc.1 + vertex.y)
        });
        centroid = (centroid.0 / 3.0, centroid.1 / 3.0);

        // Sort by relative counterclockwise angle to the centroid
        (&mut t[0..3]).sort_by(|idx1, idx2| {
            let v1 = float_vertices[*idx1 as usize];
            let mut v1_angle = (v1.y - centroid.1).atan2(v1.x - centroid.0).to_degrees() + 360.0;
            if v1_angle > 360.0 { v1_angle -= 360.0; }

            let v2 = float_vertices[*idx2 as usize];
            let mut v2_angle = (v2.y - centroid.1).atan2(v2.x - centroid.0).to_degrees() + 360.0;
            if v2_angle > 360.0 { v2_angle -= 360.0; }

            v1_angle.partial_cmp(&v2_angle).unwrap()
        });
    });

    // Scale float vertices down to the specified width/height range
    float_vertices.iter_mut().for_each(|v| {
        *v = Vertex {
            x: -(width / 2.0) + width * v.x / dims.0,
            y: -(height / 2.0) + height * (dims.1 - v.y) / dims.1,
        };
    });

    for vertex in float_vertices {
        println!("{} {}", vertex.x, vertex.y);
    }

    for color in color_vec {
        println!("{} {} {}", color.0, color.1, color.2);
    }

    for triangle in triangles {
        println!("{} {} {} {}", triangle[0], triangle[1], triangle[2], triangle[3]);
    }

    Ok(())
}