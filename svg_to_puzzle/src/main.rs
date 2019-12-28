use std::env;
use quick_xml::Reader;
use quick_xml::events::Event;

type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;

fn main() -> Result<()> {
    let filename = env::args().nth(1)
        .ok_or::<Box<dyn std::error::Error>>(From::from("2nd arg (filename) required"))?;

    // Set up variables where we keep track of parsed geometry
    let mut dims: Option<(f32, f32)> = None;

    // Parse XML file
    let mut reader = Reader::from_file(filename)?;
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
                        for attribute in e.attributes() {
                            let a = attribute?;
                            match a.key {
                                b"fill" => {
                                    // TODO - parse color
                                },
                                b"points" => {
                                    // TODO - parse points
                                },
                                _ => (),
                            }
                        }
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

    println!("Dimensions: {:?}", dims);
    Ok(())
}