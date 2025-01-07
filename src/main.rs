mod cont;
mod types;

use types::*;

enum Label {
    Empty(usize, Vec<usize>),
    Continuation(usize, Vec<usize>),
}

impl Label {
    fn num_rets(&self) -> usize {
        match self {
            Self::Empty(n, _) => n.clone(),
            Self::Continuation(n, _) => n.clone(),
        }
    }
}

enum StackItem {
    Val(Val),
    Label(Label),
}

fn nested_instrs(instr: &Instr) -> &Vec<Instr> {
    match instr {
        Instr::Block(_, instrs) => instrs,
        Instr::Loop(_, instrs) => instrs,
        _ => panic!("unreachable"),
    }
}

macro_rules! impl_stack_push {
    ($($type:ty, $fn_name:ident, $variant:ident),* $(,)?) => {
        $(
            fn $fn_name(&mut self, v: $type) {
                self.push(StackItem::Val(Val::$variant(v)))
            }
        )*
    };
}

macro_rules! impl_stack_pop {
    ($($type:ty, $fn_name:ident, $variant:ident),* $(,)?) => {
        $(
            fn $fn_name(&mut self) -> $type {
                if let StackItem::Val(Val::$variant(i)) = self.pop() {
                    i
                } else {
                    panic!("")
                }
            }
        )*
    }
}

macro_rules! stack_val {
    ($expr:expr) => {
        match $expr {
            StackItem::Val(v) => v,
            _ => panic!("impossible"),
        }
    };
}

fn block_instrs(instr: &Instr) -> &Vec<Instr> {
    match instr {
        Instr::Block(_, instrs) => instrs,
        Instr::Loop(_, instrs) => instrs,
        _ => panic!("unreachable"),
    }
}

fn n_func_rets(ty: &Ty) -> usize {
    match ty {
        Ty::Func(_, rets) => rets.len(),
        _ => panic!("impossible"),
    }
}

struct Level<'a> {
    cur: usize,
    len: usize,
    instrs: &'a Vec<Instr>,
}

impl<'a> Level<'a> {
    fn new(instrs: &'a Vec<Instr>) -> Self {
        Level {
            cur: 0,
            len: instrs.len(),
            instrs,
        }
    }

    fn instr(&self) -> &'a Instr {
        &self.instrs[self.cur]
    }
}

trait InstrCursor<'a> {
    fn instr(&self) -> Option<&'a Instr>;
    fn next(&mut self);
    fn pos(&self) -> Vec<usize>;
    fn seek(&mut self, pos: &Vec<usize>);
    fn push_instrs(&mut self, instrs: &'a Vec<Instr>);
}

impl<'a> InstrCursor<'a> for Vec<Level<'a>> {
    fn instr(&self) -> Option<&'a Instr> {
        self.last().map(|l| { l.instr() })
    }

    fn next(&mut self) {
        while let Some(r) = self.last_mut() {
            r.cur += 1;
            if r.cur == r.len {
                self.pop();
            } else {
                break;
            }
        }
    }

    fn seek(&mut self, pos: &Vec<usize>) {
        // most likely jump to a different instr at some higher level
        for i in 0..self.len() {
            if self[i].cur != pos[i] {
                self.truncate(i);
                break;
            }
        }
        if self.len() > pos.len() {
            panic!("impossible");
        }
        for i in self.len()..pos.len() {
            self.push_instrs(block_instrs(self[i].instr()))
        }
    }

    fn pos(&self) -> Vec<usize> {
        self.iter().map(|l| l.cur).collect()
    }

    fn push_instrs(&mut self, instrs: &'a Vec<Instr>) {
        self.push(Level::new(instrs))
    }
}

struct VM {
    stack: Vec<StackItem>,
    halt: bool,
    // move this into stack frame
    locals: Vec<Val>,
    types: Vec<Ty>,
}

impl VM {
    fn new() -> VM {
        VM {
            stack: vec![],
            halt: false, // should be a thread state
            locals: vec![Val::I32(0)],
            types: vec![],
        }
    }

    fn run(&mut self, instrs: &Vec<Instr>) {
        let mut cursor = if instrs.len() > 0 {
            vec![Level::new(instrs)]
        } else {
            Vec::new()
        };
        while let Some(instr) = cursor.instr() {
            self.step(instr, &mut cursor);
        }
    }

