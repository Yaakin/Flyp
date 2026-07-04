use std::collections::HashMap;
use std::fs;

use crate::parser::{Parser, Expr, Target};

type MemId = usize;

#[derive(Clone, Debug)]
pub struct MemCell {
    //pub id: MemId,
    pub data: Stored,
}

#[derive(Copy, Clone, Debug)]
pub enum Value {
    Nil,
    Bool(bool),
    Number(f64),
    Ref(MemId),
}

impl Value {
    fn to_bool(&self, r: &Runner) -> bool {
        match self {
            Value::Nil => false,
            Value::Bool(b) => *b,
            Value::Number(x) => *x != 0.,
            Value::Ref(id) => {
                if let Some(s) = r.get_mem(*id) {
                    s.to_bool()
                } else {
                    false
                }
            }
        }
    }

    pub fn repr(&self, r: &Runner) -> String {
        match self {
            Value::Nil => "nil".to_string(),
            Value::Bool(b) => if *b { "true".to_string() } else { "false".to_string() },
            Value::Number(x) => format!("{x}"),
            Value::Ref(id) => {
                if let Some(s) = r.get_mem(*id) {
                    s.repr(r)
                } else {
                    Value::Nil.repr(r)
                }
            }
        }
    }
}

#[derive(Clone, Debug)]
pub enum Function {
    Defined {
        args: Vec<String>,
        value: Expr,
        closure: HashMap<String, Value>,
        reflection: bool,
    },
    Native(for <'a, 'b> fn(&'a mut Runner, &'b Vec<Value>, reflection: Option<Value>) -> Value),
}

#[derive(Clone, Debug)]
pub enum Stored {
    Str(String),

    Func(Function),
    Object(HashMap<String, Value>),
    List(Vec<Value>),
}

impl Stored {
    pub fn to_bool(&self) -> bool {
        match self {
            Stored::Str(s) => s.len() > 0,
            Stored::Func(_) => true,
            Stored::Object(fields) => fields.len() > 0,
            Stored::List(elements) => elements.len() > 0,
        }
    }
    
    pub fn repr(&self, r: &Runner) -> String {
       match self {
            Stored::Str(s) => s.clone(),
            Stored::Func(Function::Defined {..}) => format!("<Function>"),
            Stored::Func(Function::Native(_)) => format!("<NativeFunction>"),
            Stored::Object(fields) => {
                let mut res = "[".to_string();
                for (key, val) in fields.iter() {
                    res.push_str(&format!("{}: {}; ", key, val.repr(r)));
                }
                if res.len() > 1 {
                    res.pop(); res.pop();
                }
                res.push(']');
                res
            },
            Stored::List(elements) => {
                let mut res = "<".to_string();
                for val in elements {
                    res.push_str(&format!("{}; ", val.repr(r)));
                }
                res.pop(); res.pop();
                res.push('>');
                res
            },

       } 
    }
}

#[derive(Debug)]
pub struct Runner {
    pub loadables: HashMap<String, fn(&mut Runner) -> Value>,
    pub memory: HashMap<MemId, MemCell>,
    pub scopes: Vec<HashMap<String, Value>>,
    pub globals: HashMap<String, Value>,
}

impl Runner {
    pub fn new() -> Self {
        let mut loadables = HashMap::new();
        crate::modules::register_modules(&mut loadables);
        Self {
            loadables: loadables,
            memory: HashMap::from([
                (0 as MemId, MemCell { data: Stored::Func(Function::Native(Runner::import)) }),
                (1 as MemId, MemCell { data: Stored::Func(Function::Native(Runner::id)) }),
            ]),
            scopes: Vec::new(),
            globals: HashMap::from([
                ("nil".to_string(), Value::Nil),
                ("true".to_string(), Value::Bool(true)),
                ("false".to_string(), Value::Bool(false)),
                ("import".to_string(), Value::Ref(0)),
                ("id".to_string(), Value::Ref(1)),
            ]),
        }
    }

    pub fn store(&mut self, data: Stored) -> MemId {
        let id = self.memory.len();
        let cell = MemCell {
            data: data,
        };
        self.memory.insert(id, cell);
        id
    }

