use std::collections::HashMap;

use crate::modem::{self, NestedList};
use crate::syntax::{Stmt, Token, Var};

#[derive(Debug, Default)]
pub struct State {
    vars: HashMap<Var, Value>,
}

#[derive(Debug, PartialEq, Clone)]
pub enum Value {
    Var(Var),
    Number(i64),
    Signal(Vec<bool>), // used with modulate / demodulate
    Picture(Picture),
    BuiltIn(BuiltIn),
    Apply(Box<Value>, Box<Value>),
    Partial0(PartialAp, Box<Value>),
    Partial1(PartialAp, Box<Value>, Box<Value>),
}

#[derive(Debug, PartialEq, Clone)]
pub struct Point {
    x: u32,
    y: u32,
}

#[derive(Debug, Default, PartialEq, Clone)]
pub struct Picture {
    width: u32,
    height: u32,
    points: Vec<Point>,
}

impl Picture {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn add(&mut self, x: u32, y: u32) {
        // TODO: maybe calculate these later if slow
        if x >= self.width {
            self.width = x + 1;
        }
        if y >= self.height {
            self.height = y + 1;
        }
        self.points.push(Point { x, y });
    }
}

impl std::fmt::Display for Picture {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        // SLOW - TODO REWRITE
        for y in 0..self.height {
            for x in 0..self.width {
                if self.points.contains(&Point { x, y }) {
                    write!(f, "{}", "#")?;
                } else {
                    write!(f, "{}", " ")?;
                }
            }
            if y != self.height - 1 {
                writeln!(f)?;
            }
        }
        Ok(())
    }
}

// Built-in functions except `ap`
#[derive(Debug, PartialEq, Clone)]
pub enum BuiltIn {
    Inc,           // #5
    Dec,           // #6
    Add,           // #7
    Mul,           // #9
    Div,           // #10
    Eq,            // #11
    Lt,            // #12
    Mod,           // #13 - ???
    Dem,           // #14 - ???
    Send,          // #15 - ???
    Neg,           // #16
    S,             // #18
    C,             // #19
    B,             // #20
    True,          // #21
    False,         // #22
    Pwr2,          // #23 - ???
    I,             // #24
    Cons,          // #25
    Head,          // #26
    Tail,          // #27
    Nil,           // #28
    IsNil,         // #29
    Draw,          // #32
    Checkerboard,  // #33
    MultiDraw,     // #34
    ModList,       // #35 - ???
    Send2,         // #36 - ???
    If0,           // #37
    Interact,      // #38-39 - ???
    StatelessDraw, // #40 - ???
    StatefulDraw,  // #41 - ???
    Galaxy,        // #42
}

#[derive(Debug, PartialEq, Clone, Copy)]
pub enum PartialAp {
    Add_0,
    Mul_0,
    Div_0,
    Eq_0,
    Lt_0,
    S_1,
    S_0,
    C_1,
    C_0,
    B_1,
    B_0,
    True_0,
    False_0,
    Cons_0,
    Cons_1,
    If0_0,
    If0_1,
}

impl State {
    pub fn new() -> Self {
        State::default()
    }

    pub fn eval(&mut self, var: Var) -> Value {
        let v = self.vars.get(&var).unwrap().clone();
        self.eval_value(v)
    }

