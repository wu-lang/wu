#![macro_use]



#[macro_export]
macro_rules! to_bytes {
    ($value:expr => $t:ty) => {{
        unsafe { mem::transmute::<_,[u8;mem::size_of::<$t>()]>($value) }
    }}
}



#[macro_export]
macro_rules! from_bytes {
  ($raw:expr => $t:ty) => {{
    let mut b: [u8; mem::size_of::<$t>()] = default::Default::default();
    b.copy_from_slice($raw);
    unsafe { mem::transmute::<_,$t>(b) }
  }}
}



#[macro_export]
macro_rules! memmove {
  ($source:expr => $target:expr,[$from:expr; $size:expr]) => {{
    $target[$from as usize .. ($from + $size) as usize].copy_from_slice($source);
  }}
}



#[macro_export]
macro_rules! pop {
    ([$stack:expr, $top:expr] => $type:ty) => {{
        $top -= mem::size_of::<$type>() as u32;
        from_bytes!(&read($stack, $top, mem::size_of::<$type> as u32) =>$type)
    }}
}



pub mod vm;
pub mod compiler;

use super::parser::*;
use super::visitor::*;

pub use self::vm::*;
pub use self::compiler::*;