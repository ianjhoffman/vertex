use super::{error::*, shader::*};
use super::super::geometry::{StaticGraphicsData, DynamicGraphicsData};
use std::rc::Rc;
use std::collections::HashMap;
use wasm_bindgen::JsCast;
use web_sys::WebGlRenderingContext as GL;
use js_sys::WebAssembly;

static TRIANGLE_VS: &'static str = include_str!("./shaders/triangle-vertex.glsl");
static TRIANGLE_FS: &'static str = include_str!("./shaders/triangle-fragment.glsl");
static LINE_VS: &'static str = include_str!("./shaders/line-vertex.glsl");
static LINE_FS: &'static str = include_str!("./shaders/line-fragment.glsl");
static POINT_VS: &'static str = include_str!("./shaders/point-vertex.glsl");
static POINT_FS: &'static str = include_str!("./shaders/point-fragment.glsl");

pub struct Graphics {
    context: Rc<GL>,
    shaders: HashMap<ShaderKind, Shader>,
}

impl Graphics {
    pub fn from_canvas(canvas: &web_sys::HtmlCanvasElement) -> Result<Graphics, GraphicsError> {
        let context = canvas.get_context("webgl")
            .map_err(|_| GraphicsError::ContextFailed)?
            .ok_or(GraphicsError::ContextFailed)?
            .dyn_into::<GL>().map_err(|_| GraphicsError::ContextFailed)?;

        let mut ret = Graphics{
            context: Rc::new(context),
            shaders: HashMap::new(),
        };

        ret.shaders.insert(ShaderKind::Triangles, Shader::new(&ret.context, TRIANGLE_VS, TRIANGLE_FS)?);
        ret.shaders.insert(ShaderKind::Lines, Shader::new(&ret.context, LINE_VS, LINE_FS)?);
        ret.shaders.insert(ShaderKind::Points, Shader::new(&ret.context, POINT_VS, POINT_FS)?);
        Ok(ret)
    }

    pub fn draw(&self, static_data: &StaticGraphicsData, dynamic_data: &DynamicGraphicsData) {
        self.context.clear_color(0.5, 0.5, 0.5, 1.0);
        self.context.clear(GL::COLOR_BUFFER_BIT | GL::DEPTH_BUFFER_BIT);
        self.context.viewport(0, 0, 200, 200);

        self.draw_triangles(
            &static_data.triangle_position_vertices,
            &static_data.triangle_color_idx_vertices,
            &dynamic_data.triangle_indices,
            &static_data.colors_uniform,
        );

        self.draw_lines(&dynamic_data.line_vertices);

        self.draw_points(
            &static_data.point_position_vertices,
            &static_data.point_idx_vertices,
        );
    }

    fn draw_triangles(
        &self,
        vertex_positions: &Vec<f32>,
        vertex_colors: &Vec<f32>,
        indices: &Vec<u16>,
        colors: &Vec<f32>,
    ) {
        if indices.len() == 0 { return }

        let shader = self.shaders.get(&ShaderKind::Triangles).unwrap();
        self.context.use_program(Some(&shader.program));

        // Set up and buffer position/color index attributes
        let pos_attrib = self.context.get_attrib_location(&shader.program, "position") as u32;
        let color_attrib = self.context.get_attrib_location(&shader.program, "color") as u32;
        self.buffer_f32_data(vertex_positions, pos_attrib, 2);
        self.buffer_f32_data(vertex_colors, color_attrib, 1);
        self.buffer_u16_indices(indices);

        // Set color uniform
        let colors_uniform = shader.get_uniform_location(&self.context, "colors");
        self.context.uniform3fv_with_f32_array(colors_uniform.as_ref(), colors);

        // Draw triangles
        self.context.draw_elements_with_i32(GL::TRIANGLES, indices.len() as i32, GL::UNSIGNED_SHORT, 0);
    }

    fn draw_lines(&self, vertices: &Vec<f32>) {
        if vertices.len() == 0 { return }

        let shader = self.shaders.get(&ShaderKind::Lines).unwrap();
        self.context.use_program(Some(&shader.program));

        // Set up and buffer position attribute
        let pos_attrib = self.context.get_attrib_location(&shader.program, "position") as u32;
        self.buffer_f32_data(vertices, pos_attrib, 2);

        // Draw disconnected lines
        self.context.draw_arrays(GL::LINES, 0, (vertices.len() >> 1) as i32);
    }

    fn draw_points(&self, vertex_positions: &Vec<f32>, vertex_indices: &Vec<f32>) {
        if vertex_indices.len() == 0 { return }

        let shader = self.shaders.get(&ShaderKind::Points).unwrap();
        self.context.use_program(Some(&shader.program));

        // Set up and buffer position/index attributes
        let pos_attrib = self.context.get_attrib_location(&shader.program, "position") as u32;
        let idx_attrib = self.context.get_attrib_location(&shader.program, "index") as u32;
        self.buffer_f32_data(vertex_positions, pos_attrib, 2);
        self.buffer_f32_data(vertex_indices, idx_attrib, 1);

        // Later - we'll need to send a uniform for the selected vertex index

        // Draw points
        self.context.draw_arrays(GL::POINTS, 0, vertex_indices.len() as i32);
    }

    fn buffer_f32_data(&self, data: &[f32], attrib: u32, size: i32) {
        let memory_buffer = wasm_bindgen::memory()
            .dyn_into::<WebAssembly::Memory>()
            .unwrap()
            .buffer();

        let data_location = data.as_ptr() as u32 / 4;

        let data_array = js_sys::Float32Array::new(&memory_buffer)
            .subarray(data_location, data_location + data.len() as u32);

        let buffer = self.context.create_buffer().unwrap();

        self.context.bind_buffer(GL::ARRAY_BUFFER, Some(&buffer));
        self.context.buffer_data_with_array_buffer_view(GL::ARRAY_BUFFER, &data_array, GL::STATIC_DRAW);
        self.context.vertex_attrib_pointer_with_i32(attrib, size, GL::FLOAT, false, 0, 0);
        self.context.enable_vertex_attrib_array(attrib);
    }

    fn buffer_u16_indices(&self, indices: &[u16]) {
        let memory_buffer = wasm_bindgen::memory()
            .dyn_into::<WebAssembly::Memory>()
            .unwrap()
            .buffer();

        let indices_location = indices.as_ptr() as u32 / 2;
        let indices_array = js_sys::Uint16Array::new(&memory_buffer)
            .subarray(indices_location, indices_location + indices.len() as u32);

        let index_buffer = self.context.create_buffer().unwrap();
        self.context.bind_buffer(GL::ELEMENT_ARRAY_BUFFER, Some(&index_buffer));
        self.context.buffer_data_with_array_buffer_view(
            GL::ELEMENT_ARRAY_BUFFER,
            &indices_array,
            GL::STATIC_DRAW,
        );
    }
}