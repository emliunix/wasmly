use crate::types::*;

// ============================================================================
// Expression (instruction sequences)
// ============================================================================

#[derive(Debug, Clone)]
pub struct Expr {
    pub instrs: Vec<Instr>,
}

// ============================================================================
// Module Sections
// ============================================================================

// Section 1: Type Section
pub type TypeSection = Vec<FuncType>;

// Section 2: Import Section
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Import {
    pub module: String,
    pub name: String,
    pub desc: ImportDesc,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ImportDesc {
    Func(TypeIdx),
    Table(TableType),
    Memory(MemType),
    Global(GlobalType),
}

pub type ImportSection = Vec<Import>;

// Section 3: Function Section
pub type FunctionSection = Vec<TypeIdx>;

// Section 4: Table Section
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Table {
    pub table_type: TableType,
}

pub type TableSection = Vec<Table>;

// Section 5: Memory Section
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Memory {
    pub mem_type: MemType,
}

pub type MemorySection = Vec<Memory>;

// Section 6: Global Section
#[derive(Debug, Clone)]
pub struct Global {
    pub global_type: GlobalType,
    pub init_expr: Expr,
}

pub type GlobalSection = Vec<Global>;

// Section 7: Export Section
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Export {
    pub name: String,
    pub desc: ExportDesc,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ExportDesc {
    Func(FuncIdx),
    Table(TableIdx),
    Memory(MemIdx),
    Global(GlobalIdx),
}

pub type ExportSection = Vec<Export>;

// Section 8: Start Section
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Start {
    pub func_idx: FuncIdx,
}

// Section 9: Element Section
#[derive(Debug, Clone)]
pub struct Element {
    pub elem_type: RefType,
    pub init: Vec<Expr>,
    pub mode: ElemMode,
}

#[derive(Debug, Clone)]
pub enum ElemMode {
    Passive,
    Active { table: TableIdx, offset: Expr },
    Declarative,
}

pub type ElementSection = Vec<Element>;

// Section 10: Code Section
#[derive(Debug, Clone)]
pub struct Code {
    pub locals: Vec<ValType>,
    pub body: Expr,
}

pub type CodeSection = Vec<Code>;

// Section 11: Data Section
#[derive(Debug, Clone)]
pub struct Data {
    pub init: Vec<u8>,
    pub mode: DataMode,
}

#[derive(Debug, Clone)]
pub enum DataMode {
    Passive,
    Active { memory: MemIdx, offset: Expr },
}

pub type DataSection = Vec<Data>;

// Section 12: Data Count Section
pub type DataCountSection = Option<u32>;

// Section 0: Custom Section
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CustomSection {
    pub name: String,
    pub data: Vec<u8>,
}

// ============================================================================
// Module (compile-time representation)
// ============================================================================

#[derive(Debug, Clone)]
pub struct Module {
    pub types: TypeSection,
    pub imports: ImportSection,
    pub functions: FunctionSection,
    pub tables: TableSection,
    pub memories: MemorySection,
    pub globals: GlobalSection,
    pub exports: ExportSection,
    pub start: Option<Start>,
    pub elements: ElementSection,
    pub code: CodeSection,
    pub data: DataSection,
    pub data_count: DataCountSection,
    pub customs: Vec<CustomSection>,
}

impl Module {
    /// Create an empty module
    pub fn new() -> Self {
        Module {
            types: vec![],
            imports: vec![],
            functions: vec![],
            tables: vec![],
            memories: vec![],
            globals: vec![],
            exports: vec![],
            start: None,
            elements: vec![],
            code: vec![],
            data: vec![],
            data_count: None,
            customs: vec![],
        }
    }

    /// Get module imports as (module, name, type) tuples
    pub fn module_imports(&self) -> Vec<(String, String, ExternType)> {
        self.imports
            .iter()
            .map(|import| {
                let extern_type = match &import.desc {
                    ImportDesc::Func(type_idx) => {
                        ExternType::Func(self.types[*type_idx as usize].clone())
                    }
                    ImportDesc::Table(table_type) => ExternType::Table(table_type.clone()),
                    ImportDesc::Memory(mem_type) => ExternType::Memory(mem_type.clone()),
                    ImportDesc::Global(global_type) => ExternType::Global(global_type.clone()),
                };
                (import.module.clone(), import.name.clone(), extern_type)
            })
            .collect()
    }

    /// Get module exports as (name, type) tuples
    pub fn module_exports(&self) -> Vec<(String, ExternType)> {
        // TODO: need to resolve indices through index spaces
        // For now, placeholder implementation
        todo!("module_exports requires index space resolution")
    }

    /// Validate module structure (placeholder)
    pub fn validate(&self) -> Result<(), String> {
        // TODO: implement validation
        // - Function and Code sections must have same length
        // - All indices must be valid
        // - Export names must be unique
        // - Start function must have type [] -> []
        // - etc.
        todo!("validation not yet implemented")
    }
}

impl Default for Module {
    fn default() -> Self {
        Self::new()
    }
}

// ============================================================================
// Runtime Structures (Store, Addresses, Instances)
// ============================================================================

// Addresses are indices into the store
pub type FuncAddr = u32;
pub type TableAddr = u32;
pub type MemAddr = u32;
pub type GlobalAddr = u32;
pub type ElemAddr = u32;
pub type DataAddr = u32;

// Store holds all runtime instances
#[derive(Debug, Clone)]
pub struct Store {
    pub funcs: Vec<FuncInst>,
    pub tables: Vec<TableInst>,
    pub mems: Vec<MemInst>,
    pub globals: Vec<GlobalInst>,
    pub elems: Vec<ElemInst>,
    pub datas: Vec<DataInst>,
}

impl Store {
    pub fn new() -> Self {
        Store {
            funcs: vec![],
            tables: vec![],
            mems: vec![],
            globals: vec![],
            elems: vec![],
            datas: vec![],
        }
    }
}

impl Default for Store {
    fn default() -> Self {
        Self::new()
    }
}

// Function Instance
#[derive(Debug, Clone)]
pub enum FuncInst {
    Local {
        func_type: FuncType,
        module: ModuleAddr,
        code: Code,
    },
    Host {
        func_type: FuncType,
        host_func: HostFunc,
    },
}

pub type ModuleAddr = u32;

#[derive(Debug, Clone)]
pub struct HostFunc {
    // Placeholder for host function pointer/callback
    // In real implementation, this would be a function pointer or closure
    pub name: String,
}

// Table Instance
#[derive(Debug, Clone)]
pub struct TableInst {
    pub table_type: TableType,
    pub elem: Vec<Ref>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Ref {
    Null(RefType),
    Func(FuncAddr),
    Extern(u32), // External reference (opaque)
}

// Memory Instance
#[derive(Debug, Clone)]
pub struct MemInst {
    pub mem_type: MemType,
    pub data: Vec<u8>,
}

// Global Instance
#[derive(Debug, Clone)]
pub struct GlobalInst {
    pub global_type: GlobalType,
    pub value: Val,
}

// Element Instance
#[derive(Debug, Clone)]
pub struct ElemInst {
    pub elem_type: RefType,
    pub elem: Vec<Ref>,
}

// Data Instance
#[derive(Debug, Clone)]
pub struct DataInst {
    pub data: Vec<u8>,
}

// Module Instance (runtime representation)
#[derive(Debug, Clone)]
pub struct ModuleInst {
    pub types: Vec<FuncType>,
    pub func_addrs: Vec<FuncAddr>,
    pub table_addrs: Vec<TableAddr>,
    pub mem_addrs: Vec<MemAddr>,
    pub global_addrs: Vec<GlobalAddr>,
    pub elem_addrs: Vec<ElemAddr>,
    pub data_addrs: Vec<DataAddr>,
    pub exports: Vec<ExportInst>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ExportInst {
    pub name: String,
    pub value: ExternVal,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ExternVal {
    Func(FuncAddr),
    Table(TableAddr),
    Memory(MemAddr),
    Global(GlobalAddr),
}

// ============================================================================
// Embedder API (placeholder implementations)
// ============================================================================

/// Decode and validate a WASM module from bytes
pub fn module_decode(_bytes: &[u8]) -> Result<Module, String> {
    // TODO: implement binary parser
    todo!("module_decode not yet implemented")
}

/// Instantiate a module with given imports
pub fn module_instantiate(
    _store: &mut Store,
    _module: &Module,
    _imports: &[ExternVal],
) -> Result<ModuleInst, String> {
    // TODO: implement instantiation
    // - Allocate functions, tables, memories, globals
    // - Initialize tables with element segments
    // - Initialize memories with data segments
    // - Build export map
    // - Execute start function if present
    todo!("module_instantiate not yet implemented")
}

/// Invoke a function with given arguments
pub fn func_invoke(
    _store: &mut Store,
    _func_addr: FuncAddr,
    _args: &[Val],
) -> Result<Vec<Val>, String> {
    // TODO: implement function invocation
    // - Set up stack frame
    // - Execute instructions
    // - Return results
    todo!("func_invoke not yet implemented")
}

/// Get function type
pub fn func_type(store: &Store, func_addr: FuncAddr) -> &FuncType {
    match &store.funcs[func_addr as usize] {
        FuncInst::Local { func_type, .. } => func_type,
        FuncInst::Host { func_type, .. } => func_type,
    }
}

/// Allocate a host function
pub fn func_alloc_host(store: &mut Store, func_type: FuncType, host_func: HostFunc) -> FuncAddr {
    let addr = store.funcs.len() as u32;
    store.funcs.push(FuncInst::Host {
        func_type,
        host_func,
    });
    addr
}

/// Read from table
pub fn table_read(store: &Store, table_addr: TableAddr, index: u32) -> Result<Ref, String> {
    let table = &store.tables[table_addr as usize];
    table
        .elem
        .get(index as usize)
        .cloned()
        .ok_or_else(|| "table index out of bounds".to_string())
}

/// Write to table
pub fn table_write(
    store: &mut Store,
    table_addr: TableAddr,
    index: u32,
    val: Ref,
) -> Result<(), String> {
    let table = &mut store.tables[table_addr as usize];
    if (index as usize) < table.elem.len() {
        table.elem[index as usize] = val;
        Ok(())
    } else {
        Err("table index out of bounds".to_string())
    }
}

/// Get table size
pub fn table_size(store: &Store, table_addr: TableAddr) -> u32 {
    store.tables[table_addr as usize].elem.len() as u32
}

/// Grow table
pub fn table_grow(
    _store: &mut Store,
    _table_addr: TableAddr,
    _delta: u32,
    _init: Ref,
) -> Result<u32, String> {
    // TODO: implement table grow with limit checking
    todo!("table_grow not yet implemented")
}

/// Read from memory
pub fn mem_read(store: &Store, mem_addr: MemAddr, offset: u32, len: u32) -> Result<&[u8], String> {
    let mem = &store.mems[mem_addr as usize];
    let start = offset as usize;
    let end = start + len as usize;
    mem.data
        .get(start..end)
        .ok_or_else(|| "memory access out of bounds".to_string())
}

/// Write to memory
pub fn mem_write(
    store: &mut Store,
    mem_addr: MemAddr,
    offset: u32,
    data: &[u8],
) -> Result<(), String> {
    let mem = &mut store.mems[mem_addr as usize];
    let start = offset as usize;
    let end = start + data.len();
    if end <= mem.data.len() {
        mem.data[start..end].copy_from_slice(data);
        Ok(())
    } else {
        Err("memory access out of bounds".to_string())
    }
}

/// Get memory size in pages (64KiB each)
pub fn mem_size(store: &Store, mem_addr: MemAddr) -> u32 {
    let mem = &store.mems[mem_addr as usize];
    (mem.data.len() / 65536) as u32
}

/// Grow memory by delta pages
pub fn mem_grow(_store: &mut Store, _mem_addr: MemAddr, _delta: u32) -> Result<u32, String> {
    // TODO: implement memory grow with limit checking
    todo!("mem_grow not yet implemented")
}

/// Read global value
pub fn global_read(store: &Store, global_addr: GlobalAddr) -> Val {
    store.globals[global_addr as usize].value.clone()
}

/// Write global value
pub fn global_write(store: &mut Store, global_addr: GlobalAddr, val: Val) -> Result<(), String> {
    let global = &mut store.globals[global_addr as usize];
    if global.global_type.mutability == Mutability::Var {
        global.value = val;
        Ok(())
    } else {
        Err("global is immutable".to_string())
    }
}
