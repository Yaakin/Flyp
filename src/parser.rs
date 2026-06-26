use std::collections::HashMap;

#[derive(Clone, Debug)]
pub enum Target {
    Var(String),
    Field{
        var: Box<Expr>,
        field: String,
    },
    Index{
        var: Box<Expr>,
        index: Box<Expr>,
    }
}

#[derive(Clone, Debug)]
pub enum Expr {
    Nil,
    Number(f64),
    Str(String),
    Function {
        args: Vec<String>,
        value: Box<Expr>,
    },
    Object(HashMap<String, Expr>),
    List(Vec<Expr>),

    Identifier(String),
    Access(Target),

    FuncCall {
        func: Box<Expr>,
        args: Vec<Expr>,
    },

    If {
        cond: Box<Expr>,
        value: Box<Expr>,
        else_value: Box<Expr>,
    },
    While {
        cond: Box<Expr>,
        value: Box<Expr>,
        else_value: Box<Expr>,
    },
    Binding {
        target: Target,
        value: Box<Expr>,
    },
    Chain(Vec<Expr>),
}

#[derive(Copy, Clone, Debug)]
pub struct ParserState {
    pub index: usize,
}

#[derive(Debug)]
pub struct Parser<'a> {
    src: &'a str,
    pub state: ParserState,
}

impl<'a> Parser<'a> {
    pub fn new(src: &'a str) -> Self {
        Self {
            src: src,
            state: ParserState {
                index: 0,
            },
        }
    }

    fn peek_char(&self) -> Option<char> {
        self.src.chars().nth(self.state.index)
    }
    
    fn next_char(&mut self) -> char {
        let res = self.peek_char().expect("Expected a character, found end of file");
        self.state.index += 1;
        res
    }

    fn trim(&mut self) {
        while let Some(c) = self.peek_char() {
            match c {
                ' ' | '\t' | '\n' => { self.next_char(); },
                '#' => {
                    self.next_char();
                    while self.next_char() != '#' {}
                },
                '/' => {
                    if let Some(c) = self.src.chars().nth(self.state.index+1) && c == '/' {
                        while self.next_char() != '\n' {}
                    } else {
                        break;
                    }
                }
                _ => break,
            }
        }
    }

    fn num(&mut self) -> (Option<f64>, ParserState) {
        let reset = *&self.state;
        let mut res = 0.;
        let mut decimal_part = false;
        let negative = if let Some(c) = self.peek_char() {
            match c {
                '-' => {
                    self.next_char();
                    true
                },
                '0'..='9' => {
                    false
                },
                _ => {
                    self.state = reset;
                    return (None, reset);
                }
            }
        } else {
            self.state = reset;
            return (None, reset);
        };
        while let Some(c) = self.peek_char() {
            match c {
                '0'..='9' => {
                    res = res * 10. + (c as u32 - '0' as u32) as f64;
                    self.next_char();
                },
                '.' => {
                    self.next_char();
                    decimal_part = true;
                    break
                },
                _ => break,
            }
        }
        if decimal_part {
            let mut power_of_10 = 0.1;
            while let Some(c) = self.peek_char() {
                match c {
                    '0'..='9' => {
                        res += power_of_10 * (c as u32 - '0' as u32) as f64;
                        power_of_10 /= 10.0;
                        self.next_char();
                    },
                    _ => break,
                }
            }
        }
        if negative {
            res *= -1.;
        }
        (Some(res), reset)
    }

    fn str(&mut self) -> (Option<String>, ParserState) {
        let reset = *&self.state;
        if let (Some(_), _) = self.litteral("\"") {
            let mut res = String::new();
            
            while let Some(c) = self.peek_char() {
                match c {
                    '"' => {
                        self.next_char();
                        return (Some(res), reset);
                    },
                    _ => {
                        res.push(c);
                        self.next_char();
                    }
                }
            }
            println!("Unterminated string");
            (None, reset)
        } else {
            println!("Expected '\"' to start a string");
            (None, reset)
        }
    }

    fn identifier(&mut self) -> (Option<String>, ParserState) {
        let reset = *&self.state;
        self.trim();
        let mut res = String::new();
        while let Some(c) = self.peek_char() {
            match c {
                'a'..='z' | 'A'..='Z' | '_' | '0'..='9' => {
                    res.push(c);
                    self.next_char();
                },
                _ => break,
            }
        }
        if res.len() > 0 {
            (Some(res), reset)
        } else {
            (None, reset)
        }
    }

