use std::fmt;
use std::mem;
use std::default;

#[macro_use]
use super::*;



pub enum Instruction {
  Halt      = 0x00,
  Push      = 0x01,
  Pop       = 0x02,
  PushDeref = 0x03,
  ConvIF    = 0x04,
  ConvFI    = 0x05,
  ConvII    = 0x06,
  ConvFF    = 0x07,
  AddI      = 0x08,
  AddF      = 0x09,
  Jmp       = 0x10,

  CmpI      = 0x11,
  CmpF      = 0x12,

  JEq       = 0x13,
  JLt       = 0x14,
  JGt       = 0x15,
  JNE       = 0x16,
  JLE       = 0x17,
  JGE       = 0x18,
}

impl fmt::Display for Instruction {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    use self::Instruction::*;

    let name = match *self {
      Halt      => "halt",
      Push      => "push",
      Pop       => "pop",
      PushDeref => "pushd",
      ConvIF    => "convif",
      ConvFI    => "convfi",
      ConvII    => "convii",
      ConvFF    => "convff",
      AddI      => "addi",
      AddF      => "addf",
      Jmp       => "jmp",
      CmpI      => "cmpi",
      CmpF      => "cmpf",

      JEq       => "jeq",
      JLt       => "jlt",
      JGt       => "jgt",
      JNE       => "jne",
      JLE       => "jle",
      JGE       => "jge",
    };

    write!(f, "{}", name)
  }
}



mod CmpResult {
  pub const Eq: u8 = 0b001;
  pub const Gt: u8 = 0b010;
  pub const Lt: u8 = 0b100;
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

    let mut ip: u32 = 0;

