use std::cell::RefCell;
use super::Type;
use super::super::error::Response::Wrong;



#[derive(Clone)]
pub struct TypeTab<'s> {
  pub parent:  Option<&'s TypeTab<'s>>,
  pub types:   RefCell<Vec<Type>>,
}

impl<'s> TypeTab<'s> {
  pub fn new(parent: &'s TypeTab, types: &[Type]) -> Self {

    TypeTab {
      parent: Some(parent),
      types:  RefCell::new(types.to_owned()),
    }
  }

  pub fn global() -> Self {
    TypeTab {
      parent: None,
      types:  RefCell::new(Vec::new()),
    }
  }

  pub fn set_type(&self, index: usize, env_index: usize, t: Type) -> Result<(), ()> {
    if env_index == 0usize {
      match self.types.borrow_mut().get_mut(index) {
        Some(v) => {
          *v = t;
          Ok(())
        },
        None => Err(response!(Wrong("[type table] invalid type index")))
      }
    } else {
      match self.parent {
        Some(ref p) => p.set_type(index, env_index - 1, t),
        None        => Err(response!(Wrong("[type table] invalid environment index")))
      }
    }
  }

  pub fn get_type(&self, index: usize, env_index: usize) -> Result<Type, ()> {
    if env_index == 0 {
      match self.types.borrow().get(index) {
        Some(v) => Ok(v.clone()),
        None    => Err(response!(Wrong("[type table] invalid type index")))
      }
    } else {
      match self.parent {
        Some(ref p) => p.get_type(index, env_index - 1),
        None        => Err(response!(Wrong("[type table] invalid environment index")))
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



  pub fn size(&self) -> usize {
    self.types.borrow().len()
  }

  pub fn grow(&mut self) {
    RefCell::borrow_mut(&self.types).push(Type::nil())
  }
}