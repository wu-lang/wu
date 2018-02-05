use super::value::*;
use super::compiler::*;

use std::mem;
use std::ptr;

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Instruction {
    Add,
    Sub,
    Mul,
    Mod,
    Div,
    
    Neg,
    
    Lt,
    LtEqual,
    Gt,
    GtEqual,
    Equal,
    NEqual,
    
    LoadConst(u16),
    LoadLocal(u16),
    StoreLocal(u16),
    
    BranchTrue(i16),
    BranchFalse(i16),
    Jump(i16),
    
    Pop,
    Return,
    Print,

    Call(u8),
}

pub struct CallInfo<'c> {
    locals: Box<[Value<'c>]>,
    pc: usize,
    func: *const Block<'c>,
}

pub struct Machine<'m> {
    value_stack: Vec<Value<'m>>,
    next_object: *mut Object<'m>,
    call_stack:  Vec<CallInfo<'m>>,
}

impl<'m> Machine<'m> {
    pub fn new() -> Self {
        Machine {
            value_stack: Vec::new(),
            next_object: ptr::null_mut(),
            call_stack:  Vec::new(),
        }
    }

    pub fn execute(&mut self, initial: *const Block<'m>) {
        use self::Instruction::*;
        use self::Value::*;

        let mut pointer: usize = 0;
        let mut func: &Block   = unsafe {
            &*initial
        };

        macro_rules! match_binop {
            ($($pat:pat => $block:block)+) => {{
                let _a = self.value_stack.pop().unwrap();
                let _b = self.value_stack.pop().unwrap();
                let _result = match (_b, _a) {
                    $($pat => $block)+,
                    _      => unreachable!()
                };
                self.value_stack.push(_result);
            }}
        }

        let mut locals = vec![Nil; func.local_names.len()].into_boxed_slice();

        loop {
            let op = func.code[pointer];

            match op {
                LoadConst(index)  => self.value_stack.push(func.constants[index as usize].clone()),
                LoadLocal(index)  => self.value_stack.push(locals[index as usize].clone()),
                StoreLocal(index) => locals[index as usize] = self.value_stack.pop().unwrap(),

                BranchTrue(diff) => {
                    if self.value_stack.pop().unwrap() == Bool(true) {
                        pointer = pointer.wrapping_add((diff as isize) as usize)
                    } else {
                        pointer = pointer.wrapping_add(1)
                    }

                    continue
                }

                BranchFalse(diff) => {
                    if self.value_stack.pop().unwrap() != Bool(true) {
                        pointer = pointer.wrapping_add((diff as isize) as usize)
                    } else {
                        pointer = pointer.wrapping_add(1)
                    }

                    continue
                },

                Jump(diff) => {
                    pointer = pointer.wrapping_add((diff as isize) as usize);

                    continue
                },

                Pop => { self.value_stack.pop().unwrap(); },

                Add => match_binop! {
                    (Int(a), Int(b)) => { Int(a + b) }
                },
                Sub => match_binop! {
                    (Int(a), Int(b)) => { Int(a - b) }
                },
                Mul => match_binop! {
                    (Int(a), Int(b)) => { Int(a * b) }
                },

                Mod => match_binop! {
                    (Int(a), Int(b)) => {
                        assert!(b != 0);
                        Int(a % b)
                    }
                },

                Div => match_binop! {
                    (Int(a), Int(b)) => {
                        assert!(b != 0);
                        Int(a / b)
                    }
                },

                Lt      => match_binop! { (Int(a), Int(b)) => { Bool(a < b)  } },
                LtEqual => match_binop! { (Int(a), Int(b)) => { Bool(a <= b) } },
                Gt      => match_binop! { (Int(a), Int(b)) => { Bool(a > b)  } },
                GtEqual => match_binop! { (Int(a), Int(b)) => { Bool(a >= b) } },

                Equal => {
                    let a = self.value_stack.pop().unwrap();
                    let b = self.value_stack.pop().unwrap();

                    self.value_stack.push(Bool(a == b));
                },

                NEqual => {
                    let a = self.value_stack.pop().unwrap();
                    let b = self.value_stack.pop().unwrap();
                    self.value_stack.push(Bool(a != b));
                },

                Neg => if let Int(n) = self.value_stack.pop().unwrap() {
                    self.value_stack.push(Int(-n));
                } else {
                    panic!("invalid negative");
                },

                Print => println!("{:?}", self.value_stack.pop().unwrap()),

                Return => if let Some(call_info) = self.call_stack.pop() {
                    func    = unsafe { &*call_info.func };
                    locals  = call_info.locals;
                    pointer = call_info.pc
                } else {
                    break
                },

                Call(arg_count) => {
                    let arg_count = arg_count as usize;

                    let func_idx = self.value_stack.len() - arg_count - 1;
                    let func_val = self.value_stack[func_idx].clone();

                    let old_func = func;

                    func = if let Object(p) = func_val {
                        let obj = unsafe { &*p };
                        if let ObjectType::Lambda(ref func) = obj.kind {
                            unsafe { &*func.clone() }
                        } else {
                            panic!("Tried to call uncallable value {:?}", func_val);
                        }
                    } else {
                        panic!("Tried to call uncallable value {:?}", func_val);
                    };

                    let mut new_locals = vec![Nil; func.local_names.len()].into_boxed_slice();

                    for i in 0..arg_count {
                        new_locals[i] = self.value_stack[func_idx + 1 + i].clone();
                    }

                    for _ in 0..arg_count+1 {
                        self.value_stack.pop();
                    }

                    let old_locals = mem::replace(&mut locals, new_locals);

                    self.call_stack.push(CallInfo {
                        pc:     pointer,
                        locals: old_locals,
                        func:   old_func,
                    });

                    pointer = 0;

                    continue
                }

                _ => continue,
            }

            pointer = pointer.wrapping_add(1)
        }
    }

    pub fn alloc(&mut self, kind: ObjectType<'m>) -> Value<'m> {
        let object = Box::into_raw(
            Box::new(
                Object {
                    next: self.next_object,
                    kind,
                }
            )
        );
        
        self.next_object = object;

        Value::Object(object)
    }
}