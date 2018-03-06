use std::cell::RefCell;
use std::collections::HashMap;

use std::rc::Rc;



#[derive(Clone)]
pub struct SymTab<'s> {
  pub parent: Option<&'s SymTab<'s>>,
  pub names:  RefCell<HashMap<String, usize>>,
}

impl<'s> SymTab<'s> {
  pub fn new(parent: &'s Self, names: &[String]) -> Self {
    let mut hash_names = HashMap::new();

    for (i, name) in names.iter().enumerate() {
      hash_names.insert((*name).clone(), i);
    }

    SymTab {
      parent:  Some(parent),
      names:   RefCell::new(hash_names),
    }
  }

  pub fn global() -> Self {
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
}