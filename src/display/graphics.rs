use super::{error::*, shader::*};
use super::super::geometry::{StaticGraphicsData, DynamicGraphicsData};
use std::rc::Rc;
use std::collections::HashMap;
use wasm_bindgen::JsCast;
use web_sys::WebGlRenderingContext as GL;

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
            &dynamic_data.triangle_indices
        );

        self.draw_lines(&dynamic_data.line_vertices);

        self.draw_points(
            &static_data.point_position_vertices,
            &static_data.point_idx_vertices,
        );
    }

    fn draw_triangles(&self, vertex_positions: &Vec<f32>, vertex_colors: &Vec<f32>, indices: &Vec<u32>) {
        let shader = self.shaders.get(&ShaderKind::Triangles).unwrap();
        self.context.use_program(Some(&shader.program));
    }

    fn draw_lines(&self, vertices: &Vec<f32>) {
        let shader = self.shaders.get(&ShaderKind::Lines).unwrap();
        self.context.use_program(Some(&shader.program));
    }

    fn draw_points(&self, vertex_positions: &Vec<f32>, vertex_indices: &Vec<f32>) {
        let shader = self.shaders.get(&ShaderKind::Points).unwrap();
        self.context.use_program(Some(&shader.program));
    }
}