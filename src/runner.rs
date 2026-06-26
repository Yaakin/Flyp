use std::collections::HashMap;
use std::fs;
use std::any::Any;

use crate::parser::{Parser, Expr, Target};

#[derive(Clone, Debug)]
pub enum Value {
    Nil,
    Bool(bool),
    Number(f64),
    Str(String),

    Function{
        args: Vec<String>,
        value: Expr,
    },
    Native(for <'a, 'b> fn(&'a mut Runner, &'b Vec<Value>, Option<Value>) -> Value),
    Object(HashMap<String, Value>),
    List(Vec<Value>),
}

impl Value {
    fn to_bool(&self) -> bool {
        match self {
            Value::Nil => false,
            Value::Bool(b) => *b,
            Value::Number(x) => *x != 0.,
            Value::Str(s) => s.len() > 0,

            Value::Function{..} => true,
            Value::Native(_) => true,
            Value::Object(fields) => fields.len() > 0,
            Value::List(elements) => elements.len() > 0,
        }
    }

    pub fn repr(&self) -> String {
        match self {
            Value::Nil => "nil".to_string(),
            Value::Bool(b) => if *b { "true".to_string() } else { "false".to_string() },
            Value::Number(x) => format!("{x}"),
            Value::Str(s) => s.clone(),
            Value::Function{..} => format!("<Function>"),
        Value::Native(_) => format!("<NativeFunction>"),
            Value::Object(fields) => {
                let mut res = "[".to_string();
                for (key, val) in fields.iter() {
                    res.push_str(&format!("{}: {}; ", key, val.repr()));
                }
                if res.len() > 1 {
                    res.pop(); res.pop();
                }
                res.push(']');
                res
            },
            Value::List(elements) => {
                let mut res = "<".to_string();
                for val in elements {
                    res.push_str(&format!("{}; ", val.repr()));
                }
                res.pop(); res.pop();
                res.push('>');
                res
            },
        }
    }
}

impl std::fmt::Display for Value {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.repr())
    }
}

#[derive(Debug)]
pub struct Runner {
    pub loadables: HashMap<String, fn(&mut Runner) -> Value>,
    #[allow(dead_code)]
    pub internals: HashMap<String, Box<dyn Any>>,
    globals: HashMap<String, Value>,
    scopes: Vec<HashMap<String, Value>>,
}

impl Runner {
    pub fn new() -> Self {
        let mut loadables = HashMap::new();
        crate::modules::register_modules(&mut loadables);
        Self {
            loadables: loadables,
            internals: HashMap::new(),
            globals: HashMap::from([
                ("nil".to_string(),   Value::Nil),
                ("true".to_string(),  Value::Bool(true)),
                ("false".to_string(), Value::Bool(false)),

                ("import".to_string(), Value::Native(Runner::import)),
            ]),
            scopes: Vec::new(),
        }
    }

    fn push_scope(&mut self) {
        self.scopes.push(HashMap::new());
    }

    fn pop_scope(&mut self) {
        self.scopes.pop();
    }

    fn get_scope(&mut self) -> &mut HashMap<String, Value> {
        let n = self.scopes.len();
        if n > 0 {
            &mut self.scopes[n - 1]
        } else {
            &mut self.globals
        }
    }

    fn get_val(&mut self, name: &String) -> Value {
        let n = self.scopes.len();

        for i in 0..n {
            if self.scopes[n - i - 1].contains_key(name) {
                return self.scopes[n - i - 1].get(name).unwrap().clone();
            }
        }
        if self.globals.contains_key(name) {
            self.globals.get(name).unwrap().clone()
        } else {
            println!("Identifier {name} not found");
            Value::Nil
        }
    }

    fn get_ref(&mut self, name: &String) -> Option<&mut Value> {
        let n = self.scopes.len();

        for i in 0..n {
            if self.scopes[n - i - 1].contains_key(name) {
                return self.scopes[n - i - 1].get_mut(name);
            }
        }
        if self.globals.contains_key(name) {
            self.globals.get_mut(name)
        } else {
            self.get_scope().insert(name.to_string(), Value::Nil);
            self.get_scope().get_mut(name)
        }

    }
    
    fn set_val(&mut self, name: &String, val: Value) {
        self.get_scope().insert(name.to_string(), val);
    }

    fn access(&mut self, dest: Expr) -> Option<&mut Value> {
        match dest {
            Expr::Identifier(name) => self.get_ref(&name),
            Expr::Access(target) => {
                match target {
                    Target::Var(name) => self.get_ref(&name),
                    Target::Field{ var, field } => {
                        if let Some(Value::Object(dest)) = self.access(*var) {
                            dest.get_mut(&field)
                        } else {
                            println!("Trying to access a field from non-object value");
                            None
                        }
                    },
                    Target::Index{ var, index } => {
                        let Value::Number(i) = self.eval(&*index) else {
                            println!("Cannot index a list with non-number value");
                            return None;
                        };
                        if let Some(Value::List(dest)) = self.access(*var) {
                            dest.get_mut(i as usize)
                        } else {
                            println!("Cannot index non-list value");
                            None
                        }
                    }
                }
            },
            _ => {
                None
            }
        }
    }