    fn litteral(&mut self, expected: &str) -> (Option<String>, ParserState) {
        let reset = *&self.state;
        self.trim();
        for i in 0..expected.len() {
            if expected.chars().nth(i) != self.src.chars().nth(self.state.index + i) {
                return (None, reset);
            }
        }
        self.state.index += expected.len();
        (Some(expected.to_string()), reset)
    }

    fn modifiers(&mut self, initial: Expr) -> (bool, Expr) {
        if let (Some(_), _) = self.litteral(".") {
            if let (Some(attr), _) = self.identifier() {
                (true, Expr::Access(Target::Field{
                    var: Box::new(initial),
                    field: attr,
                }))
            } else {
                println!("Expected a field identifier");
                (false, Expr::Nil)
            }
        } else if let (Some(_), r) = self.litteral("(") {
            let mut args = Vec::new();
            while let (None, _) = self.litteral(")") {
                args.push(self.expr());
                self.trim();
                match self.peek_char() {
                    Some(',') => { self.next_char(); },
                    Some(')') => {
                        self.next_char();
                        break;
                    },
                    Some(c) => {
                        println!("Unexpected character '{}' when ',' was expected", c);
                        self.state = r;
                        return (false, Expr::Nil);
                    },
                    None => {
                        println!("Unterminated argument definition");
                        self.state = r;
                        return (false, Expr::Nil);
                    }
                }
            }

            (true, Expr::FuncCall{
                func: Box::new(initial),
                args: args,
            })
        } else if let (Some(_), _) = self.litteral("<") {
            let index = self.expr();
            if let (Some(_), _) = self.litteral(">") {
                (true, Expr::Access(Target::Index{
                    var: Box::new(initial),
                    index: Box::new(index),
                }))
            } else {
                println!("Unterminated index accessor");
                (false, Expr::Nil)
            }
        } else {
            (false, initial)
        }
    }

    fn target(&mut self) -> Option<Expr> {
        if let (Some(base), _) = self.identifier() {
            let mut found;
            let mut e = Expr::Identifier(base);
            (found, e) = self.modifiers(e);
            while found {
                (found, e) = self.modifiers(e);
            }
            Some(e)
        } else {
            println!("Expected identifier");
            None
        }
    }

