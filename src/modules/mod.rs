use std::collections::HashMap;
use crate::runner::{NativeObject, Value, Runner};

mod math;
mod io;
mod list;

pub trait Module: NativeObject {
    fn name() -> String;
    fn load(r: &mut Runner) -> Value;
}

pub fn register_modules(table: &mut HashMap<String, fn(&mut Runner) -> Value>) {
    table.insert(math::Math::name(), math::Math::load);
    table.insert(io::Io::name(), io::Io::load);
    table.insert(list::List::name(), list::List::load);
}
