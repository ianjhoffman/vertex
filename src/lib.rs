#[macro_use] extern crate quick_error;

mod geometry;
mod puzzle_state;
mod display;

use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;

fn window() -> Result<web_sys::Window, JsValue> {
    web_sys::window().ok_or("No global window exists".into())
}

fn alert(msg: &str) -> Result<(), JsValue> {
    window()?.alert_with_message(msg)
}

fn get_canvas() -> Result<web_sys::HtmlCanvasElement, JsValue> {
    let document = window()?.document().ok_or("Could not get document")?;
    let canvas = document.get_element_by_id("vertex-canvas").ok_or("Could not find canvas")?;
    Ok(canvas.dyn_into::<web_sys::HtmlCanvasElement>()?)
}

#[wasm_bindgen]
pub fn run(puzzle: &str) -> Result<(), JsValue> {
    let puzzle_data = geometry::PuzzleData::from_reader(&mut puzzle.as_bytes()).map_err(|e| e.to_string())?;
    let mut puzzle_state = puzzle_state::PuzzleState::from_data(&puzzle_data);
    let mut graphics = display::graphics::Graphics::from_canvas(&get_canvas()?).map_err(|e| e.to_string())?;

    let static_geometry = puzzle_data.get_static_graphics_data();
    let mut dynamic_geometry = puzzle_data.get_dynamic_graphics_data(&puzzle_state);

    while !puzzle_state.is_finished() {
        graphics.draw(&static_geometry, &dynamic_geometry);
        if let Some(message) = window()?.prompt_with_message("Unlock an edge")? {
            let split = message.split_whitespace().collect::<Vec<&str>>();
            if split.len() != 2 {
                alert(&format!("Invalid edge: {}", message))?;
                continue
            }
            if let (Ok(v0), Ok(v1)) = (split[0].parse::<u32>(), split[1].parse::<u32>()) {
                let edge = (v0, v1);
                if !puzzle_data.is_valid_edge(&edge) {
                    alert(&format!("Invalid edge: {}", message))?;
                    continue
                }
                puzzle_state.connect_edge(&puzzle_data, &edge);
                alert(&format!("Unlocked edge: {:?}!", edge))?;
                dynamic_geometry = puzzle_data.get_dynamic_graphics_data(&puzzle_state);
            } else {
                alert(&format!("Invalid edge: {}", message))?;
                continue
            }
        }
    }

    alert("Puzzle complete!")?;
    Ok(())
}