    pub fn expr(&mut self) -> Expr {
        self.trim();
        let mut e = if let Some('"') = self.peek_char() {
            if let (Some(content), _) = self.str() {
                Expr::Str(content)
            } else {
                Expr::Nil
            }
        } else if let (Some(_), r) = self.litteral("let") {
            let t = self.target();
            if let Some(Expr::Access(target)) = t {
                if let (Some(_), _) = self.litteral("=") {
                    Expr::Binding {
                        target: target,
                        value: Box::new(self.expr()),
                    }
                } else {
                    println!("Expected = for binding");
                    self.state = r;
                    Expr::Nil
                }
            } else if let Some(Expr::Identifier(name)) = t {
                if let (Some(_), _) = self.litteral("=") {
                    Expr::Binding {
                        target: Target::Var(name),
                        value: Box::new(self.expr()),
                    }
                } else {
                    println!("Expected = for binding");
                    self.state = r;
                    Expr::Nil
                }

            } else {
                println!("Expected an identifier for binding, got: {t:?}");
                self.state = r;
                Expr::Nil
            }
        } else if let (Some(_), r) = self.litteral("fn") {
            if let (Some(_), _) = self.litteral("(") {
                let mut args = Vec::new();
                while let (None, _) = self.litteral(")") {
                    if let (Some(a), _) = self.identifier() {
                        args.push(a);
                    } else {
                        println!("Expected argument for function definition");
                        self.state = r;
                        return Expr::Nil;
                    }
                    self.trim();
                    match self.peek_char() {
                        Some(',') => { self.next_char(); },
                        Some(')') => {
                            self.next_char();
                            break;
                        },
                        Some(c) => {
                            println!("Unexpected character '{}' when ',' was expected", c);
                            self.state = r;
                            return Expr::Nil;
                        },
                        None => {
                            println!("Unterminated argument definition");
                            self.state = r;
                            return Expr::Nil;
                        }
                    }
                }

                Expr::Function{
                    args: args,
                    value: Box::new(self.expr()),
                }
            } else {
                println!("Expected '(' for argument definition");
                self.state = r;
                Expr::Nil
            }
        } else if let (Some(_), r) = self.litteral("if") {
            if let (Some(_), _) = self.litteral("(") {
                let cond = self.expr();
                if let (Some(_), _) = self.litteral(")") {
                    let value = self.expr();
                    let mut else_value = Expr::Nil;

                    if let (Some(_), _) = self.litteral("else") {
                        else_value = self.expr();
                    }

                    Expr::If{
                        cond: Box::new(cond),
                        value: Box::new(value),
                        else_value: Box::new(else_value),
                    }
                } else {
                    println!("Expected '(' before a condition");
                    self.state = r;
                    Expr::Nil
                }
            } else {
                println!("Expected '(' before a condition");
                self.state = r;
                Expr::Nil
            }
        } else if let (Some(_), r) = self.litteral("while") {
            if let (Some(_), _) = self.litteral("(") {
                let cond = self.expr();
                if let (Some(_), _) = self.litteral(")") {
                    let value = self.expr();
                    let mut else_value = Expr::Nil;

                    if let (Some(_), _) = self.litteral("else") {
                        else_value = self.expr();
                    }

                    Expr::While{
                        cond: Box::new(cond),
                        value: Box::new(value),
                        else_value: Box::new(else_value),
                    }
                } else {
                    println!("Expected '(' before a condition");
                    self.state = r;
                    Expr::Nil
                }
            } else {
                println!("Expected '(' before a condition");
                self.state = r;
                Expr::Nil
            }
        } else if let (Some(_), r) = self.litteral("{") {
            let mut exprs = Vec::new();

            while let (None, _) = self.litteral("}") {
                exprs.push(self.expr());
                self.trim();
                match self.peek_char() {
                    Some(';') => { self.next_char(); },
                    Some('}') => {
                        self.next_char();
                        break;
                    },
                    Some(c) => {
                        println!("Unexpected character '{}' when ';' was expected", c);
                        self.state = r;
                        return Expr::Nil;
                    },
                    None => {
                        println!("Unterminated chain expression");
                        self.state = r;
                        return Expr::Nil;
                    }
                }
            }
            Expr::Chain(exprs)
        } else if let (Some(_), r) = self.litteral("<") {
            let mut exprs = Vec::new();

            while let (None, _) = self.litteral(">") {
                exprs.push(self.expr());
                self.trim();
                match self.peek_char() {
                    Some(';') => { self.next_char(); },
                    Some('>') => {
                        self.next_char();
                        break;
                    },
                    Some(c) => {
                        println!("Unexpected character '{}' when ';' was expected", c);
                        self.state = r;
                        return Expr::Nil;
                    },
                    None => {
                        println!("Unterminated list expression");
                        self.state = r;
                        return Expr::Nil;
                    }
                }
            }
            Expr::List(exprs)
        } else if let (Some(_), r) = self.litteral("[") {
            let mut fields = HashMap::new();

            while let (None, _) = self.litteral("]") {
                if let (Some(name), _) = self.identifier() {
                    if let (None, _) = self.litteral(":") {
                        println!("Expected ':' after field name");
                        self.state = r;
                        return Expr::Nil;
                    }

                    fields.insert(name, self.expr());
                    self.trim();
                    match self.peek_char() {
                        Some(';') => { self.next_char(); },
                        Some(']') => {
                            self.next_char();
                            break;
                        },
                        Some(c) => {
                            println!("Unexpected character '{}' when ';' was expected", c);
                            self.state = r;
                            return Expr::Nil;
                        },
                        None => {
                            println!("Unterminated chain expression");
                            self.state = r;
                            return Expr::Nil;
                        }
                    }
                } else {
                    println!("Expected a field name");
                    self.state = r;
                    return Expr::Nil;
                }
            }
            Expr::Object(fields)
        } else if let (Some(x), _) = self.num() {
            Expr::Number(x)
        } else if let (Some(name), _) = self.identifier() {
            Expr::Access(Target::Var(name))
        } else if let None = self.peek_char() {
            Expr::Nil
        } else {
            println!("{:?} at {}", self.peek_char(), self.state.index);
            println!("Could not parse expression");
            Expr::Nil
        };

        let mut found;
        (found, e) = self.modifiers(e);
        while found {
            (found, e) = self.modifiers(e);
        }

        e
    }
}
