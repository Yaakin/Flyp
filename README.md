# What is flyp ?

Flyp is a homemade interpreted language that I made mainly for educational purposes, but I aim to use it in some other projects of mine once it is stable.
Be warned that the language is severly lacking a mathematical expression parser for now, as well as a standard library. 
The latter will be growing with time, but it is pretty easy to create a module tailored for your needs if you want

# Syntax

In flyp, everything is an expression. When a file is executed, the first expression is evaluated, and everything else is considered as a comment.

## Primary expressions

Primary expressions are basic values that are used to build more complex expressions. They can be of different types:
- Nil: `nil`
- Bool: `true` or `false`
- Number: `0`, `3.14`, `-1.382374`, ...
- String: `"hello, world"` (which does not handle special characters like `"\n"` so far)

## Litterals

You can then build expressions from these:
- Binding: `let x = val`, stores a variable in the current scope (or globals if it already exists) and is evaluated as `val`
- Identifier: `x`, refers to a variable in the current (or global) scope, defined by a binding
- Lists: `<1; 2; 3; ... >`, are indexed with `your_list<index>`
- Objects: `[ name: "Yaakin"; score: 10; ... ]`, are indexed with `your_obj.name`
- Functions: `fn(arg1, arg2, ...) return_value`, can be called with `func_name(arg1, arg2, ...)`

## Control Flow

- If: `if (cond) val`, accepts an `else val` part which is evaluated if `cond` is false
- While: `while (cond) val`, can be suffixed with an `else val` which is executed if no iteration occurs
- Chains: `{expr1; expr2; ...; expr_final}`, expressions are evaluated in order and the whole chain value is of expr_final

# Building and running

To build the project:
```sh
git clone https://github.com/Yaakin/flyp
cd flyp
cargo build```

To run a file, `cargo run -- run your_file.flyp`
There is also a repl, but it is quite unfinished and I probably won't work on it anytime soon

Finally, flyp can be used as a library, using this in your `Cargo.toml`:
```toml
flyp = { version = "0.1.0", path = "../flyp" }
```
Then with a minimal example:
```rs
use std::fs;

fn main() {
    let filename = "./src.flyp";

    let mut r = flyp::Runner::new();
    if let Ok(src) = fs::read_to_string(filename) {
        let mut p = flyp::Parser::new(&src);
        r.eval(&p.expr());
    } else {
        println!("Cant read input file {}", filename);
    }
}
