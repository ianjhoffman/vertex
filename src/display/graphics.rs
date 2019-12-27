use super::{error::*, shader::*};
use std::rc::Rc;
use std::collections::HashMap;
use wasm_bindgen::JsCast;
use web_sys::WebGlRenderingContext;

static TRIANGLE_VS: &'static str = include_str!("./shaders/triangle-vertex.glsl");
static TRIANGLE_FS: &'static str = include_str!("./shaders/triangle-fragment.glsl");

pub struct Graphics {
    context: Rc<WebGlRenderingContext>,
    shaders: HashMap<ShaderKind, Shader>,
}

impl Graphics {
    pub fn from_canvas(canvas: &web_sys::HtmlCanvasElement) -> Result<Graphics, GraphicsError> {
        let context = canvas.get_context("webgl")
            .map_err(|_| GraphicsError::ContextFailed)?
            .ok_or(GraphicsError::ContextFailed)?
            .dyn_into::<WebGlRenderingContext>().map_err(|_| GraphicsError::ContextFailed)?;

        let mut ret = Graphics{
            context: Rc::new(context),
            shaders: HashMap::new(),
        };

        ret.shaders.insert(ShaderKind::Triangles, Shader::new(&ret.context, TRIANGLE_VS, TRIANGLE_FS)?);
        Ok(ret)
    }
}