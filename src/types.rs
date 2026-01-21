use std::ops::Index;

// ============================================================================
// Value Types
// ============================================================================

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ValType {
    I32,
    I64,
    F32,
    F64,
    V128,
    FuncRef,
    ExternRef,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RefType {
    FuncRef,
    ExternRef,
}

// Legacy Ty enum for existing VM code - will eventually migrate to ValType
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Ty {
    I32,
    I64,
    F32,
    F64,
    Func(Vec<Ty>, Vec<Ty>),
}

// ============================================================================
// Function Types
// ============================================================================

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FuncType {
    pub params: Vec<ValType>,
    pub results: Vec<ValType>,
}

// ============================================================================
// Limits
// ============================================================================

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Limits {
    pub min: u32,
    pub max: Option<u32>,
}

// ============================================================================
// Table Types
// ============================================================================

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TableType {
    pub limits: Limits,
    pub elem_type: RefType,
}

// ============================================================================
// Memory Types
// ============================================================================

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MemType {
    pub limits: Limits,
}

// ============================================================================
// Global Types
// ============================================================================

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Mutability {
    Const,
    Var,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GlobalType {
    pub value_type: ValType,
    pub mutability: Mutability,
}

// ============================================================================
// External Types (for imports/exports)
// ============================================================================

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ExternType {
    Func(FuncType),
    Table(TableType),
    Memory(MemType),
    Global(GlobalType),
}

impl Ty {
    pub fn func_tys(&self) -> (&[Ty], &[Ty]) {
        match self {
            Ty::Func(args, res) => (args, res),
            _ => panic!("not a function type"),
        }
    }
}

// ============================================================================
// Indices (type-safe wrappers)
// ============================================================================

pub type TypeIdx = u32;
pub type FuncIdx = u32;
pub type TableIdx = u32;
pub type MemIdx = u32;
pub type GlobalIdx = u32;
pub type ElemIdx = u32;
pub type DataIdx = u32;
pub type LocalIdx = u32;
pub type LabelIdx = u32;

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

pub fn block_type<I: Index<usize, Output = Ty>>(ind_tys: &I, ty: &BlockType) -> Ty {
    match ty {
        BlockType::Empty => Ty::Func(vec![], vec![]),
        BlockType::Index(i) => ind_tys[*i].clone(),
        BlockType::ValTy(ty) => Ty::Func(vec![], vec![ty.clone()]),
    }
}
