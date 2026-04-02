use crate::core::ast::types::{Multiplicity, Type};
use crate::core::multiplicity::semiring::Quantity;
use crate::core::multiplicity::BorrowContextMix;
use hashbrown::HashMap;

#[derive(Debug, Clone)]
pub struct ConstructorInfo {
    pub type_name: String,
    pub field_types: Vec<Type>,
    pub result_type: Type,
}

#[derive(Debug, Clone, Default)]
pub struct TypeContext {
    bindings: HashMap<String, (Multiplicity, Type)>,
    constructors: HashMap<String, ConstructorInfo>,
}

impl TypeContext {
    pub fn new() -> TypeContext {
        let mut ctx = TypeContext {
            bindings: HashMap::new(),
            constructors: HashMap::new(),
        };

        // Register primitive operations as built-in bindings
        ctx.register_primitives();
        ctx
    }

    fn register_primitives(&mut self) {
        // Arithmetic primitives for Int
        let int_binop_ty = Type::arrow(
            Type::int(),
            Multiplicity::One,
            Type::arrow(Type::int(), Multiplicity::One, Type::int()),
        );

        self.bindings.insert(
            "prim_iadd".to_string(),
            (Multiplicity::Omega, int_binop_ty.clone()),
        );
        self.bindings.insert(
            "prim_isub".to_string(),
            (Multiplicity::Omega, int_binop_ty.clone()),
        );
        self.bindings.insert(
            "prim_imul".to_string(),
            (Multiplicity::Omega, int_binop_ty.clone()),
        );

        // Comparison primitives for Int (return ADT Bool, not native bool)
        let bool_ty = Type::inductive("Bool".to_string(), vec![]);
        let int_cmp_ty = Type::arrow(
            Type::int(),
            Multiplicity::One,
            Type::arrow(Type::int(), Multiplicity::One, bool_ty.clone()),
        );

        self.bindings.insert(
            "prim_ieq".to_string(),
            (Multiplicity::Omega, int_cmp_ty.clone()),
        );
        self.bindings.insert(
            "prim_igt".to_string(),
            (Multiplicity::Omega, int_cmp_ty.clone()),
        );
        self.bindings.insert(
            "prim_ilt".to_string(),
            (Multiplicity::Omega, int_cmp_ty.clone()),
        );

        // Division with error handling
        let division_by_zero = Type::inductive("DivisionByZero".to_string(), vec![]);
        let int_div_ty = Type::arrow(
            Type::int(),
            Multiplicity::One,
            Type::arrow(
                Type::int(),
                Multiplicity::One,
                Type::inductive("Result".to_string(), vec![Type::int(), division_by_zero]),
            ),
        );

        self.bindings.insert(
            "prim_idiv".to_string(),
            (Multiplicity::Omega, int_div_ty.clone()),
        );
        self.bindings
            .insert("prim_irem".to_string(), (Multiplicity::Omega, int_div_ty));

        // Unary operations for Int
        let int_unary_ty = Type::arrow(Type::int(), Multiplicity::One, Type::int());
        self.bindings
            .insert("prim_ineg".to_string(), (Multiplicity::Omega, int_unary_ty));

        // Hash for Int
        let int_hash_ty = Type::arrow(Type::int(), Multiplicity::One, Type::int());
        self.bindings
            .insert("prim_ihash".to_string(), (Multiplicity::Omega, int_hash_ty));

        // Bool primitives (Bool is an ADT type, not native)
        let bool_ty = Type::inductive("Bool".to_string(), vec![]);

        let bool_binop_ty = Type::arrow(
            bool_ty.clone(),
            Multiplicity::One,
            Type::arrow(bool_ty.clone(), Multiplicity::One, bool_ty.clone()),
        );

        self.bindings.insert(
            "prim_and".to_string(),
            (Multiplicity::Omega, bool_binop_ty.clone()),
        );
        self.bindings.insert(
            "prim_or".to_string(),
            (Multiplicity::Omega, bool_binop_ty.clone()),
        );

        let bool_unary_ty = Type::arrow(bool_ty.clone(), Multiplicity::One, bool_ty.clone());
        self.bindings
            .insert("prim_not".to_string(), (Multiplicity::Omega, bool_unary_ty));

        let bool_cmp_ty = Type::arrow(
            bool_ty.clone(),
            Multiplicity::One,
            Type::arrow(bool_ty.clone(), Multiplicity::One, bool_ty.clone()),
        );
        self.bindings
            .insert("prim_beq".to_string(), (Multiplicity::Omega, bool_cmp_ty));

        let bool_hash_ty = Type::arrow(bool_ty.clone(), Multiplicity::One, Type::int());
        self.bindings.insert(
            "prim_bhash".to_string(),
            (Multiplicity::Omega, bool_hash_ty),
        );

        // Float primitives
        let float_binop_ty = Type::arrow(
            Type::float(),
            Multiplicity::One,
            Type::arrow(Type::float(), Multiplicity::One, Type::float()),
        );

        self.bindings.insert(
            "prim_fadd".to_string(),
            (Multiplicity::Omega, float_binop_ty.clone()),
        );
        self.bindings.insert(
            "prim_fsub".to_string(),
            (Multiplicity::Omega, float_binop_ty.clone()),
        );
        self.bindings.insert(
            "prim_fmul".to_string(),
            (Multiplicity::Omega, float_binop_ty.clone()),
        );
        self.bindings.insert(
            "prim_fdiv".to_string(),
            (Multiplicity::Omega, float_binop_ty.clone()),
        );

        let float_cmp_ty = Type::arrow(
            Type::float(),
            Multiplicity::One,
            Type::arrow(Type::float(), Multiplicity::One, bool_ty.clone()),
        );

        self.bindings.insert(
            "prim_feq".to_string(),
            (Multiplicity::Omega, float_cmp_ty.clone()),
        );
        self.bindings.insert(
            "prim_fgt".to_string(),
            (Multiplicity::Omega, float_cmp_ty.clone()),
        );

        let float_unary_ty = Type::arrow(Type::float(), Multiplicity::One, Type::float());
        self.bindings.insert(
            "prim_fneg".to_string(),
            (Multiplicity::Omega, float_unary_ty),
        );

        // Char primitives
        let char_cmp_ty = Type::arrow(
            Type::char(),
            Multiplicity::One,
            Type::arrow(Type::char(), Multiplicity::One, Type::bool()),
        );

        self.bindings.insert(
            "prim_ceq".to_string(),
            (Multiplicity::Omega, char_cmp_ty.clone()),
        );

        self.bindings
            .insert("prim_cord".to_string(), (Multiplicity::Omega, char_cmp_ty));

        let char_hash_ty = Type::arrow(Type::char(), Multiplicity::One, Type::int());
        self.bindings.insert(
            "prim_chash".to_string(),
            (Multiplicity::Omega, char_hash_ty),
        );
    }

