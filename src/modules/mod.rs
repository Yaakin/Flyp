use std::collections::HashMap;
use crate::runner::{Runner, Value};

mod math;
mod io;

pub trait Module {
    fn get_load_name() -> String;
    fn load(_: &mut Runner) -> Value;
}

#[macro_export] macro_rules! register_module {
    ($r:expr, $m:ty) => {
        $r.insert(<$m>::get_load_name(), <$m>::load);
    }
}

#[macro_export] macro_rules! internals {
    ($r:expr, $m:ty) => {
        $r.internals.get_mut(&<$m>::get_load_name())
            .unwrap()
            .downcast_mut::<$m>()
            .unwrap();
    }
}

pub fn register_modules(table: &mut HashMap<String, fn(&mut Runner) -> Value>) {
    register_module!(table, math::Math);
    register_module!(table, io::Io);
}
