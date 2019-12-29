use super::{error::*, shader::*};
use super::super::geometry::{StaticGraphicsData, DynamicGraphicsData};
use std::rc::Rc;
use std::collections::{HashMap, HashSet};
use wasm_bindgen::JsCast;
use web_sys::WebGlRenderingContext as GL;
use js_sys::WebAssembly;
use nalgebra_glm::{TVec4, TMat4};

static TRIANGLE_VS: &'static str = include_str!("./shaders/triangle-vertex.glsl");
static TRIANGLE_FS: &'static str = include_str!("./shaders/triangle-fragment.glsl");
static LINE_VS: &'static str = include_str!("./shaders/line-vertex.glsl");
static LINE_FS: &'static str = include_str!("./shaders/line-fragment.glsl");
static POINT_VS: &'static str = include_str!("./shaders/point-vertex.glsl");
static POINT_FS: &'static str = include_str!("./shaders/point-fragment.glsl");

const DEFAULT_CLEAR_COLOR: [f32; 4] = [0.8, 0.8, 0.8, 1.0];

pub struct Graphics {
    context: Rc<GL>,
    shaders: HashMap<ShaderKind, Shader>,
    window_size: (u32, u32),
    clear_color: [f32; 4],
    view_matrix: TMat4<f32>,
    viewport: TVec4<f32>,
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
            window_size: (canvas.width(), canvas.height()),
            clear_color: DEFAULT_CLEAR_COLOR,
            view_matrix: nalgebra_glm::ortho(-3.0, 3.0, -3.0, 3.0, 0.1, 1000.0),
            viewport: nalgebra_glm::make_vec4(&[0., 0., canvas.width() as f32, canvas.height() as f32]),
        };

        ret.shaders.insert(ShaderKind::Triangles, Shader::new(&ret.context, TRIANGLE_VS, TRIANGLE_FS)?);
        ret.shaders.insert(ShaderKind::Lines, Shader::new(&ret.context, LINE_VS, LINE_FS)?);
        ret.shaders.insert(ShaderKind::Points, Shader::new(&ret.context, POINT_VS, POINT_FS)?);
        Ok(ret)
    }

    // Take x, y pixels and map them to model space
    pub fn unproject(&self, x: i32, y: i32) -> (f32, f32) {
        let unprojected = nalgebra_glm::unproject(
            // Need to invert the y since canvas +Y goes downwards
            &nalgebra_glm::make_vec3(&[x as f32, (self.window_size.1 as i32 - y) as f32, 0.]),
            &nalgebra_glm::identity(),
            &self.view_matrix,
            self.viewport
        );
        (unprojected.x, unprojected.y)
    }

    pub fn set_bounds(&mut self, lower: (f32, f32), upper: (f32, f32)) {
        self.view_matrix = nalgebra_glm::ortho(lower.0 - 1.0, upper.0 + 1.0, lower.1 - 1.0, upper.1 + 1.0, 0.1, 1000.0);
    }

    pub fn draw(&self, static_data: &StaticGraphicsData, dynamic_data: &DynamicGraphicsData) {
        self.context.clear_color(self.clear_color[0], self.clear_color[1], self.clear_color[2], self.clear_color[3]);
        self.context.clear(GL::COLOR_BUFFER_BIT | GL::DEPTH_BUFFER_BIT);
        self.context.viewport(0, 0, self.window_size.0 as i32, self.window_size.1 as i32);

        let mut view_matrix = [0.; 16];
        view_matrix.clone_from_slice(self.view_matrix.as_slice());

        self.draw_triangles(
            &view_matrix,
            &static_data.triangle_position_vertices,
            &static_data.triangle_color_idx_vertices,
            &dynamic_data.triangle_indices,
            &static_data.colors_uniform,
        );

        self.draw_lines(
            &view_matrix,
            &dynamic_data.line_vertices
        );

        self.draw_points(
            &view_matrix,
            &static_data.point_position_vertices,
            &static_data.point_idx_vertices,
            &dynamic_data.selected_vertices,
        );
    }

    pub fn set_clear_color(&mut self, color: [f32; 4]) {
        self.clear_color = color;
    }

    fn draw_triangles(
        &self,
        view_matrix: &[f32; 16],
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

        // Set color and view matrix uniforms
        let colors_uniform = shader.get_uniform_location(&self.context, "colors");
        self.context.uniform3fv_with_f32_array(colors_uniform.as_ref(), colors);

        let view_matrix_uniform = shader.get_uniform_location(&self.context, "viewMatrix");
        self.context.uniform_matrix4fv_with_f32_array(view_matrix_uniform.as_ref(), false, view_matrix);

        // Draw triangles
        self.context.draw_elements_with_i32(GL::TRIANGLES, indices.len() as i32, GL::UNSIGNED_SHORT, 0);
    }

    fn draw_lines(
        &self, 
        view_matrix: &[f32; 16],
        vertices: &Vec<f32>
    ) {
        if vertices.len() == 0 { return }

        let shader = self.shaders.get(&ShaderKind::Lines).unwrap();
        self.context.use_program(Some(&shader.program));

        // Set up and buffer position attribute
        let pos_attrib = self.context.get_attrib_location(&shader.program, "position") as u32;
        self.buffer_f32_data(vertices, pos_attrib, 2);

        // Set view matrix uniform
        let view_matrix_uniform = shader.get_uniform_location(&self.context, "viewMatrix");
        self.context.uniform_matrix4fv_with_f32_array(view_matrix_uniform.as_ref(), false, view_matrix);

        // Draw disconnected lines
        self.context.line_width(2.0);
        self.context.draw_arrays(GL::LINES, 0, (vertices.len() >> 1) as i32);
    }

    fn draw_points(
        &self, 
        view_matrix: &[f32; 16],
        vertex_positions: &Vec<f32>,
        vertex_indices: &Vec<f32>,
        selected: &HashSet<u32>,
    ){
        if vertex_indices.len() == 0 { return }

        let shader = self.shaders.get(&ShaderKind::Points).unwrap();
        self.context.use_program(Some(&shader.program));

        // Set up and buffer position/index attributes
        let pos_attrib = self.context.get_attrib_location(&shader.program, "position") as u32;
        let idx_attrib = self.context.get_attrib_location(&shader.program, "index") as u32;
        self.buffer_f32_data(vertex_positions, pos_attrib, 2);
        self.buffer_f32_data(vertex_indices, idx_attrib, 1);

        // Set uniform for selected vertices
        let s_padded = selected.iter().map(|&i| i as i32).chain(std::iter::repeat(-1)).take(2).collect::<Vec<i32>>();
        let selected_vertices_uniform = shader.get_uniform_location(&self.context, "selected");
        self.context.uniform1iv_with_i32_array(selected_vertices_uniform.as_ref(), s_padded.as_slice());

        // Set view matrix uniform
        let view_matrix_uniform = shader.get_uniform_location(&self.context, "viewMatrix");
        self.context.uniform_matrix4fv_with_f32_array(view_matrix_uniform.as_ref(), false, view_matrix);

        // Draw points
        self.context.enable(GL::BLEND);
        self.context.blend_func(GL::SRC_ALPHA, GL::ONE_MINUS_SRC_ALPHA);
        self.context.draw_arrays(GL::POINTS, 0, vertex_indices.len() as i32);
        self.context.disable(GL::BLEND);
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