    pub fn register(&mut self, data: Stored) -> Value {
        let id = self.store(data);
        Value::Ref(id)
    }

    pub fn get_mem(&self, id: MemId) -> Option<&Stored> {
        if let Some(cell) = self.memory.get(&id) {
            Some(&cell.data)
        } else { None }
    }

    pub fn get_mem_mut(&mut self, id: MemId) -> Option<&mut Stored> {
        if let Some(cell) = self.memory.get_mut(&id) {
            Some(&mut cell.data)
        } else { None }
    }

    pub fn get_scope(&mut self) -> &mut HashMap<String, Value> {
        if self.scopes.len() > 0 {
            let n = self.scopes.len() - 1;
            &mut self.scopes[n]
        } else {
            &mut self.globals
        }
    }


    pub fn get_val(&mut self, t: Target) -> Value {
        match t {
            Target::Var(name) => {
                let scope = self.get_scope();
                if scope.contains_key(&name) {
                    *scope.get(&name).unwrap()
                } else if self.globals.contains_key(&name) {
                    *self.globals.get(&name).unwrap()
                } else {
                    println!("Cannot find identifier {name} in this scope THIS MESSAGE IS NOT NORMAL IT NEEDS TO BE FIXED");
                    Value::Nil
                }
            },
            Target::Field { var, field } => {
                if let Value::Ref(id) = self.eval(&*var) && let Some(Stored::Object(fields)) = self.get_mem(id) {
                    if fields.contains_key(&field) {
                        *fields.get(&field).unwrap()
                    } else {
                        println!("Cannot find field {field}");
                        Value::Nil
                    }
                } else {
                    println!("Trying to access a field from non-object value");
                    Value::Nil
                }
            },
            Target::Index { var, index } => {
                if let Value::Number(i) = self.eval(&*index) {
                    if let Value::Ref(id) = self.eval(&var) && let Some(Stored::List(vals)) = self.get_mem(id) {
                        if (i as usize) < vals.len() {
                            vals[i as usize]
                        } else {
                            Value::Nil
                        }
                    } else {
                        println!("Cannot index a non-list value");
                        Value::Nil
                    }
                } else {
                    println!("Cannot index a list with non-number value");
                    Value::Nil
                }
            }
        }
    }

    pub fn set_val(&mut self, t: Target, val: Value) {
        match t {
            Target::Var(name) => {
                if self.get_scope().contains_key(&name) {
                    self.get_scope().insert(name, val);
                } else if self.globals.contains_key(&name) {
                    self.globals.insert(name, val);
                } else {
                    self.get_scope().insert(name, val);
                }
            },
            Target::Field { var, field } => {
                if let Value::Ref(id) = self.eval(&*var) && let Some(Stored::Object(fields)) = self.get_mem_mut(id) {
                    fields.insert(field, val);
                } else {
                    println!("Trying to set a field from non-object value");
                }
            },
            Target::Index { var, index } => {
                if let Value::Number(i) = self.eval(&*index) {
                    if let Value::Ref(id) = self.eval(&var) && let Some(Stored::List(vals)) = self.get_mem_mut(id) {
                        if (i as usize) < vals.len() {
                            vals[i as usize] = val;
                        } else {
                            println!("Index out of bounds");
                        }
                    } else {
                        println!("Cannot index a non-list value");
                    }
                } else {
                    println!("Cannot index a list with non-number value");
                }
            }
        }
    }

    pub fn needs_reflection(&mut self, val: &Expr) -> Option<(Function, Option<Value>)> {
        if let Expr::Access(Target::Field { var, field }) = val {
            if let Value::Ref(id) = self.eval(var) && let Some(Stored::Object(fields)) = self.get_mem(id) {
                if fields.contains_key(field) && let Value::Ref(f_id) = fields.get(field).unwrap() && let Some(Stored::Func(f)) = self.get_mem(*f_id) {
                    Some((f.clone(), Some(Value::Ref(id))))
                } else {
                    None
                }
            } else {
                None
            }
        } else {
            if let Value::Ref(id) = self.eval(val) && let Some(Stored::Func(f)) = self.get_mem(id) {
                Some((f.clone(), None))
            } else {
                None
            }
        }
    }

