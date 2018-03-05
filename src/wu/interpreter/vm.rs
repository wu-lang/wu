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



macro_rules! memmove {
  ($source:expr => $target:expr,[$from:expr; $size:expr]) => {{
    $target[$from as usize .. ($from + $size) as usize].copy_from_slice($source);
  }}
}



pub enum Instruction {
  Halt = 0x00,
  Push = 0x01,
  Pop  = 0x02,
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
      let last_code = bytecode[ip];

      match unsafe { mem::transmute::<u8, Instruction>(bytecode[ip]) } {
        Push => {
          ip += 1;

          let size = bytecode[ip];

          ip += 1;

          let value = read(&bytecode, ip as u32, size as u32);
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

          let value = read(&self.compute_stack, self.compute_top - size as u32, size as u32);

          self.compute_top -= size as u32;

          memmove!(value => self.var_stack, [address + *self.frames.last().unwrap(); size as u32]);

          if self.var_top < (address + size as u32) {
            self.var_top = address + size as u32
          }
        }

        Halt => break,
      }

      println!("\nstate(last code was {}):\n\tstack: {:?}\n\tvars: {:?}", last_code, &self.compute_stack[..16], &self.var_stack[..16])
    }

    Ok(())
  }
}



fn read (mem: &[u8], from: u32, size: u32) -> &[u8] {
  &mem[from as usize .. (from + size) as usize]
}