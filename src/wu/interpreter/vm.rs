use std::slice;
use std::mem;
use std::default;

#[macro_use]
use super::*;



pub enum Instruction {
  Halt      = 0x00,
  Push      = 0x01,
  Pop       = 0x02,
  PushDeref = 0x03,
  ToF32     = 0x04,
  ToI32     = 0x05,
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
      frames:        vec!(0),

      var_top:     0,
      compute_top: 0,
    }
  }

  pub fn execute(&mut self, bytecode: &[u8]) -> Result<(), ()> {
    use self::Instruction::*;

    let mut ip = 0;

    loop {
      match unsafe { mem::transmute::<u8, Instruction>(bytecode[ip]) } {
        Halt => break,

        Push => {
          ip += 1;

          let size = bytecode[ip];

          ip += 1;

          let value = &read(&bytecode, ip as u32, size as u32);
          ip += size as usize;

          memmove!(value => self.compute_stack, [self.compute_top; size as u32]);      

          self.compute_top += size as u32
        },

        Pop => {
          ip += 1;

          let size = bytecode[ip];

          ip += 1;

          let address = from_bytes!(&bytecode[ip .. ip + 4] => u32);

          ip += 4;

          let value = &read(&self.compute_stack, self.compute_top - size as u32, size as u32);

          self.compute_top -= size as u32;

          memmove!(value => self.var_stack, [address + *self.frames.last().unwrap(); size as u32]);

          if self.var_top < (address + size as u32) {
            self.var_top = address + size as u32
          }
        }

        PushDeref => {
          ip += 1;

          let size = bytecode[ip];

          ip += 1;

          let address = from_bytes!(&bytecode[ip .. ip + 4] => u32) + self.frames.last().unwrap();

          ip += 4;

          let value = &read(&self.var_stack, address, size as u32);

          self.var_top -= size as u32;

          memmove!(value => self.compute_stack, [self.compute_top; size as u32]);

          if self.compute_top < (address + size as u32) {
            self.compute_top = address + size as u32
          }
        },

        ToF32 => {
          ip += 1;

          let size = bytecode[ip];

          ip += 1;

          self.compute_top += mem::size_of::<f32>() as u32 - size as u32;

          let compute_stack_tmp = self.compute_stack.clone();
          let value = &read(&compute_stack_tmp, self.compute_top - size as u32, size as u32);

          self.compute_top -= size as u32;

          let converted      = from_bytes!(value => u32) as f32;
          let converted_size = mem::size_of::<f32>() as u32;

          memmove!(&to_bytes!(converted => f32) => self.compute_stack, [self.compute_top; converted_size]);      

          self.compute_top += converted_size
        }

        ToI32 => {
          ip += 1;

          let size = bytecode[ip];

          ip += 1;

          self.compute_top += mem::size_of::<i32>() as u32 - size as u32;

          let compute_stack_tmp = self.compute_stack.clone();
          let value = &read(&compute_stack_tmp, self.compute_top - size as u32, size as u32);

          self.compute_top -= size as u32;

          let converted      = from_bytes!(value => u32) as i32;
          let converted_size = mem::size_of::<i32>() as u32;

          memmove!(&to_bytes!(converted => i32) => self.compute_stack, [self.compute_top; converted_size]);      

          self.compute_top += converted_size
        }

        _ => (),
      }
    }

    Ok(())
  }
}



fn read (mem: &[u8], from: u32, size: u32) -> Vec<u8> {
  mem[from as usize .. (from + size) as usize].iter().cloned().collect()
}