    pub fn eval(&mut self, e: &Expr) -> Value {
        match e {
            Expr::Nil => Value::Nil,
            Expr::Number(x) => Value::Number(*x),
            Expr::Str(s) => {
                let id =self.store(Stored::Str(s.to_string()));
                Value::Ref(id)
            },
            Expr::Object(fields) => {
                let mut res = HashMap::new();

                for (key, val) in fields.iter() {
                    res.insert(key.to_string(), self.eval(val));
                }

                let id = self.store(Stored::Object(res));
                Value::Ref(id)
            },
            Expr::List(elements) => {
                let mut res = Vec::new();

                for e in elements {
                    res.push(self.eval(e));
                }

                let id = self.store(Stored::List(res));
                Value::Ref(id)
            },
            Expr::Function { args, value, closure, reflection } => {
                let mut closed = HashMap::<String, Value>::new();
                for name in closure {
                    closed.insert(name.clone(), self.eval(&Expr::Access(Target::Var(name.clone()))));
                }
                let id = self.store(Stored::Func(Function::Defined { args: args.to_vec(), value: *value.clone(), closure: closed, reflection: *reflection }));
                Value::Ref(id)
            },
            Expr::If { cond, value, else_value } => {
                if self.eval(cond).to_bool(&self) {
                    self.eval(value)
                } else {
                    self.eval(else_value)
                }
            },
            Expr::While { cond, value, else_value } => {
                let mut ran_once = false;
                let mut res = Value::Nil;
                while self.eval(cond).to_bool(&self) {
                    res = self.eval(value);
                    ran_once = true;
                }
                if !ran_once {
                    res = self.eval(else_value);
                }
                res
            },
            Expr::Binding { target, value } => {
                let v = self.eval(value);
                self.set_val(target.clone(), v);
                v
            },
            Expr::Access(target) => {
                self.get_val(target.clone())
            },
            Expr::FuncCall { func, args: arg_exprs } => {
                match self.needs_reflection(func) {
                    Some((f, refl)) => {
                        let mut arg_values = Vec::with_capacity(arg_exprs.len());
                        for e in arg_exprs {
                            arg_values.push(self.eval(e));
                        }

                        self.call_func(f, arg_values, refl)
                    },
                    None => {
                        println!("Cannot call non-function value");
                        Value::Nil
                    }
                }
            },
            Expr::Chain(exprs) => {
                let mut res = Value::Nil;
                for e in exprs {
                    res = self.eval(e);
                }
                res
            },
        }
    }

    pub fn call_func(&mut self, f: Function, args: Vec<Value>, refl: Option<Value>) -> Value {
        match f {
            Function::Defined { args: arg_names, value, closure, reflection } => {
                self.scopes.push(HashMap::new());
                for (k, v) in closure {
                    self.set_val(Target::Var(k.to_string()), v);
                }

                for i in 0..arg_names.len() {
                    if i < args.len() {
                        self.set_val(Target::Var(arg_names[i].clone()), args[i].clone());
                    } else {
                        self.set_val(Target::Var(arg_names[i].clone()), Value::Nil);
                    }
                }
                
                if reflection {
                    if refl.is_some() {
                        self.set_val(Target::Var("self".to_string()), refl.unwrap());
                    } else {
                        self.set_val(Target::Var("self".to_string()), Value::Nil);
                    }
                } 

                let res = self.eval(&value);
                self.scopes.pop();
                res
            },
            Function::Native(f) => f(self, &args, refl)
        }
    }

    fn import(&mut self, args: &Vec<Value>, _: Option<Value>) -> Value {
        if args.len() < 1 {
            println!("Import path expected");
            return Value::Nil;
        }
        if let Value::Ref(id) = &args[0] && let Some(Stored::Str(path)) = self.get_mem(*id) {
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

    fn id(&mut self, args: &Vec<Value>, _: Option<Value>) -> Value {
        if args.len() < 1 {
            println!("Invalid arguments for id");
            return Value::Nil;
        }

        match args[0] {
            Value::Ref(id) => Value::Number(id as f64),
            _ => Value::Nil,
        }
    }
}