    pub fn eval_value(&mut self, val: Value) -> Value {
        // println!("eval_value: {:?}", val);
        match val {
            Value::Var(v) => self.eval_value(self.vars.get(&v).unwrap().clone()),
            Value::Number(_) => val,
            Value::Signal(_) => val,
            Value::Picture(_) => val,
            Value::BuiltIn(_) => val,
            Value::Apply(f, arg) => {
                let e_f = self.eval_value(*f);
                match e_f {
                    Value::BuiltIn(BuiltIn::Inc) => {
                        if let Value::Number(n) = self.eval_value(*arg) {
                            Value::Number(n + 1)
                        } else {
                            panic!("Invalid argument for `inc`");
                        }
                    }
                    Value::BuiltIn(BuiltIn::Dec) => {
                        if let Value::Number(n) = self.eval_value(*arg) {
                            Value::Number(n - 1)
                        } else {
                            panic!("Invalid argument for `dec`");
                        }
                    }
                    Value::BuiltIn(BuiltIn::Add) => Value::Partial0(PartialAp::Add_0, arg),
                    Value::Partial0(PartialAp::Add_0, arg0) => {
                        if let Value::Number(b) = self.eval_value(*arg) {
                            if let Value::Number(a) = self.eval_value(*arg0) {
                                Value::Number(a + b)
                            } else {
                                panic!("Invalid argument for `add`");
                            }
                        } else {
                            panic!("Invalid argument for `add`");
                        }
                    }
                    Value::BuiltIn(BuiltIn::Mul) => Value::Partial0(PartialAp::Mul_0, arg),
                    Value::Partial0(PartialAp::Mul_0, arg0) => {
                        if let Value::Number(b) = self.eval_value(*arg) {
                            if let Value::Number(a) = self.eval_value(*arg0) {
                                Value::Number(a * b)
                            } else {
                                panic!("Invalid argument for `mul`");
                            }
                        } else {
                            panic!("Invalid argument for `mul`");
                        }
                    }
                    Value::BuiltIn(BuiltIn::Div) => Value::Partial0(PartialAp::Div_0, arg),
                    Value::Partial0(PartialAp::Div_0, arg0) => {
                        if let Value::Number(b) = self.eval_value(*arg) {
                            if let Value::Number(a) = self.eval_value(*arg0) {
                                Value::Number(a / b)
                            } else {
                                panic!("Invalid argument for `div`");
                            }
                        } else {
                            panic!("Invalid argument for `div`");
                        }
                    }
                    Value::BuiltIn(BuiltIn::Eq) => Value::Partial0(PartialAp::Eq_0, arg),
                    Value::Partial0(PartialAp::Eq_0, arg0) => {
                        if let Value::Number(b) = self.eval_value(*arg) {
                            if let Value::Number(a) = self.eval_value(*arg0) {
                                if a == b {
                                    Value::BuiltIn(BuiltIn::True)
                                } else {
                                    Value::BuiltIn(BuiltIn::False)
                                }
                            } else {
                                panic!("Invalid argument for `eq`");
                            }
                        } else {
                            panic!("Invalid argument for `eq`");
                        }
                    }
                    Value::BuiltIn(BuiltIn::Lt) => Value::Partial0(PartialAp::Lt_0, arg),
                    Value::Partial0(PartialAp::Lt_0, arg0) => {
                        if let Value::Number(b) = self.eval_value(*arg) {
                            if let Value::Number(a) = self.eval_value(*arg0) {
                                if a < b {
                                    Value::BuiltIn(BuiltIn::True)
                                } else {
                                    Value::BuiltIn(BuiltIn::False)
                                }
                            } else {
                                panic!("Invalid argument for `lt`");
                            }
                        } else {
                            panic!("Invalid argument for `lt`");
                        }
                    }
                    Value::BuiltIn(BuiltIn::Mod) => {
                        match self.eval_value(*arg) {
                            Value::Number(n) => Value::Signal(modem::mod_num(n)),
                            // TODO: lists
                            _ => panic!("Invalid argument for `mod`"),
                        }
                    }
                    Value::BuiltIn(BuiltIn::Dem) => {
                        if let Value::Signal(s) = self.eval_value(*arg) {
                            let list = modem::dem_list(&s);
                            panic!("TODO: demodulate {:?}", list);
                        } else {
                            panic!("Invalid argument for `dem`");
                        }
                    }
                    Value::BuiltIn(BuiltIn::Neg) => {
                        if let Value::Number(n) = self.eval_value(*arg) {
                            Value::Number(-n)
                        } else {
                            panic!("Invalid argument for `neg`");
                        }
                    }
                    Value::BuiltIn(BuiltIn::S) => Value::Partial0(PartialAp::S_0, arg),
                    Value::Partial0(PartialAp::S_0, arg0) => {
                        Value::Partial1(PartialAp::S_1, arg0, arg)
                    }
                    Value::Partial1(PartialAp::S_1, arg0, arg1) => {
                        self.eval_value(Value::Apply(
                            Box::new(Value::Apply(arg0, arg.clone())), // If costly, use Rc
                            Box::new(Value::Apply(arg1, arg)),
                        ))
                    }
                    Value::BuiltIn(BuiltIn::C) => Value::Partial0(PartialAp::C_0, arg),
                    Value::Partial0(PartialAp::C_0, arg0) => {
                        Value::Partial1(PartialAp::C_1, arg0, arg)
                    }
                    Value::Partial1(PartialAp::C_1, arg0, arg1) => {
                        self.eval_value(Value::Apply(Box::new(Value::Apply(arg0, arg)), arg1))
                    }
                    Value::BuiltIn(BuiltIn::B) => Value::Partial0(PartialAp::B_0, arg),
                    Value::Partial0(PartialAp::B_0, arg0) => {
                        Value::Partial1(PartialAp::B_1, arg0, arg)
                    }
                    Value::Partial1(PartialAp::B_1, arg0, arg1) => {
                        self.eval_value(Value::Apply(arg0, Box::new(Value::Apply(arg1, arg))))
                    }
                    Value::BuiltIn(BuiltIn::True) => Value::Partial0(PartialAp::True_0, arg),
                    Value::Partial0(PartialAp::True_0, arg0) => self.eval_value(*arg0),
                    Value::BuiltIn(BuiltIn::False) => Value::Partial0(PartialAp::False_0, arg),
                    Value::Partial0(PartialAp::False_0, _) => self.eval_value(*arg),
                    Value::BuiltIn(BuiltIn::Pwr2) => {
                        if let Value::Number(n) = self.eval_value(*arg) {
                            Value::Number((2 as i64).pow(n as u32))
                        } else {
                            panic!("Invalid argument for `pwr2`");
                        }
                    }
                    Value::BuiltIn(BuiltIn::I) => self.eval_value(*arg),
                    Value::BuiltIn(BuiltIn::Cons) => Value::Partial0(PartialAp::Cons_0, arg),
                    Value::Partial0(PartialAp::Cons_0, arg0) => {
                        Value::Partial1(PartialAp::Cons_1, arg0, arg)
                    }
                    Value::Partial1(PartialAp::Cons_1, arg0, arg1) => {
                        self.eval_value(Value::Apply(Box::new(Value::Apply(arg, arg0)), arg1))
                    }
                    Value::BuiltIn(BuiltIn::Head) => {
                        self.eval_value(Value::Apply(arg, Box::new(Value::BuiltIn(BuiltIn::True))))
                    }
                    Value::BuiltIn(BuiltIn::Tail) => {
                        self.eval_value(Value::Apply(arg, Box::new(Value::BuiltIn(BuiltIn::False))))
                    }
                    Value::BuiltIn(BuiltIn::Nil) => Value::BuiltIn(BuiltIn::True),
                    Value::BuiltIn(BuiltIn::IsNil) => {
                        if let Value::BuiltIn(BuiltIn::Nil) = *arg {
                            Value::BuiltIn(BuiltIn::True)
                        } else {
                            Value::BuiltIn(BuiltIn::False)
                        }
                    }
                    Value::BuiltIn(BuiltIn::Draw) => Value::Picture(self.eval_draw(*arg)),
                    Value::BuiltIn(BuiltIn::MultiDraw) => self.eval_multidraw(*arg),
                    Value::BuiltIn(BuiltIn::If0) => Value::Partial0(PartialAp::If0_0, arg),
                    Value::Partial0(PartialAp::If0_0, arg0) => {
                        Value::Partial1(PartialAp::If0_1, arg0, arg)
                    }
                    Value::Partial1(PartialAp::If0_1, arg0, arg1) => {
                        if let Value::Number(0) = self.eval_value(*arg0) {
                            self.eval_value(*arg1)
                        } else {
                            self.eval_value(*arg)
                        }
                    }
                    f => panic!("!{:?}", f),
                }
            }
            Value::Partial0(_, _) => panic!(),
            Value::Partial1(_, _, _) => panic!(),
        }
    }

