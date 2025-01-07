use std::ops::Index;


#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Ty {
    I32,
    I64,
    F32,
    F64,
    Func(Vec<Ty>, Vec<Ty>),
}

impl Ty {
    pub fn func_tys(&self) -> (&[Ty], &[Ty]) {
        match self {
            Ty::Func(args, res) => (args, res),
            _ => panic!("not a function type"),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum BlockType {
    Empty,
    Index(usize),
    ValTy(Ty),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Instr {
    Unreachable,
    I32Const(i32),
    I32Add,
    I32Eq,
    LocalTee(usize),
    LocalGet(usize),
    LocalSet(usize),
    Br(usize),
    If(BlockType, Vec<Instr>, Vec<Instr>),
    Loop(BlockType, Vec<Instr>),
    Block(BlockType, Vec<Instr>),
}

#[derive(Debug, Clone, PartialEq)]
pub enum Val {
    I32(i32),
    I64(i64),
    F32(f32),
    F64(f64),
    NULL(Ty),
}

pub fn block_type<I: Index<usize, Output=Ty>>(ind_tys: &I, ty: &BlockType) -> Ty {
    match ty {
        BlockType::Empty => Ty::Func(vec![], vec![]),
        BlockType::Index(i) => ind_tys[*i].clone(),
        BlockType::ValTy(ty) => Ty::Func(vec![], vec![ty.clone()]),
    }
}
