use std::io::BufRead;
use std::collections::HashMap;

quick_error! {
    #[derive(Debug)]
    pub enum GeometryError {
        ParseFailure { from(std::io::Error) from() }
        InvalidVertex
        InvalidTriangle
        InvalidColor
    }
}

#[derive(Debug)]
pub struct PuzzleData {
    vertices: Vec<(f32, f32)>, // x, y
    triangles: Vec<(u32, u32, u32, u32)>, // v0, v1, v2, color
    colors: Vec<(u8, u8, u8)>, // r, g, b
    edge_to_triangles: HashMap<(u32, u32), Vec<usize>>, // v0, v1 -> triangle indices (edge indices are sorted)
}

impl PuzzleData {
    pub fn from_reader<R: BufRead>(reader: &mut R) -> Result<PuzzleData, GeometryError> {
        let mut out = PuzzleData{
            vertices: vec![],
            triangles: vec![],
            colors: vec![],
            edge_to_triangles: HashMap::new(),
        };

        // Parse geometry and colors
        for line in reader.lines() {
            let l = line?;
            let split: Vec<&str> = l.split_whitespace().collect();
            match split.len() {
                2 => { // vertex
                    out.vertices.push((
                        split[0].parse::<f32>().map_err(|_| GeometryError::InvalidVertex)?,
                        split[1].parse::<f32>().map_err(|_| GeometryError::InvalidVertex)?
                    ));
                },
                3 => { // RGB color
                    out.colors.push((
                        split[0].parse::<u8>().map_err(|_| GeometryError::InvalidColor)?,
                        split[1].parse::<u8>().map_err(|_| GeometryError::InvalidColor)?,
                        split[2].parse::<u8>().map_err(|_| GeometryError::InvalidColor)?
                    ));
                },
                4 => { // triangle
                    let triangle_indices = split.iter().take(3).map(|s| {
                        let idx = s.parse::<u32>().map_err(|_| GeometryError::InvalidTriangle)?;
                        if idx as usize >= out.vertices.len() { Err(GeometryError::InvalidTriangle) } else { Ok(idx) }
                    }).collect::<Result<Vec<u32>, GeometryError>>()?;
                    
                    // Check for duplicate vertices
                    let mut triangle_index_integrity = triangle_indices.clone();
                    triangle_index_integrity.sort();
                    triangle_index_integrity.dedup();
                    if triangle_index_integrity.len() < 3 {
                        return Err(GeometryError::InvalidTriangle);
                    }

                    let color_idx = split[3].parse::<u32>().map_err(|_| GeometryError::InvalidTriangle)?;
                    if color_idx as usize > out.colors.len() { return Err(GeometryError::InvalidTriangle); }

                    out.triangles.push((
                        triangle_indices[0],
                        triangle_indices[1],
                        triangle_indices[2],
                        color_idx
                    ));
                },
                _ => return Err(GeometryError::ParseFailure)
            }
        }

        // Construct edge to triangle membership map
        for (idx, (v0, v1, v2, _)) in (&out.triangles).iter().enumerate() {
            let mut sorted = vec![v0, v1, v2];
            sorted.sort();
            for (&e0, &e1) in vec![(sorted[0], sorted[1]), (sorted[1], sorted[2]), (sorted[0], sorted[2])] {
                out.edge_to_triangles.entry((e0, e1)).or_insert(vec![]).push(idx);
            }
        }

        Ok(out)
    }

    pub fn num_triangles(&self) -> usize { self.triangles.len() }

    pub fn triangles_with_edge(&self, edge: &(u32, u32)) -> Option<&Vec<usize>> {
        self.edge_to_triangles.get(edge)
    }

    pub fn is_valid_edge(&self, edge: &(u32, u32)) -> bool {
        (edge.0 as usize) < self.vertices.len() && (edge.1 as usize) < self.vertices.len()
    }
}