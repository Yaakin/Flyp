use std::collections::HashMap;
use crate::runner::{Runner, Value};
use crate::modules::Module;

use sdl3::pixels::Color;
use sdl3::event::Event;

pub struct Sdl3Internals {
    //context: sdl3::Sdl,
    //video_subsystem: sdl3::VideoSubsystem,
    //window: sdl3::video::Window,
    canvas: sdl3::render::WindowCanvas,

    event_pump: sdl3::EventPump,
}

impl Sdl3Internals {
    fn set_color(runner: &mut Runner, args: &Vec<Value>, _reflection: Option<Value>) -> Value {
        if args.len() != 3 {
            println!("sdl3.set_color expects 3 arguments");
            return Value::Nil;
        }

        if let Value::Number(r) = args[0] && let Value::Number(g) = args[1] && let Value::Number(b) = args[2] {
            let i: &mut Sdl3Internals = runner.internals.get_mut(&SDL3::get_load_name())
                .unwrap()
                .downcast_mut::<Sdl3Internals>()
                .unwrap();
            i.canvas.set_draw_color(Color::RGB(r as u8, g as u8, b as u8));
        } else {
            println!("Invalid arguments for sdl3.set_color");
        }

        Value::Nil
    }

    fn clear(r: &mut Runner, _args: &Vec<Value>, _reflection: Option<Value>) -> Value {
        let i: &mut Sdl3Internals = r.internals.get_mut(&SDL3::get_load_name())
            .unwrap()
            .downcast_mut::<Sdl3Internals>()
            .unwrap();
        i.canvas.clear();
        Value::Nil
    }

    fn present(r: &mut Runner, _args: &Vec<Value>, _reflection: Option<Value>) -> Value {
        let i: &mut Sdl3Internals = r.internals.get_mut(&SDL3::get_load_name())
            .unwrap()
            .downcast_mut::<Sdl3Internals>()
            .unwrap();
        i.canvas.present();
        Value::Nil
    }

    fn poll_event(r: &mut Runner, _args: &Vec<Value>, _reflection: Option<Value>) -> Value {
        let i: &mut Sdl3Internals = r.internals.get_mut(&SDL3::get_load_name())
            .unwrap()
            .downcast_mut::<Sdl3Internals>()
            .unwrap();
        match i.event_pump.poll_event() {
            None => Value::Nil,
            Some(Event::Quit{..}) => Value::Object(HashMap::from([("Event_Quit".to_string(), Value::Nil)])),
            Some(Event::KeyDown{ keycode: Some(kc), .. }) => {
                Value::Object(HashMap::from([
                    ("Event_KeyDown".to_string(), Value::Nil),
                    ("keycode".to_string(), Value::Number(kc as u64 as f64)),
                ]))
            },
            Some(Event::MouseMotion{ x, y, .. }) => {
                Value::Object(HashMap::from([
                    ("Event_MouseMotion".to_string(), Value::Nil),
                    ("x".to_string(), Value::Number(x.into())),
                    ("y".to_string(), Value::Number(y.into())),
                ]))
            }
            _ => Value::Object(HashMap::new()),
        }
    }

    fn rect(r: &mut Runner, args: &Vec<Value>, _reflection: Option<Value>) -> Value {
        if args.len() < 1 {
            println!("Invalid arguments for sdl3.rect");
            return Value::Nil;
        }
        let i: &mut Sdl3Internals = r.internals.get_mut(&SDL3::get_load_name())
            .unwrap()
            .downcast_mut::<Sdl3Internals>()
            .unwrap();

        if let Value::Object(rect) = &args[0] {
            if rect.contains_key("x")
            && rect.contains_key("y")
            && rect.contains_key("w")
            && rect.contains_key("h") {
                if let Value::Number(x) = rect.get("x").unwrap()
                && let Value::Number(y) = rect.get("y").unwrap()
                && let Value::Number(w) = rect.get("w").unwrap()
                && let Value::Number(h) = rect.get("h").unwrap() {
                    let _ = i.canvas.fill_rect(sdl3::render::FRect::new(*x as f32, *y as f32, *w as f32, *h as f32));
                } else {
                    println!("Invalid arguments for sdl3.rect");
                }
            } else {
                println!("Invalid arguments for sdl3.rect");
            }
        } else {
            println!("Invalid arguments for sdl3.rect");
        }
        Value::Nil
    }
}

pub struct SDL3;

impl SDL3 {
    pub fn init(r: &mut Runner, _args: &Vec<Value>, _: Option<Value>) -> Value {
        let context = sdl3::init().unwrap();
        let video_subsystem = context.video().unwrap();

        let window = video_subsystem.window("Flyp", 800, 600)
            .position_centered()
            .resizable()
            .build()
            .unwrap();
        let canvas = window.clone().into_canvas();

        let event_pump = context.event_pump().unwrap();

        let i = Sdl3Internals {
            //context: context,
            //video_subsystem: video_subsystem,
            //window: window,
            canvas: canvas,

            event_pump: event_pump,
        };
        r.internals.insert(SDL3::get_load_name(), Box::new(i));

        Value::Object(HashMap::from([
            ("set_color".to_string(), Value::Native(Sdl3Internals::set_color)),
            ("clear".to_string(), Value::Native(Sdl3Internals::clear)),
            ("present".to_string(), Value::Native(Sdl3Internals::present)),
            ("poll_event".to_string(), Value::Native(Sdl3Internals::poll_event)),
            ("rect".to_string(), Value::Native(Sdl3Internals::rect)),
        ]))
    }

    pub fn get_keycodes() -> HashMap<String, Value> {
        HashMap::from([
            ("ESCAPE".to_string(), Value::Number(27.)),
        ])
    }
}

impl Module for SDL3 {
    fn get_load_name() -> String {
        "sdl3".to_string()
    }

    fn load(_r: &mut Runner) -> Value {
        Value::Object(HashMap::from([
            ("init".to_string(), Value::Native(SDL3::init)),
            ("keycodes".to_string(), Value::Object(SDL3::get_keycodes())),
        ]))
    }
}
