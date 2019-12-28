use std::io::BufRead;
use std::collections::HashMap;
use super::puzzle_state::PuzzleState;

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
    triangles: Vec<[u32; 4]>, // v0, v1, v2, color
    colors: Vec<[f32; 3]>, // r, g, b (0-1 float)
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
                    out.colors.push([
                        split[0].parse::<u8>().map_err(|_| GeometryError::InvalidColor)? as f32 / 255.0,
                        split[1].parse::<u8>().map_err(|_| GeometryError::InvalidColor)? as f32 / 255.0,
                        split[2].parse::<u8>().map_err(|_| GeometryError::InvalidColor)? as f32 / 255.0
                    ]);
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

                    out.triangles.push([
                        triangle_indices[0],
                        triangle_indices[1],
                        triangle_indices[2],
                        color_idx
                    ]);
                },
                _ => return Err(GeometryError::ParseFailure)
            }
        }

        // Construct edge to triangle membership map
        for (idx, triangle_data) in (&out.triangles).iter().enumerate() {
            let mut sorted = triangle_data[0..3].to_vec();
            sorted.sort();
            for (e0, e1) in vec![(sorted[0], sorted[1]), (sorted[1], sorted[2]), (sorted[0], sorted[2])] {
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

    pub fn get_static_graphics_data(&self) -> StaticGraphicsData {
        StaticGraphicsData::from_data(self)
    }

    pub fn get_dynamic_graphics_data(&self, state: &PuzzleState) -> DynamicGraphicsData {
        DynamicGraphicsData::from_data_and_state(self, state)
    }
}

// Should only need to ever make one of these per puzzle
#[derive(Debug)]
pub struct StaticGraphicsData {
    pub num_vertices: usize,
    pub triangle_position_vertices: Vec<f32>,
    pub triangle_color_idx_vertices: Vec<f32>,
    pub point_position_vertices: Vec<f32>,
    pub point_idx_vertices: Vec<f32>,
    pub colors_uniform: Vec<f32>,
}

impl StaticGraphicsData {
    fn from_data(data: &PuzzleData) -> StaticGraphicsData {
        let mut out = StaticGraphicsData {
            num_vertices: data.vertices.len(),
            triangle_position_vertices: vec![],
            triangle_color_idx_vertices: vec![],
            point_position_vertices: vec![],
            point_idx_vertices: vec![],
            colors_uniform: vec![],
        };

        for triangle in &data.triangles {
            // We need to make multiple copies of vertices for each triangle that uses them
            // The second attribute of a triangle vertex is the color index in the color array uniform
            let color_idx = triangle[3];
            for &vert_idx in &triangle[0..3] {
                let (x, y) = &data.vertices[vert_idx as usize];
                out.triangle_position_vertices.append(&mut vec![*x, *y]);
                out.triangle_color_idx_vertices.push(color_idx as f32);
            }
        }

        for (idx, &(x, y)) in (&data.vertices).iter().enumerate() {
            // The second attribute of a line/point vertex is which vertex it is (for highlighting)
            out.point_position_vertices.append(&mut vec![x, y]);
            out.point_idx_vertices.push(idx as f32);
        }

        for color in &data.colors {
            out.colors_uniform.append(&mut color.to_vec());
        }

        out
    }
}

// Need to make one one of these for every frame
#[derive(Debug)]
pub struct DynamicGraphicsData {
    pub triangle_indices: Vec<u16>,
    pub line_vertices: Vec<f32>,
}

impl DynamicGraphicsData {
    fn from_data_and_state(data: &PuzzleData, state: &PuzzleState) -> DynamicGraphicsData {
        let mut out = DynamicGraphicsData {
            triangle_indices: vec![],
            line_vertices: vec![],
        };

        for &(start, end) in state.get_connected_edges() {
            let ((start_x, start_y), (end_x, end_y)) = (data.vertices[start as usize], data.vertices[end as usize]);
            out.line_vertices.append(&mut vec![start_x, start_y, end_x, end_y]);
        }

        for &idx in state.get_unlocked_triangles() {
            let base = idx as u16 * 3;
            out.triangle_indices.append(&mut vec![base, base + 1, base + 2]);
        }

        out
    }
}