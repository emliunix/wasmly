#[derive(Debug, Clone, PartialEq, Eq)]
enum Ty {
    I32,
    I64,
    F32,
    F64,
}

pub enum Instr {
    I32Const(i32),
    I32Add,
}

#[derive(Debug, Clone, PartialEq)]
enum Val {
    I32(i32),
    I64(i64),
    F32(f32),
    F64(f64),
    NULL(Ty),
}

struct Label;

enum StackItem {
    Val(Val),
    Label(Label),
}

struct VM {
    stack: Vec<StackItem>,
}

macro_rules! impl_stack_push {
    ($($type:ty, $fn_name:ident, $variant:ident),* $(,)?) => {
        $(
            fn $fn_name(&mut self, v: $type) {
                self.stack.push(StackItem::Val(Val::$variant(v)))
            }
        )*
    };
}

macro_rules! impl_stack_pop {
    ($($type:ty, $fn_name:ident, $variant:ident),* $(,)?) => {
        $(
            fn $fn_name(&mut self) -> $type {
                if let Some(StackItem::Val(Val::$variant(i))) = self.stack.pop() {
                    i
                } else {
                    panic!("")
                }
            }
        )*
    }
}


impl VM {
    fn new() -> VM {
        VM {
            stack: vec![],
        }
    }

    fn run(&mut self, instrs: &Vec<Instr>) {
        for i in 0..instrs.len() {
            self.step(i, instrs);
        }
    }

    fn step(&mut self, i: usize, instrs: &Vec<Instr>) {
        match instrs[i] {
            Instr::I32Const(i) => self.push_i32(i),
            Instr::I32Add => {
                let i1 = self.pop_i32();
                let i2 = self.pop_i32();
                self.push_i32(i1 + i2);
            },
        }
    }

    fn result(&self) -> Option<Val> {
        if let Some(StackItem::Val(v)) = self.stack.last() {
            Some(v.clone())
        } else {
            None
        }
    }

    impl_stack_push!(
        i32, push_i32, I32,
        i64, push_i64, I64,
        f32, push_f32, F32,
        f64, push_f64, F64,
    );

    impl_stack_pop!(
        i32, pop_i32, I32,
        i64, pop_i64, I64,
    );
}

fn main() {
    println!("Hello, world!");
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    pub fn test() {
        let mut vm = VM::new();
        vm.run(&vec![
            Instr::I32Const(1),
            Instr::I32Const(1),
            Instr::I32Add,
        ]);
        assert_eq!(Some(Val::I32(2)), vm.result());
    }
}
