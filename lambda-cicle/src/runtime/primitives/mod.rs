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
    Println,
    EPrint,
    EPrintln,
    ReadLine,
    Open,
    Close,
    Read,
    Write,
}

impl IOOp {
    pub fn arity(&self) -> usize {
        match self {
            IOOp::Print
            | IOOp::Println
            | IOOp::EPrint
            | IOOp::EPrintln
            | IOOp::Close
            | IOOp::Read => 1,
            IOOp::Open | IOOp::Write => 2,
            IOOp::ReadLine => 0, // nullary - no value argument, only IO_token
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
    String(String),
}

impl PrimVal {
    pub fn native_kind(&self) -> NativeKind {
        match self {
            PrimVal::Int(_) => NativeKind::Int,
            PrimVal::Float(_) => NativeKind::Float,
            PrimVal::Bool(_) => NativeKind::Bool,
            PrimVal::Char(_) => NativeKind::Char,
            PrimVal::Unit => NativeKind::Unit,
            PrimVal::String(_) => NativeKind::Unit, // String not yet native
        }
    }
}

pub use operations::prim_name_to_io_op;
pub use operations::prim_name_to_op;

pub static INTRINSICS_TABLE: &[&str] = &[
    // Integer arithmetic
    "prim_iadd",
    "prim_isub",
    "prim_imul",
    "prim_idiv",
    "prim_irem",
    "prim_ineg",
    // Float arithmetic
    "prim_fadd",
    "prim_fsub",
    "prim_fmul",
    "prim_fdiv",
    "prim_fneg",
    // Integer comparison
    "prim_ieq",
    "prim_ifeq",
    "prim_igt",
    "prim_ige",
    "prim_ilt",
    "prim_ile",
    // Float comparison
    "prim_feq",
    "prim_fne",
    "prim_fgt",
    "prim_fge",
    "prim_flt",
    "prim_fle",
    // Boolean
    "prim_not",
    "prim_and",
    "prim_or",
    // Char
    "prim_chr",
    "prim_ord",
    // Conversion to string
    "prim_int_to_string",
    "prim_float_to_string",
    "prim_char_to_string",
    // IO operations (from §16.3.2)
    "prim_io_print",
    "prim_io_println",
    "prim_io_eprint",
    "prim_io_eprintln",
    "prim_io_read_line",
    "prim_io_open",
    "prim_io_close",
    "prim_io_read",
    "prim_io_write",
];

pub fn is_valid_primitive(name: &str) -> bool {
    INTRINSICS_TABLE.contains(&name)
}
