mod operations;

pub use operations::PrimOp;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum NativeKind {
    Int,
    Float,
    Bool,
    Char,
    Unit,
}

impl NativeKind {
    pub fn all() -> &'static [NativeKind] {
        &[
            NativeKind::Int,
            NativeKind::Float,
            NativeKind::Bool,
            NativeKind::Char,
            NativeKind::Unit,
        ]
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum IOOp {
    Print,
    ReadLine,
    OpenFile,
    CloseFile,
    FileWrite,
}

impl IOOp {
    pub fn arity(&self) -> usize {
        match self {
            IOOp::Print | IOOp::ReadLine => 1,
            IOOp::OpenFile | IOOp::CloseFile | IOOp::FileWrite => 2,
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum PrimVal {
    Int(i64),
    Float(f64),
    Bool(bool),
    Char(char),
    Unit,
}

impl PrimVal {
    pub fn native_kind(&self) -> NativeKind {
        match self {
            PrimVal::Int(_) => NativeKind::Int,
            PrimVal::Float(_) => NativeKind::Float,
            PrimVal::Bool(_) => NativeKind::Bool,
            PrimVal::Char(_) => NativeKind::Char,
            PrimVal::Unit => NativeKind::Unit,
        }
    }
}

pub use operations::prim_name_to_op;

pub static INTRINSICS_TABLE: &[&str] = &[
    "prim_iadd",
    "prim_isub",
    "prim_imul",
    "prim_idiv",
    "prim_irem",
    "prim_ineg",
    "prim_fadd",
    "prim_fsub",
    "prim_fmul",
    "prim_fdiv",
    "prim_fneg",
    "prim_ieq",
    "prim_ifeq",
    "prim_igt",
    "prim_ige",
    "prim_ilt",
    "prim_ile",
    "prim_feq",
    "prim_fne",
    "prim_fgt",
    "prim_fge",
    "prim_flt",
    "prim_fle",
    "prim_not",
    "prim_and",
    "prim_or",
    "prim_chr",
    "prim_ord",
    "prim_print",
    "prim_read_line",
    "prim_open_file",
    "prim_close_file",
    "prim_file_write",
];

pub fn is_valid_primitive(name: &str) -> bool {
    INTRINSICS_TABLE.contains(&name)
}
