pub struct OperandStack {
    vec: Vec<i32>,
}

impl OperandStack {
    pub fn new() -> OperandStack {
        OperandStack {
            vec: vec![],
        }
    }

    pub fn pop(&mut self) -> i32 {
        self.vec.pop().unwrap()
    }

    pub fn push(&mut self, n: i32) {
        self.vec.push(n);
    }

    pub fn swap(&mut self) {
        let len = self.vec.len();
        self.vec.swap(len- 1, len - 2);
    }
}

pub struct LocalVariables {
    vec: Vec<i32>
}

impl LocalVariables {
    pub fn new(size: usize) -> LocalVariables {
        LocalVariables {
            vec: vec![0; size]
        }
    }

    pub fn store(&mut self, address: u16, value: i32) {
        self.vec.insert(address as usize, value);
    }

    pub fn store2(&mut self, address: u16, value1: i32, value2: i32) {
        self.vec.insert(address as usize, value1);
        self.vec.insert(address as usize + 1, value2);
    }

    pub fn load(&self, address: u16) -> i32 {
        self.vec[address as usize]
    }
    pub fn load2(&self, address: u16) -> (i32, i32) {
        (self.vec[address as usize], self.vec[address as usize + 1])
    }

}

#[cfg(test)]
mod tests {
    use crate::model::{LocalVariables, OperandStack};

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
        let mut vars = LocalVariables::new(10);

        vars.store(1, 546);
        vars.store(2, 100);
        vars.store2(3, 100, 466);

        assert_eq!(vars.load(1), 546);
        assert_eq!(vars.load(3), 100);
        assert_eq!(vars.load(4), 466);
    }
}