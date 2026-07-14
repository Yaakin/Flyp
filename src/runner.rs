use std::collections::HashMap;
use std::rc::Rc;
use std::cell::RefCell;
use std::fs;

use crate::parser::{Parser, Expr, Target};

pub type StrId      = usize;
pub type ListId     = usize;
pub type ObjectId   = usize;
pub type FunctionId = usize;

pub type NativeFunction = dyn for<'a> FnMut(&'a mut Runner, &'a [Value]) -> Value;

pub trait NativeObject {
    fn get(&self, field: &str) -> Value;
}

#[derive(Clone)]
pub enum Value {
    Nil,
    Bool(bool),
    Number(f64),

    Str(StrId),
    List(ListId),
    Object(ObjectId),
    Function(FunctionId),

    NativeObject(Rc<RefCell<dyn NativeObject>>),
    NativeFunction(Rc<RefCell<NativeFunction>>),
    //Native(for <'a, 'b> fn(&'a mut Runner, &'b Vec<Value>, reflection: Option<Value>) -> Value),
}

impl Value {
    fn to_bool(&self, r: &Runner) -> bool {
        match self {
            Value::Nil => false,
            Value::Bool(b) => *b,
            Value::Number(x) => *x != 0.,
            Value::Str(id) => r.get_str(*id).len() > 0,
            Value::List(id) => r.get_list(*id).len() > 0,
            Value::Object(id) => r.get_object(*id).len() > 0,
            Value::Function(_) => true,
            Value::NativeObject(_) => true,
            Value::NativeFunction(_) => true,
        }
    }

    pub fn repr(&self, r: &Runner) -> String {
        match self {
            Value::Nil => "nil".to_string(),
            Value::Bool(b) => if *b { "true".to_string() } else { "false".to_string() },
            Value::Number(x) => format!("{}", (x*10000.).round() / 10000.),
            Value::Str(id) => r.get_str(*id).to_string(),
            Value::List(id) => {
                let mut res = String::from("<");
                for x in r.get_list(*id) {
                    res.push_str(&x.repr(r));
                    res.push_str("; ");
                }
                if res.len() > 1 {
                    res.pop();
                    res.pop();
                }
                res.push('>');
                res
            },
            Value::Object(id) => {
                let mut res = String::from("[");
                for (k, v) in r.get_object(*id) {
                    res.push_str(k);
                    res.push_str(": ");
                    res.push_str(&v.repr(r));
                    res.push_str("; ");
                }
                if res.len() > 1 {
                    res.pop();
                    res.pop();
                }
                res.push(']');
                res
            },
            Value::Function(_) => "<Function>".to_string(),
            Value::NativeObject(_) => "<NativeObject> (to be implemented)".to_string(),
            Value::NativeFunction(_) => "<NativeFunction>".to_string(),
        }
    }
}

#[derive(Clone)]
pub struct Function {
    args: Vec<String>,
    value: Expr,
    closure: HashMap<String, Value>,
    reflection: bool,
}

pub struct Runner {
    pub loadables: HashMap<String, fn(&mut Runner) -> Value>,

    pub strings: HashMap<StrId, String>,
    pub lists: HashMap<ListId, Vec<Value>>,
    pub objects: HashMap<ObjectId, HashMap<String, Value>>,
    pub functions: HashMap<FunctionId, Function>,

    pub scopes: Vec<HashMap<String, Value>>,
    pub globals: HashMap<String, Value>,
}

impl Runner {
    pub fn new() -> Self {
        let mut loadables = HashMap::new();
        crate::modules::register_modules(&mut loadables);
        Self {
            loadables: loadables,

            strings: HashMap::new(),
            lists: HashMap::new(),
            objects: HashMap::new(),
            functions: HashMap::new(),

            scopes: Vec::new(),
            globals: HashMap::from([
                ("nil".to_string(), Value::Nil),
                ("true".to_string(), Value::Bool(true)),
                ("false".to_string(), Value::Bool(false)),
                ("import".to_string(), Value::NativeFunction(Rc::new(RefCell::new(Runner::import)))),
                //("io".to_string(), Value::NativeObject(Rc::new(RefCell::new(modules::Io::new())))),
            ]),
        }
    }

    pub fn get_str(&self, id: StrId) -> &str {
        if let Some(s) = self.strings.get(&id) {
            s
        } else {
            panic!("Unable to retrieve str with id {id}, interpreter memory corrupted");
        }
    }