    fn step<'a>(&mut self, instr: &'a Instr, cursor: &mut Vec<Level<'a>>) {
        // most instrs moves cursor to next, so we factor out a boolean
        let mut cursor_updated = false;
        // execute instr
        match instr {
            Instr::Unreachable => { }
            Instr::I32Const(i) => self.push_i32(i.clone()),
            Instr::I32Add => {
                let i1 = self.pop_i32();
                let i2 = self.pop_i32();
                self.push_i32(i1 + i2);
            },
            Instr::I32Eq => {
                let a = self.pop_i32();
                let b = self.pop_i32();
                if a == b {
                    self.push_i32(1);
                } else {
                    self.push_i32(0);
                }
            }
            &Instr::LocalSet(i) => {
                let v = stack_val!(self.pop());
                self.locals[i] = v.clone();
            },
            &Instr::LocalTee(i) => {
                let v = stack_val!(self.pop());
                self.locals[i] = v.clone();
                self.push(StackItem::Val(v));
            },
            &Instr::LocalGet(i) => {
                self.push(StackItem::Val(self.locals[i].clone()));
            },
            &Instr::Br(l) => {
                let (label, vals) = self.pop_label(l);
                vals.into_iter().for_each(|v| self.push(StackItem::Val(v)));
                match label {
                    Label::Empty(_, pos) => {
                        cursor.seek(&pos);
                        cursor.next();
                    }
                    Label::Continuation(_, pos) => {
                        cursor.seek(&pos);
                    }
                }
                cursor_updated = true;
            },
            Instr::Loop(bt, instrs) => {
                self.push(StackItem::Label(Label::Continuation(block_type(&self.types, bt).func_tys().0.len(), cursor.pos())));
                cursor.push_instrs(instrs);
                cursor_updated = true;
            },
            Instr::Block(bt, instrs) => {
                self.push(StackItem::Label(Label::Empty(block_type(&self.types, bt).func_tys().1.len(), cursor.pos())));
                cursor.push_instrs(instrs);
                cursor_updated = true;
            },
            Instr::If(bt, instrs_then, instrs_else) => {
                let b = self.pop_i32();
                self.push(StackItem::Label(Label::Empty(block_type(&self.types, bt).func_tys().1.len(), cursor.pos())));
                if b != 0 {
                    cursor.push_instrs(instrs_then);
                } else {
                    cursor.push_instrs(instrs_else);
                }
                cursor_updated = true;
            },
        }
        if !cursor_updated {
            cursor.next();
        }
    }

    fn result(&self) -> Option<Val> {
        if let Some(StackItem::Val(v)) = self.stack.last() {
            Some(v.clone())
        } else {
            None
        }
    }

    #[inline]
    fn push(&mut self, item: StackItem) {
        self.stack.push(item);
    }

    #[inline]
    fn pop(&mut self) -> StackItem {
        self.stack.pop().unwrap()
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
        f32, pop_f32, F32,
        f64, pop_f64, F64,
    );

    fn pop_label(&mut self, mut n_labels: usize) -> (Label, Vec<Val>) {
        let mut i_lbl = None;
        for i in (0..self.stack.len()).rev() {
            if let StackItem::Label(lbl) = &self.stack[i] {
                if n_labels == 0 {
                    i_lbl = Some(i);
                }
                n_labels -= 1;
            }
        }
        let i_lbl = i_lbl.unwrap();
        let n_vals = if let StackItem::Label(lbl) = &self.stack[i_lbl] { lbl.num_rets() } else { panic!("impossible") };
        let vals = self.stack.drain((self.stack.len()-n_vals)..)
            .map(|i| match i {
                StackItem::Val(v) => v,
                _ => panic!("impossible"),
            })
            .collect::<Vec<Val>>();
        self.stack.truncate(i_lbl+1);
        let label = {
            match self.stack.pop() {
                Some(StackItem::Label(lbl)) => lbl,
                _ => panic!("impossible"),
            }
        };
        (label, vals)
    }
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

    #[test]
    pub fn test_block() {
        let mut vm = VM::new();
        vm.run(&vec![
            Instr::Block(
                BlockType::ValTy(Ty::I32),
                vec![
                    Instr::I32Const(1),
                    Instr::I32Const(1),
                    Instr::I32Add,
                ]),
        ]);
        assert_eq!(Some(Val::I32(2)), vm.result());
    }

    #[test]
    pub fn test_loop() {
        let mut vm = VM::new();
        vm.run(&vec![
            Instr::Block(BlockType::Empty, vec![
                Instr::Loop(BlockType::Empty, vec![
                    Instr::LocalGet(0),
                    Instr::I32Const(1),
                    Instr::I32Add,
                    Instr::LocalTee(0),
                    Instr::I32Const(3),
                    Instr::I32Eq,
                    Instr::If(BlockType::Empty, vec![
                        Instr::Br(1),
                    ], vec![
                        Instr::Br(0),
                    ]),
                ])
            ]),
            Instr::LocalGet(0),
        ]);
        assert_eq!(Some(Val::I32(2)), vm.result());
    }
}