    pub fn eval(&mut self, e: &Expr) -> Value {
        match e {
            Expr::Nil => Value::Nil,
            Expr::Number(x) => Value::Number(*x),
            Expr::Function{ args, value } => Value::Function {
                args: args.to_vec(),
                value: *value.clone(),
            },
            Expr::Str(s) => Value::Str(s.to_string()),
            Expr::Object(fields) => {
                let mut res = HashMap::new();

                for (key, val) in fields.iter() {
                    res.insert(key.to_string(), self.eval(val));
                }

                Value::Object(res)
            },
            Expr::List(elements) => {
                let mut res = Vec::new();

                for e in elements {
                    res.push(self.eval(e));
                }

                Value::List(res)
            }
            Expr::Identifier(name) => self.get_val(&name),
            Expr::Access(target) => {
                match target {
                    Target::Var(name) => self.get_val(&name),
                    Target::Field{ var, field } => {
                        if let Value::Object(fields) = self.eval(&**var) {
                            if fields.contains_key(field) {
                                fields.get(field).unwrap().clone()
                            } else {
                                println!("Trying to access inexistant field {field}");
                                Value::Nil
                            }
                        } else {
                            println!("Trying to access field from a non-object value");
                            Value::Nil
                        }
                    },
                    Target::Index{ var, index } => {
                        if let Value::List(elements) = self.eval(&**var) {
                            if let Value::Number(i) = self.eval(&**index) {
                                if (i as usize) < elements.len() {
                                    elements[i as usize].clone()
                                } else {
                                    Value::Nil
                                }
                            } else {
                                println!("Trying to access elements with a non-number index");
                                Value::Nil
                            }
                        } else {
                            println!("Trying to index a non-list value");
                            Value::Nil
                        }
                    },
                }
                            },
            Expr::FuncCall{ func, args: arg_exprs } => {
                if let Expr::Access(Target::Field{ var, field }) = &**func { // handle reflection
                    if let Value::Object(fields) = self.eval(&**var) {
                            if fields.contains_key(field) {
                                let mut arg_values = Vec::new();
                                for e in arg_exprs {
                                    arg_values.push(self.eval(e));
                                }
                                if let Value::Native(f) = fields.get(field).unwrap() {
                                    f(self, &arg_values, Some(Value::Object(fields)))
                                } else {
                                    println!("Cannot evaluate function call: expression is not a function");
                                    Value::Nil
                                }
                            } else {
                                println!("Trying to access inexistant field {field}");
                                Value::Nil
                            }
                        } else {
                            println!("Trying to access field from a non-object value");
                            Value::Nil
                        }

                } else {
                match self.eval(&**func) {
                    Value::Function{ args, value } => {
                        let mut arg_values = Vec::new();
                        for e in arg_exprs {
                            arg_values.push(self.eval(e));
                        }

                        self.push_scope();
                        for i in 0..args.len() {
                            if i < arg_values.len() {
                                self.set_val(&args[i], arg_values[i].clone());
                            } else {
                                self.set_val(&args[i], Value::Nil);
                            }
                        }
                        let res = self.eval(&value);
                        self.pop_scope();
                        res
                    },
                    Value::Native(f) => {
                        let mut arg_values = Vec::new();
                        for e in arg_exprs {
                            arg_values.push(self.eval(e));
                        }
                        f(self, &arg_values, None)
                    },
                    _ => {
                        println!("Cannot evaluate function call: expression is not a function");
                        Value::Nil
                    }
                }}
            },
            Expr::If{ cond, value, else_value } => {
                if self.eval(&**cond).to_bool() {
                    self.eval(&**value)
                } else {
                    self.eval(&**else_value)
                }
            },
            Expr::While{ cond, value, else_value } => {
                let mut executed_once = false;
                let mut res = Value::Nil;

                self.push_scope();
                while self.eval(&**cond).to_bool() {
                    executed_once = true;
                    res = self.eval(&**value);
                } 
                if !executed_once {
                    res = self.eval(&**else_value)
                }
                self.pop_scope();
                res
            },
            Expr::Binding{ target, value } => {
                let val = self.eval(&**value);
                if let Some(dest) = self.access(Expr::Access(target.clone())) {
                    *dest = val.clone();
                    val
                } else {
                    println!("Cannot access binding destination");
                    Value::Nil
                }
            },
            Expr::Chain(exprs) => {
                //self.push_scope();
                let mut result = Value::Nil;
                for e in exprs {
                    result = self.eval(&e);
                }
                //self.pop_scope();
                result
            }
        } 
    }

    fn import(&mut self, args: &Vec<Value>, _: Option<Value>) -> Value {
        if args.len() < 1 {
            println!("Import path expected");
            return Value::Nil;
        }
        if let Value::Str(path) = &args[0] {
            if let Ok(true) = fs::exists(&path) {
                if let Ok(content) = fs::read_to_string(path) {
                    let mut p = Parser::new(&content);
                    self.eval(&p.expr())
                } else {
                    println!("Error while reading imported file");
                    Value::Nil
                }
            } else if path.starts_with("std::") {
                let namespace = &path[5..];
                if self.loadables.contains_key(namespace) {
                    self.loadables.get(namespace).unwrap()(self)
                } else {
                    println!("Cannot find standard module {namespace}");
                    Value::Nil
                }
            } else {
                println!("Import path does not exist");
                Value::Nil
            }
        } else {
            println!("Expected a string path to import");
            Value::Nil
        }
    }
}