    pub fn get_list(&self, id: ListId) -> &Vec<Value> {
        if let Some(s) = self.lists.get(&id) {
            s
        } else {
            panic!("Unable to retrieve list with id {id}, interpreter memory corrupted");
        }
    }

    pub fn get_object(&self, id: ObjectId) -> &HashMap<String, Value> {
        if let Some(s) = self.objects.get(&id) {
            s
        } else {
            panic!("Unable to retrieve object with id {id}, interpreter memory corrupted");
        }
    }

    pub fn get_func(&self, id: FunctionId) -> &Function {
        if let Some(s) = self.functions.get(&id) {
            s
        } else {
            panic!("Unable to retrieve function with id {id}, interpreter memory corrupted");
        }
    }

    pub fn get_str_mut(&mut self, id: StrId) -> &mut str {
        if let Some(s) = self.strings.get_mut(&id) {
            s
        } else {
            panic!("Unable to retrieve str with id {id}, interpreter memory corrupted");
        }
    }

    pub fn get_list_mut(&mut self, id: ListId) -> &mut Vec<Value> {
        if let Some(l) = self.lists.get_mut(&id) {
            l
        } else {
            panic!("Unable to retrieve list with id {id}, interpreter memory corrupted");
        }
    }

    pub fn get_object_mut(&mut self, id: ObjectId) -> &mut HashMap<String, Value> {
        if let Some(o) = self.objects.get_mut(&id) {
            o
        } else {
            panic!("Unable to retrieve object with id {id}, interpreter memory corrupted");
        }
    }

    pub fn get_func_mut(&mut self, id: FunctionId) -> &mut Function {
        if let Some(f) = self.functions.get_mut(&id) {
            f
        } else {
            panic!("Unable to retrieve function with id {id}, interpreter memory corrupted");
        }
    }

