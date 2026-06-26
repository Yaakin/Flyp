use std::collections::HashMap;
use crate::runner::{Runner, Value};
use crate::modules::Module;

pub struct Math;

impl Math {
    pub fn sqrt(_r: &mut Runner, args: &Vec<Value>, _reflection: Option<Value>) -> Value {
        if args.len() < 1 {
            println!("Sqrt needs at least one argument");
            return Value::Nil;
        }
        if let Value::Number(x) = args[0] {
            Value::Number(x.sqrt())
        } else {
            println!("Cannot compute non-number square root");
            Value::Nil
        }
    }

    pub fn sub(_r: &mut Runner, args: &Vec<Value>, _reflection: Option<Value>) -> Value {
        if args.len() < 2 {
            println!("Sub operation needs two operands");
            return Value::Nil;
        }
        if let Value::Number(x1) = args[0] && let Value::Number(x2) = args[1] {
            Value::Number(x1 - x2)
        } else {
            println!("Cannot perform sub operation on non-number values");
            Value::Nil
        }
    }

    pub fn equ(_r: &mut Runner, args: &Vec<Value>, _reflection: Option<Value>) -> Value {
        if args.len() != 2 {
            println!("Cannot perform quality comparison without two operands");
            return Value::Nil;
        }
        if let Value::Number(x1) = args[0] && let Value::Number(x2) = args[1] {
            Value::Bool(x1 == x2)
        } else {
            println!("Cannot perform equality comparison on non-number values");
            Value::Nil
        }
    }
}

impl Module for Math {
    fn get_load_name() -> String {
        "math".to_string()
    }

    fn load(_r: &mut Runner) -> Value {
        Value::Object(HashMap::from([
            ("sqrt".to_string(), Value::Native(Math::sqrt)),
            ("equ".to_string(), Value::Native(Math::equ)),
            ("sub".to_string(), Value::Native(Math::sub)),
        ]))
    }
}
