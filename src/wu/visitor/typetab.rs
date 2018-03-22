use std::cell::RefCell;
use super::{ Type, TypeNode, };
use super::super::error::Response::Wrong;

use std::rc::Rc;



#[derive(Clone, Debug)]
pub struct TypeTab {
  pub parent:  Option<Rc<TypeTab>>,
  pub types:   RefCell<Vec<(Type, u32, u32)>>, // type and offset
}

impl TypeTab {
  pub fn new(parent: Rc<Self>, types: &[(Type, u32, u32)]) -> Self {
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

  pub fn set_type(&self, index: usize, env_index: usize, t: (Type, u32, u32)) -> Result<(), ()> {
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
        Some(v) => Ok(v.0.clone()),
        None    => Err(response!(Wrong("[type table] invalid type index")))
      }
    } else {
      match self.parent {
        Some(ref p) => p.get_type(index, env_index - 1),
        None        => Err(response!(Wrong("[type table] invalid environment index")))
      }
    }
  }

  pub fn get_offset(&self, index: usize, env_index: usize) -> Result<u32, ()> {
    if env_index == 0 {
      match self.types.borrow().get(index) {
        Some(v) => Ok(v.1.clone()),
        None    => Err(response!(Wrong("[type table] invalid type index")))
      }
    } else {
      match self.parent {
        Some(ref p) => p.get_offset(index, env_index - 1),
        None        => Err(response!(Wrong("[type table] invalid environment index")))
      }
    }
  }

  pub fn get_depth(&self, index: usize, env_index: usize) -> Result<u32, ()> {
    if env_index == 0 {
      match self.types.borrow().get(index) {
        Some(v) => Ok(v.2.clone()),
        None    => Err(response!(Wrong("[type table] invalid type index")))
      }
    } else {
      match self.parent {
        Some(ref p) => p.get_depth(index, env_index - 1),
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
    RefCell::borrow_mut(&self.types).push((Type::from(TypeNode::Nil), 0, 0))
  }
}