    pub fn extend(&self, var: String, mult: Multiplicity, ty: Type) -> TypeContext {
        let mut bindings = self.bindings.clone();
        bindings.insert(var, (mult, ty));
        TypeContext {
            bindings,
            constructors: self.constructors.clone(),
        }
    }

    pub fn get(&self, var: &str) -> Option<&(Multiplicity, Type)> {
        self.bindings.get(var)
    }

    pub fn contains(&self, var: &str) -> bool {
        self.bindings.contains_key(var)
    }

    pub fn get_constructor(&self, name: &str) -> Option<&ConstructorInfo> {
        self.constructors.get(name)
    }

    pub fn register_constructor(&self, name: String, info: ConstructorInfo) -> TypeContext {
        let mut constructors = self.constructors.clone();
        constructors.insert(name, info);
        TypeContext {
            bindings: self.bindings.clone(),
            constructors,
        }
    }

    pub fn is_empty(&self) -> bool {
        self.bindings.is_empty()
    }

    pub fn len(&self) -> usize {
        self.bindings.len()
    }

    pub fn add(&self, other: &TypeContext) -> Result<TypeContext, BorrowContextMix> {
        let mut result = self.bindings.clone();

        for (var, (mult2, ty2)) in &other.bindings {
            match result.get(var) {
                Some((mult1, _)) => {
                    let new_mult = add_multiplicity(mult1, mult2)?;
                    result.insert(var.clone(), (new_mult, ty2.clone()));
                }
                None => {
                    result.insert(var.clone(), (mult2.clone(), ty2.clone()));
                }
            }
        }

        Ok(TypeContext {
            bindings: result,
            constructors: self.constructors.clone(),
        })
    }

    pub fn scale(&self, q: Quantity) -> Result<TypeContext, BorrowContextMix> {
        let mut result = HashMap::new();

        for (var, (mult, ty)) in &self.bindings {
            let new_mult = match mult {
                Multiplicity::Borrow => return Err(BorrowContextMix::CannotScaleBorrow),
                Multiplicity::Zero => Multiplicity::Zero,
                Multiplicity::One => match q {
                    Quantity::Zero => Multiplicity::Zero,
                    Quantity::One => Multiplicity::One,
                    Quantity::Omega => Multiplicity::Omega,
                },
                Multiplicity::Omega => Multiplicity::Omega,
            };
            result.insert(var.clone(), (new_mult, ty.clone()));
        }

        Ok(TypeContext {
            bindings: result,
            constructors: self.constructors.clone(),
        })
    }

    pub fn weaken(&self, var: String, ty: Type) -> TypeContext {
        self.extend(var, Multiplicity::Zero, ty)
    }
}

fn add_multiplicity(
    m1: &Multiplicity,
    m2: &Multiplicity,
) -> Result<Multiplicity, BorrowContextMix> {
    match (m1, m2) {
        (Multiplicity::Borrow, _) | (_, Multiplicity::Borrow) => {
            Err(BorrowContextMix::CannotMixBorrowWithQuantity)
        }
        (Multiplicity::Zero, m) | (m, Multiplicity::Zero) => Ok(m.clone()),
        (Multiplicity::One, Multiplicity::One) => Ok(Multiplicity::Omega),
        (Multiplicity::One, Multiplicity::Omega) | (Multiplicity::Omega, Multiplicity::One) => {
            Ok(Multiplicity::Omega)
        }
        (Multiplicity::Omega, Multiplicity::Omega) => Ok(Multiplicity::Omega),
    }
}
