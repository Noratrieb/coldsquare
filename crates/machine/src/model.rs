pub struct OperandStack {
    arr: [u32; 255],
    sp: u8,
}

impl OperandStack {
    pub fn new() -> Self {
        Self {
            arr: [0; 255],
            sp: 0,
        }
    }

    pub fn pop(&mut self) -> u32 {
        self.sp -= 1;
        self.arr[self.sp as usize]
    }

    pub fn push(&mut self, n: u32) {
        self.arr[self.sp as usize] = n;
        self.sp += 1;
    }

    pub fn swap(&mut self) {
        self.arr
            .swap((self.sp - 2) as usize, (self.sp - 2) as usize);
    }
}

pub struct LocalVariables {
    arr: [u32; 255],
}

impl LocalVariables {
    pub fn new() -> Self {
        Self { arr: [0; 255] }
    }

    pub fn store(&mut self, address: u8, value: u32) {
        self.arr[address as usize] = value;
    }

    pub fn store2(&mut self, address: u8, value1: u32, value2: u32) {
        self.arr[address as usize] = value1;
        self.arr[address as usize + 1] = value2;
    }

    pub fn load(&self, address: u8) -> u32 {
        self.arr[address as usize]
    }
    pub fn load2(&self, address: u8) -> (u32, u32) {
        (self.arr[address as usize], self.arr[address as usize + 1])
    }
}

#[cfg(test)]
mod tests {
    use super::{LocalVariables, OperandStack};

    #[test]
    fn operand_stack() {
        let mut stack = OperandStack::new();

        stack.push(10);
        stack.push(20);
        stack.push(30);
        stack.push(40);
        stack.swap();

        assert_eq!(stack.pop(), 30);
        assert_eq!(stack.pop(), 40);
        assert_eq!(stack.pop(), 20);
        assert_eq!(stack.pop(), 10);
    }

    #[test]
    fn local_vars() {
        let mut vars = LocalVariables::new();

        vars.store(1, 546);
        vars.store(2, 100);
        vars.store2(3, 100, 466);

        assert_eq!(vars.load(1), 546);
        assert_eq!(vars.load(3), 100);
        assert_eq!(vars.load(4), 466);
    }
}
