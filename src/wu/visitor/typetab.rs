use std::rc::Rc;
use std::cell::RefCell;
use std::fmt;

use super::*;

// the same as symtab but with types and linked up with symtab
// using the usize from symtab's hashmap to index the Vec of types from typetab
#[derive(Clone)]
pub struct TypeTab {
    pub parent:  Option<Rc<TypeTab>>,
    pub types:   RefCell<Vec<Type>>,
    pub acc: usize,
}

impl TypeTab {
    pub fn new(parent: Rc<TypeTab>, types: &[Type]) -> TypeTab {
        TypeTab {
            parent:  Some(parent),
            types:   RefCell::new(types.to_owned()),
            acc: 0
        }
    }

    pub fn global() -> TypeTab {
        TypeTab {
            parent:  None,
            types:   RefCell::new(Vec::new()),
            acc: 0
        }
    }

    pub fn set_type(&self, index: usize, env_index: usize, t: Type) -> Response<()> {        
        if env_index == 0usize {
            match self.types.borrow_mut().get_mut(index) {
                Some(v) => {
                    *v = t;
                    Ok(())
                },
                None => Err(make_error(None, format!("invalid type index: {}", index)))
            }
        } else {
            match self.parent {
                Some(ref p) => p.set_type(index, env_index - 1, t),
                None        => Err(make_error(None, format!("invalid type env index: {}", env_index)))
            }
        }
    }

    pub fn get_type(&self, index: usize, env_index: usize) -> Response<Type> {
        if env_index == 0 {
            match self.types.borrow().get(index) {
                Some(v) => Ok(v.clone()),
                None    => Err(make_error(None, format!("invalid type index: {}", index)))
            }
        } else {
            match self.parent {
                Some(ref p) => p.get_type(index, env_index - 1),
                None        => Err(make_error(None, format!("invalid type index: {}", index)))
            }
        }
    }

    pub fn visualize(&self, env_index: usize) {
        if env_index > 0 {
            if let Some(ref p) = self.parent {
                p.visualize(env_index - 1);
                println!("------------------------------");
            }
        }

        for (i, v) in self.types.borrow().iter().enumerate() {
            println!("({} : {}) = {:?}", i, env_index, v)
        }
    }

    fn dump(&self, f: &mut fmt::Formatter, env_index: usize) -> fmt::Result {
        if env_index > 0 {
            if let Some(ref p) = self.parent {
                try!(p.dump(f, env_index - 1));
                try!(writeln!(f, "------------------------------"));
            }
        }

        for (i, v) in self.types.borrow().iter().enumerate() {
            try!(writeln!(f, "({} : {}) = {:?}", i, env_index, v))
        }

        Ok(())
    }

    pub fn size(&self) -> usize {
        self.types.borrow().len()
    }

    pub fn grow(&mut self) {
        RefCell::borrow_mut(&self.types).push(Type::nil())
    }
}

impl fmt::Debug for TypeTab {
    fn fmt(&self, f : &mut fmt::Formatter) -> Result<(), fmt::Error> {
        try!(self.dump(f, 0));
        Ok(())
    }
}
