use std::cell::RefCell;
use std::rc::Rc;
use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;

#[derive(Debug)]
pub enum Event {
    MouseDown(i32, i32),
    MouseMove(i32, i32),
    MouseUp(i32, i32),
    MouseLeave,
}

pub struct EventHandler {
    event_queue: Vec<Event>,
}

impl EventHandler {
    pub fn init_from_canvas(canvas: &web_sys::HtmlCanvasElement) -> Result<Rc<RefCell<EventHandler>>, JsValue> {
        let out = Rc::new(RefCell::new(EventHandler{
            event_queue: vec![],
        }));

        {
            let handler = out.clone();
            let closure = Closure::wrap(Box::new(move |event: web_sys::PointerEvent| {
                if let Ok(mut h) = handler.try_borrow_mut() {
                    h.add_event(Event::MouseDown(event.offset_x(), event.offset_y()));
                }
            }) as Box<dyn FnMut(_)>);
            canvas.add_event_listener_with_callback("mousedown", closure.as_ref().unchecked_ref())?;
            closure.forget();
        }

        {
            let handler = out.clone();
            let closure = Closure::wrap(Box::new(move |event: web_sys::PointerEvent| {
                if let Ok(mut h) = handler.try_borrow_mut() {
                    h.add_event(Event::MouseMove(event.offset_x(), event.offset_y()));
                }
            }) as Box<dyn FnMut(_)>);
            canvas.add_event_listener_with_callback("mousemove", closure.as_ref().unchecked_ref())?;
            closure.forget();
        }

        {
            let handler = out.clone();
            let closure = Closure::wrap(Box::new(move |event: web_sys::PointerEvent| {
                if let Ok(mut h) = handler.try_borrow_mut() {
                    h.add_event(Event::MouseUp(event.offset_x(), event.offset_y()));
                }
            }) as Box<dyn FnMut(_)>);
            canvas.add_event_listener_with_callback("mouseup", closure.as_ref().unchecked_ref())?;
            closure.forget();
        }

        {
            let handler = out.clone();
            let closure = Closure::wrap(Box::new(move |_: web_sys::PointerEvent| {
                if let Ok(mut h) = handler.try_borrow_mut() {
                    h.add_event(Event::MouseLeave);
                }
            }) as Box<dyn FnMut(_)>);
            canvas.add_event_listener_with_callback("mouseleave", closure.as_ref().unchecked_ref())?;
            closure.forget();
        }

        Ok(out)
    }

    fn add_event(&mut self, event: Event) {
        self.event_queue.push(event);
    }

    pub fn pending(&mut self) -> Box<dyn Iterator<Item = Event>> {
        Box::new(std::mem::replace(&mut self.event_queue, vec![]).into_iter())
    }
}