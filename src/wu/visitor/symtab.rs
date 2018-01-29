use std::rc::Rc;
use std::cell::RefCell;
use std::collections::HashMap;

use std::fmt;

// a simple symbol table for keeping track of scopes
//
#[derive(Clone)]
pub struct SymTab {
    pub parent: Option<Rc<SymTab>>,
    pub names:  RefCell<HashMap<String, usize>>,
}

impl SymTab {
    pub fn new(parent: Rc<SymTab>, names: &[String]) -> SymTab {
        let mut hash_names = HashMap::new();

        for (i, name) in names.iter().enumerate() {
            hash_names.insert((*name).clone(), i);
        }

        SymTab {
            parent:  Some(parent),
            names:   RefCell::new(hash_names),
        }
    }

    pub fn global() -> SymTab {
        SymTab {
            parent:  None,
            names:   RefCell::new(HashMap::new()),
        }
    }

    pub fn add_name(&self, name: &str) -> usize {
        let new_index = self.names.borrow().len();
        self.names.borrow_mut().insert(name.to_string(), new_index);

        new_index
    }

    pub fn get_name(&self, name: &str) -> Option<(usize, usize)> {
        self.get_name_internal(name, 0)
    }

    fn get_name_internal(&self, name: &str, env_index: usize) -> Option<(usize, usize)> {
        if let Some(index) = self.names.borrow().get(name) {
            return Some((*index, env_index));
        }

        match self.parent {
            Some(ref parent) => parent.get_name_internal(name, env_index + 1),
            None => None,
        }
    }

    pub fn visualize(&self, env_index: usize) {
        if env_index > 0 {
            if let Some(ref p) = self.parent {
                p.visualize(env_index - 1);
                println!("------------------------------");
            }
        }

        for (i, v) in self.names.borrow().iter().enumerate() {
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

        for (i, v) in self.names.borrow().iter().enumerate() {
            try!(writeln!(f, "({} : {}) = {:?}", i, env_index, v))
        }

        Ok(())
    }
}

impl fmt::Debug for SymTab {
    fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        try!(self.dump(f, 0));
        Ok(())
    }
}
