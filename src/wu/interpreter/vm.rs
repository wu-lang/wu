use std::slice;
use std::mem;
use std::default;



macro_rules! from_bytes {
    ($raw:expr => $t:ty) => {{
        let mut b: [u8; mem::size_of::<$t>()] = default::Default::default();
        b.copy_from_slice($raw);
        unsafe { mem::transmute::<_,$t>(b) }
    }}
}



pub enum Instruction {
  HALT = 0x00,
  PUSH = 0x01,
  POP  = 0x02,
}

pub struct VirtualMachine {
  pub var_stack:     [u8; 262144],
  pub compute_stack: [u8; 262144],
  pub frames:        Vec<u32>,

  var_top:     u32,
  compute_top: u32,
}

impl VirtualMachine {
  pub fn new() -> Self {
    VirtualMachine {
      var_stack:     [0; 262144],
      compute_stack: [0; 262144],
      frames:        Vec::new(),

      var_top:     0,
      compute_top: 0,
    }
  }

  pub fn execute(&mut self, bytecode: &[u8]) -> Result<(), ()> {
    use self::Instruction::*;

    let mut ip = 0;

    loop {
      match unsafe { mem::transmute::<u8, Instruction>(bytecode[ip]) } {
        PUSH => {
          ip += 1;

          let size = bytecode[ip];

          ip += 1;

          let value = read(&bytecode, ip as u32, size as u32);

          memmove(&value, &mut self.compute_stack, self.compute_top, size as u32);

          ip += 1;

          self.compute_top += size as u32
        },

        POP => {
          ip += 1;

          let size = bytecode[ip];

          ip += 1;

          let address = from_bytes!(&bytecode[ip .. ip + size as usize] => u32);

          ip += 1;

          let variable = read(&self.var_stack, address + *self.frames.last().unwrap(), size as u32);
        
          self.compute_top -= size as u32;

          memmove(&variable, &mut self.compute_stack, self.compute_top, size as u32);
          
          if self.var_top < (address + size as u32) {
            self.var_top = address + size as u32
          }
        }

        HALT => break,
      }
    }

    Ok(())
  }
}



fn read (mem: &[u8], from: u32, size: u32) -> &[u8] {
  &mem[from as usize .. (from + size) as usize]
}

fn memmove (source: &[u8], target: &mut [u8], from: u32, size: u32) {
    target[from as usize .. (from + size) as usize].copy_from_slice(&source);
}