    fn eval_multidraw(&self, val: Value) -> Value {
        if let Value::Apply(f0, arg0) = val {
            if let Value::Apply(f1, arg1) = *f0 {
                if let Value::BuiltIn(BuiltIn::Cons) = *f1 {
                    return Value::Apply(
                        Box::new(Value::Apply(
                            Box::new(Value::BuiltIn(BuiltIn::Cons)),
                            Box::new(Value::Picture(self.eval_draw(*arg1))),
                        )),
                        Box::new(self.eval_multidraw(*arg0)),
                    );
                }
            }
        } else if Value::BuiltIn(BuiltIn::Nil) == val {
            return val;
        }
        panic!("Invalid multidraw argument")
    }

    fn eval_draw(&self, val: Value) -> Picture {
        let mut picture = Picture::new();
        let mut list = self.eval_nested_list(val);
        loop {
            // we expect a list of pairs here
            match list {
                NestedList::Nil => break,
                NestedList::Cons(head, tail) => {
                    match *head {
                        NestedList::Cons(x, y) => {
                            if let NestedList::Number(x) = *x {
                                if let NestedList::Number(y) = *y {
                                    picture.add(x as u32, y as u32);
                                } else {
                                    panic!("Invalid list")
                                }
                            } else {
                                panic!("Invalid list")
                            }
                        }
                        _ => panic!("Invalid list"),
                    }
                    list = *tail;
                }
                _ => panic!("Invalid list"),
            }
        }
        picture
    }