    pub fn get_scope(&mut self) -> &mut HashMap<String, Value> {
        if self.scopes.len() > 0 {
            let n = self.scopes.len() - 1;
            &mut self.scopes[n]
        } else {
            &mut self.globals
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
                if let Value::Object(id) = self.eval(&*var) {
                    self.get_object_mut(id).insert(field, val);
                } else {
                    println!("Trying to set a field from non-object value");
                }
            },
            Target::Index { var, index } => {
                if let Value::Number(i) = self.eval(&*index) {
                    if let Value::List(id) = self.eval(&var) {
                        let vals = self.get_list_mut(id);
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

    pub fn get_val(&mut self, t: Target) -> Value {
        match t {
            Target::Var(name) => {
                let scope = self.get_scope();
                if scope.contains_key(&name) {
                    scope.get(&name).unwrap().clone()
                } else if self.globals.contains_key(&name) {
                    self.globals.get(&name).unwrap().clone()
                } else {
                    println!("Cannot find identifier {name} in this scope");
                    Value::Nil
                }
            },
            Target::Field { var, field } => {
                match self.eval(&*var) {
                    Value::Object(id) => {
                        let o = self.get_object(id);
                        if o.contains_key(&field) {
                            o.get(&field).unwrap().clone()
                        } else {
                            println!("Cannot find field {field}");
                            Value::Nil
                        }
                    },
                    Value::NativeObject(o) => {
                        o.as_ref().borrow_mut().get(&field)
                    },
                    _ => {
                        println!("Trying to access a field from a non-object value");
                        Value::Nil
                    }
                }
            },
            Target::Index { var, index } => {
                if let Value::Number(i) = self.eval(&*index) {
                    if let Value::List(id) = self.eval(&*var) {
                        let l = self.get_list(id);
                        if (i as usize) < l.len() {
                            l[i as usize].clone()
                        } else {
                            println!("Index out of bounds");
                            Value::Nil
                        }
                    } else {
                        println!("Cannnot index a non-list value");
                        Value::Nil
                    }
                } else {
                    println!("Cannot index a list with a non-number value");
                    Value::Nil
                }
            }
        }
    }

    pub fn eval(&mut self, e: &Expr) -> Value {
        match e {
            Expr::Nil => Value::Nil,
            Expr::Number(x) => Value::Number(*x),
            Expr::Str(s) => {
                let id = self.strings.len();
                self.strings.insert(id, s.to_string());
                Value::Str(id)
            },
            Expr::Object(o) => {
                let mut res = HashMap::new();
                let id = self.objects.len();
                for (k, v) in o {
                    res.insert(k.clone(), self.eval(v));
                }
                self.objects.insert(id, res);
                Value::Object(id)
            },
            Expr::List(exprs) => {
                let mut res = Vec::new();
                let id = self.lists.len();
                for e in exprs {
                    res.push(self.eval(e));
                }
                self.lists.insert(id, res);
                Value::List(id)
            },
            Expr::Function { args, value, closure, reflection } => {
                let id = self.functions.len();
                let mut closed = HashMap::<String, Value>::new();
                for name in closure {
                    closed.insert(name.clone(), self.eval(&Expr::Access(Target::Var(name.clone()))));
                }
                self.functions.insert(id, Function {
                    args: args.to_vec(),
                    value: *value.clone(),
                    closure: closed,
                    reflection: *reflection
                });
                Value::Function(id)
            },


            Expr::If { cond, value, else_value } => {
                if self.eval(cond).to_bool(self) {
                    self.eval(value)
                } else {
                    self.eval(else_value)
                }
            },
            Expr::While { cond, value, else_value } => {
                let mut res = Value::Nil;
                let mut iter_once = false;
                while self.eval(cond).to_bool(self) {
                    res = self.eval(value);
                    iter_once = true;
                }
                if !iter_once {
                    res = self.eval(else_value);
                }
                res
            },
            Expr::Binding { target, value } => {
                let v = self.eval(value);
                self.set_val(target.clone(), v.clone());
                v
            },
            Expr::Chain(exprs) => {
                let mut res = Value::Nil;
                for e in exprs {
                    res = self.eval(e);
                }
                res
            },
            Expr::Access(t) => {
                self.get_val(t.clone())
            },
            Expr::FuncCall { func, args: args_exprs } => {
                let mut args_values = Vec::new();
                for e in args_exprs {
                    args_values.push(self.eval(e));
                }
                if let Expr::Access(Target::Field { var, field }) = &**func {
                    match self.eval(&*var) {
                        Value::Object(id) => {
                            let o = self.get_object(id);
                            if o.contains_key(field) {
                                self.call_func(o.get(field).unwrap().clone(), &args_values, Some(Value::Object(id)))
                            } else {
                                println!("Cannot find field {field}");
                                Value::Nil
                            }
                        },
                        Value::NativeObject(o) => {
                            self.call_func(o.as_ref().borrow_mut().get(&field), &args_values, None)
                        },
                        _ => {
                            println!("Trying to access a field from a non-object value");
                            Value::Nil
                        }
                    }
                } else {
                    let f = self.eval(func);
                    self.call_func(f, &args_values, None)
                }
            },
        }
    }


    pub fn call_func(&mut self, f: Value, args: &Vec<Value>, refl: Option<Value>) -> Value {
        match f {
            Value::Function(id) => {
                let f = self.get_func(id);
                let Function { args: args_names, value, closure, reflection } = f.clone();

                self.scopes.push(HashMap::new());
                for (k, v) in closure {
                    self.set_val(Target::Var(k.to_string()), v.clone());
                }
                for i in 0..args_names.len() {
                    if args.len() > i {
                        self.set_val(Target::Var(args_names[i].clone()), args[i].clone());
                    } else {
                        self.set_val(Target::Var(args_names[i].clone()), Value::Nil);
                    }
                }
                if reflection && refl.is_some() {
                    self.set_val(Target::Var("self".to_string()), refl.unwrap());
                }
                let res = self.eval(&value);
                self.scopes.pop();
                res
            },
            Value::NativeFunction(f) => {
                let mut func = f.as_ref().borrow_mut();
                func(self, &args)
            },
            _ => {
                println!("Cannot call non-function value");
                Value::Nil
            }
        }
    }


    pub fn import(r: &mut Runner, args: &[Value]) -> Value {
        if args.len() < 1 {
            println!("Invalid arguments");
            return Value::Nil;
        }

        if let Value::Str(id) = &args[0] {
            let name = r.get_str(*id);
            if r.loadables.contains_key(name) {
                r.loadables.get(name).unwrap()(r)
            } else if let Ok(true) = fs::exists(name) && let Ok(src) = fs::read_to_string(name) {
                let mut p = Parser::new(&src);
                r.eval(&p.expr())
            } else {
                println!("Unable to import \"{name}\": not found in modules, and path does not exist");
                Value::Nil
            }
        } else {
            println!("Invalid arguments");
            Value::Nil
        }
    }
}
