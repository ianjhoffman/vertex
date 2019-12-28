#[macro_use] extern crate quick_error;

mod geometry;
mod puzzle_state;
mod display;
mod events;

use std::rc::Rc;
use std::cell::RefCell;
use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;

fn window() -> Result<web_sys::Window, JsValue> {
    web_sys::window().ok_or("No global window exists".into())
}

fn get_canvas() -> Result<web_sys::HtmlCanvasElement, JsValue> {
    let document = window()?.document().ok_or("Could not get document")?;
    let canvas = document.get_element_by_id("vertex-canvas").ok_or("Could not find canvas")?;
    Ok(canvas.dyn_into::<web_sys::HtmlCanvasElement>()?)
}

fn request_animation_frame(f: &Closure<dyn FnMut()>) -> Result<i32, JsValue> {
    window()?.request_animation_frame(f.as_ref().unchecked_ref())
}

#[wasm_bindgen]
pub fn run(puzzle: &str) -> Result<(), JsValue> {
    // Set up main components of the game
    let puzzle_data = geometry::PuzzleData::from_reader(&mut puzzle.as_bytes()).map_err(|e| e.to_string())?;
    let mut puzzle_state = puzzle_state::PuzzleState::from_data(&puzzle_data);
    let graphics = display::graphics::Graphics::from_canvas(&get_canvas()?).map_err(|e| e.to_string())?;
    let event_handler = events::EventHandler::init_from_canvas(&get_canvas()?)?;

    // Set up static and dynamic geometry
    let static_geometry = puzzle_data.get_static_graphics_data();
    let mut dynamic_geometry = puzzle_data.get_dynamic_graphics_data(&puzzle_state);

    let mut last_vertex_clicked: Option<u32> = None;

    // We need to do some funky stuff here to allow the animation frame
    // callback to reference itself (to request the next frame)
    let f = Rc::new(RefCell::new(None));
    let g = f.clone();
    *g.borrow_mut() = Some(Closure::wrap(Box::new(move || {
        if !puzzle_state.is_finished() {
            if let Ok(mut h) = event_handler.try_borrow_mut() {
                for event in h.pending() {
                    match event {
                        events::Event::MouseDown(x, y) => {
                            let unprojected = graphics.unproject(x, y);
                            if let Some(v) = puzzle_data.get_vertex_near(unprojected.0, unprojected.1, 0.075) {
                                if let Some(v2) = last_vertex_clicked {
                                    puzzle_state.connect_edge(&puzzle_data, &(v, v2));
                                    dynamic_geometry = puzzle_data.get_dynamic_graphics_data(&puzzle_state);
                                    last_vertex_clicked = None;
                                } else {
                                    last_vertex_clicked = Some(v);
                                }
                            } else if last_vertex_clicked.is_some() {
                                last_vertex_clicked = None;
                            }
                        }
                    }
                }
            }
        }

        graphics.draw(&static_geometry, &dynamic_geometry);
        request_animation_frame(f.borrow().as_ref().unwrap()).unwrap();
    }) as Box<dyn FnMut()>));
    request_animation_frame(g.borrow().as_ref().unwrap())?;
    Ok(())
}