    fn eval_nested_list(&self, val: Value) -> NestedList {
        match val {
            Value::Apply(f0, arg0) => {
                if let Value::Apply(f1, arg1) = *f0 {
                    if let Value::BuiltIn(BuiltIn::Cons) = *f1 {
                        NestedList::Cons(
                            Box::new(self.eval_nested_list(*arg1)),
                            Box::new(self.eval_nested_list(*arg0)),
                        )
                    } else {
                        panic!("Invalid list format")
                    }
                } else {
                    panic!("Invalid list format")
                }
            }
            Value::BuiltIn(BuiltIn::Nil) => NestedList::Nil,
            Value::Number(n) => NestedList::Number(n),
            _ => panic!("Invalid value in eval_list: {:?}", val),
        }
    }

    pub fn interpret(&mut self, stmt: Stmt) {
        // println!("Compiling {:?}", stmt.var);
        // println!("Raw: {:?}", stmt.code);
        let v = self.compile(stmt.code);
        // println!("Compiled: {:?}", v);
        self.vars.insert(stmt.var, v);
    }

    fn compile(&self, code: Vec<Token>) -> Value {
        let mut stack: Vec<Value> = vec![];
        for token in code.into_iter().rev() {
            match token {
                Token::Var(v) => stack.push(Value::Var(v)),

                Token::Number(n) => stack.push(Value::Number(n)),
                Token::True => stack.push(Value::BuiltIn(BuiltIn::True)),
                Token::False => stack.push(Value::BuiltIn(BuiltIn::False)),
                Token::Nil => stack.push(Value::BuiltIn(BuiltIn::Nil)),

                Token::Inc => stack.push(Value::BuiltIn(BuiltIn::Inc)),
                Token::Dec => stack.push(Value::BuiltIn(BuiltIn::Dec)),
                Token::Add => stack.push(Value::BuiltIn(BuiltIn::Add)),
                Token::Mul => stack.push(Value::BuiltIn(BuiltIn::Mul)),
                Token::Div => stack.push(Value::BuiltIn(BuiltIn::Div)),
                Token::Eq => stack.push(Value::BuiltIn(BuiltIn::Eq)),
                Token::Lt => stack.push(Value::BuiltIn(BuiltIn::Lt)),
                Token::Mod => stack.push(Value::BuiltIn(BuiltIn::Mod)),
                Token::Dem => stack.push(Value::BuiltIn(BuiltIn::Dem)),
                Token::Send => stack.push(Value::BuiltIn(BuiltIn::Send)),
                Token::Neg => stack.push(Value::BuiltIn(BuiltIn::Neg)),
                Token::S => stack.push(Value::BuiltIn(BuiltIn::S)),
                Token::C => stack.push(Value::BuiltIn(BuiltIn::C)),
                Token::B => stack.push(Value::BuiltIn(BuiltIn::B)),
                Token::Pwr2 => stack.push(Value::BuiltIn(BuiltIn::Pwr2)),
                Token::I => stack.push(Value::BuiltIn(BuiltIn::I)),
                Token::Cons => stack.push(Value::BuiltIn(BuiltIn::Cons)),
                Token::Head => stack.push(Value::BuiltIn(BuiltIn::Head)),
                Token::Tail => stack.push(Value::BuiltIn(BuiltIn::Tail)),
                Token::IsNil => stack.push(Value::BuiltIn(BuiltIn::IsNil)),
                Token::Draw => stack.push(Value::BuiltIn(BuiltIn::Draw)),
                Token::Checkerboard => stack.push(Value::BuiltIn(BuiltIn::Checkerboard)),
                Token::MultiDraw => stack.push(Value::BuiltIn(BuiltIn::MultiDraw)),
                Token::ModList => stack.push(Value::BuiltIn(BuiltIn::ModList)),
                Token::Send2 => stack.push(Value::BuiltIn(BuiltIn::Send2)),
                Token::If0 => stack.push(Value::BuiltIn(BuiltIn::If0)),
                Token::Interact => stack.push(Value::BuiltIn(BuiltIn::Interact)),
                Token::StatelessDraw => stack.push(Value::BuiltIn(BuiltIn::StatelessDraw)),
                Token::StatefulDraw => stack.push(Value::BuiltIn(BuiltIn::StatefulDraw)),
                Token::Galaxy => stack.push(Value::BuiltIn(BuiltIn::Galaxy)),

                Token::Ap => {
                    let x = stack.pop().unwrap();
                    let v = stack.pop().unwrap();
                    stack.push(Value::Apply(Box::new(x), Box::new(v)));
                }
            }
        }
        assert!(stack.len() == 1);
        stack[0].clone()
    }
}