    loop {
      match unsafe { mem::transmute::<u8, Instruction>(bytecode[ip as usize]) } {
        Halt => break,

        Push => {
          ip += 1;

          let size = bytecode[ip as usize];

          ip += 1;

          let value = &read(&bytecode, ip as u32, size as u32);
          ip += size as u32;

          memmove!(value => self.compute_stack, [self.compute_top; size as u32]);      

          self.compute_top += size as u32
        },

        Pop => {
          ip += 1;

          let size = bytecode[ip as usize];

          ip += 1;

          let address = from_bytes!(&bytecode[ip as usize .. ip as usize + 4] => u32);

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

          let size = bytecode[ip as usize];

          ip += 1;

          let address = from_bytes!(&bytecode[ip as usize .. ip as usize + 4] => u32) + self.frames.last().unwrap();

          ip += 4;

          let value = &read(&self.var_stack, address, size as u32);

          self.var_top -= size as u32;

          memmove!(value => self.compute_stack, [self.compute_top; size as u32]);

          if self.compute_top < (address + size as u32) {
            self.compute_top = address + size as u32
          }
        },

        // Int to Float ; size_from size_to ; convif i8 f32
        ConvIF => {
          ip += 1;

          let mut size_from = bytecode[ip as usize] as i8;

          let is_signed = size_from < 0;

          if is_signed {
            size_from = -size_from 
          };

          ip += 1;

          let size_to = bytecode[ip as usize];

          ip += 1;

          self.compute_top += 16 - size_from as u32;

          let value = from_bytes!(&read(&self.compute_stack, self.compute_top - 16, 16) => u128);

          self.compute_top -= 16;

          if size_to == 4 {
            let new_value = if is_signed {
              value as i128 as f32
            } else {
              value as f32
            };

            let converted_size = mem::size_of::<f32>() as u32;

            memmove!(&to_bytes!(new_value => f32) => self.compute_stack, [self.compute_top; converted_size]);      

            self.compute_top += converted_size

          } else if size_to == 8 {
            let new_value = if is_signed {
              value as i128 as f64
            } else {
              value as f64
            };

            let converted_size = mem::size_of::<f64>() as u32;

            memmove!(&to_bytes!(new_value => f64) => self.compute_stack, [self.compute_top; converted_size]);      

            self.compute_top += converted_size
          }
        },

        ConvFI => {
          ip += 1;

          let mut size_from = bytecode[ip as usize] as i8;

          let is_signed = size_from < 0;

          if is_signed {
            size_from = -size_from 
          };

          ip += 1;

          let size_to = bytecode[ip as usize];

          ip += 1;

          let value = from_bytes!(&read(&self.compute_stack, self.compute_top - size_from as u32, size_from as u32) => u32);

          if size_to == 4 {
            let new_value = if is_signed {
              value as i128 as f32
            } else {
              value as f32
            };

            let converted_size = mem::size_of::<f32>() as u32;

            memmove!(&to_bytes!(new_value => u32)[0 .. size_to as usize] => self.compute_stack, [self.compute_top; size_to as u32]);

            self.compute_top += converted_size

          } else if size_to == 8 {
            let new_value = if is_signed {
              value as i128 as f64
            } else {
              value as f64
            };

            let converted_size = mem::size_of::<f64>() as u32;

            memmove!(&to_bytes!(new_value => u64)[0 .. size_to as usize] => self.compute_stack, [self.compute_top; size_to as u32]);

            self.compute_top += converted_size
          }
        },

        ConvII => {
          ip += 1;

          let size_from = bytecode[ip as usize];

          ip += 1;

          let size_to = bytecode[ip as usize];

          ip += 1;

          if size_from != size_to {
            self.compute_top = (self.compute_top as i32 + size_to as i32 - size_from as i32) as u32
          }
        },

        ConvFF => {
          ip += 1;

          let size_from = bytecode[ip as usize];

          ip += 1;

          let size_to = bytecode[ip as usize];

          ip += 1;

          let value = from_bytes!(&read(&self.compute_stack, self.compute_top - size_from as u32, size_from as u32) => u32);

          if size_to == 4 {
            let new_value      = value as f32;
            let converted_size = mem::size_of::<f32>() as u32;

            memmove!(&to_bytes!(new_value => f32) => self.compute_stack, [self.compute_top; converted_size]);

            self.compute_top += converted_size

          } else if size_to == 8 {
            let new_value      = value as f64;
            let converted_size = mem::size_of::<f64>() as u32;

            memmove!(&to_bytes!(new_value => f64) => self.compute_stack, [self.compute_top; converted_size]);

            self.compute_top += converted_size
          }
        },

        AddI => {
          ip += 1;

          let size = bytecode[ip as usize];

          ip += 1;

          match size {
            1 => {
              let b = pop!([&self.compute_stack, self.compute_top] => u8);
              let a = pop!([&self.compute_stack, self.compute_top] => u8);

              push!(&to_bytes!(a.wrapping_add(b) => u8) => self.compute_stack, [self.compute_top; size as u32]);
            },

            4 => {
              let b = pop!([&self.compute_stack, self.compute_top] => u32);
              let a = pop!([&self.compute_stack, self.compute_top] => u32);

              push!(&to_bytes!(a.wrapping_add(b) => u32) => self.compute_stack, [self.compute_top; size as u32]);
            },

            8 => {
              let b = pop!([&self.compute_stack, self.compute_top] => u64);
              let a = pop!([&self.compute_stack, self.compute_top] => u64);

              push!(&to_bytes!(a.wrapping_add(b) => u64) => self.compute_stack, [self.compute_top; size as u32]);
            },

            16 => {
              let b = pop!([&self.compute_stack, self.compute_top] => u128);
              let a = pop!([&self.compute_stack, self.compute_top] => u128);

              push!(&to_bytes!(a.wrapping_add(b) => u128) => self.compute_stack, [self.compute_top; size as u32]);
            },

            _ => unreachable!()
          }
        },

        AddF => {
          ip += 1;

          let size = bytecode[ip as usize];

          ip += 1;

          match size {
            4 => {
              let b = pop!([&self.compute_stack, self.compute_top] => f32);
              let a = pop!([&self.compute_stack, self.compute_top] => f32);

              push!(&to_bytes!(a + b => f32) => self.compute_stack, [self.compute_top; size as u32]);
            },

            8 => {
              let b = pop!([&self.compute_stack, self.compute_top] => f64);
              let a = pop!([&self.compute_stack, self.compute_top] => f64);

              push!(&to_bytes!(a + b => f64) => self.compute_stack, [self.compute_top; size as u32]);
            },

            _ => unreachable!()
          }
        },

        Jmp => {
          ip += 5;

          ip = pop!([bytecode, ip] => u32)
        },

        CmpI => {
          ip += 1;

          let b = pop!([&self.compute_stack, self.compute_top] => u128);
          let a = pop!([&self.compute_stack, self.compute_top] => u128);

          let mut result = 0b000;

          if a == b {
            result |= CmpResult::Eq
          }

          if a > b {
            result |= CmpResult::Gt
          }

          if a < b {
            result |= CmpResult::Lt
          }

          push!((&to_bytes!(result as u8 => u8)) => self.compute_stack, [self.compute_top; 1 as u32])
        },

        CmpF => {
          ip += 1;

          let a = pop!([&self.compute_stack, self.compute_top] => f64);
          let b = pop!([&self.compute_stack, self.compute_top] => f64);

          let mut result = 0b000;

          if a == b {
            result |= CmpResult::Eq
          }

          if a < b {
            result |= CmpResult::Lt
          }

          if a > b {
            result |= CmpResult::Gt
          }

          push!((&to_bytes!(result as u8 => u8)) => self.compute_stack, [self.compute_top; 1 as u32])
        },

        JEq => {
          ip += 5;

          if pop!([&self.compute_stack, self.compute_top] => u8) as u8 == CmpResult::Eq {
            ip = pop!([bytecode, ip] => u32)
          }
        },

        JLt => {
          ip += 5;

          if pop!([&self.compute_stack, self.compute_top] => u8) as u8 == CmpResult::Lt {
            ip = pop!([bytecode, ip] => u32)
          }
        },

        JGt => {
          ip += 5;

          if pop!([&self.compute_stack, self.compute_top] => u8) as u8 == CmpResult::Gt {
            ip = pop!([bytecode, ip] => u32)
          }
        },

        JNE => {
          ip += 5;

          if pop!([&self.compute_stack, self.compute_top] => u8) as u8 != CmpResult::Eq {
            ip = pop!([bytecode, ip] => u32)
          }
        },

        JLE => {
          ip += 5;

          if pop!([&self.compute_stack, self.compute_top] => u8) as u8 == CmpResult::Lt | CmpResult::Eq {
            ip = pop!([bytecode, ip] => u32)
          }
        },

        JGE => {
          ip += 5;

          if pop!([&self.compute_stack, self.compute_top] => u8) as u8 == CmpResult::Gt | CmpResult::Eq {
            ip = pop!([bytecode, ip] => u32)
          }
        },
      }
    }

    Ok(())
  }
}



fn read (mem: &[u8], from: u32, size: u32) -> Vec<u8> {
  mem[from as usize .. (from + size) as usize].iter().cloned().collect()
}