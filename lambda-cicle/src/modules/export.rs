use crate::core::ast::Type;
use std::collections::HashMap;

#[derive(Debug, Clone, Default)]
pub struct Exports {
    values: HashMap<String, Type>,
    types: HashMap<String, Type>,
}

impl Exports {
    pub fn new() -> Self {
        Exports {
            values: HashMap::new(),
            types: HashMap::new(),
        }
    }

    pub fn add_value(&mut self, name: String, ty: Type) {
        self.values.insert(name, ty);
    }

    pub fn add_type(&mut self, name: String, ty: Type) {
        self.types.insert(name, ty);
    }

    pub fn get_value(&self, name: &str) -> Option<&Type> {
        self.values.get(name)
    }

    pub fn get_type(&self, name: &str) -> Option<&Type> {
        self.types.get(name)
    }

    pub fn values(&self) -> impl Iterator<Item = (&String, &Type)> {
        self.values.iter()
    }

    pub fn types(&self) -> impl Iterator<Item = (&String, &Type)> {
        self.types.iter()
    }

    pub fn from_term(term: &crate::core::ast::Term, return_type: Type) -> Self {
        let mut exports = Exports::new();

        collect_exports(term, &mut exports);

        if let crate::core::ast::Type::Arrow(_, _, ret) = return_type {
            exports.add_value("main".to_string(), *ret);
        } else {
            exports.add_value("main".to_string(), return_type);
        }

        exports
    }
}

fn collect_exports(term: &crate::core::ast::Term, exports: &mut Exports) {
    match term {
        crate::core::ast::Term::Abs {
            var, annot, body, ..
        } => {
            let fn_type = crate::core::ast::Type::arrow(
                annot.clone(),
                crate::core::ast::Multiplicity::One,
                crate::core::ast::Type::unit(),
            );
            exports.add_value(var.clone(), fn_type);
            collect_exports(body, exports);
        }
        crate::core::ast::Term::App { fun, arg } => {
            collect_exports(fun, exports);
            collect_exports(arg, exports);
        }
        crate::core::ast::Term::Let { body, .. } => {
            collect_exports(body, exports);
        }
        crate::core::ast::Term::Match { scrutinee, arms } => {
            collect_exports(scrutinee, exports);
            for arm in arms {
                collect_exports(&arm.body, exports);
            }
        }
        crate::core::ast::Term::View { scrutinee, arms } => {
            collect_exports(scrutinee, exports);
            for arm in arms {
                collect_exports(&arm.body, exports);
            }
        }
        _ => {}
    }
}

impl serde::Serialize for Exports {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut s = serializer.serialize_struct("Exports", 2)?;
        s.serialize_field("values", &self.values)?;
        s.serialize_field("types", &self.types)?;
        s.end()
    }
}

impl<'de> serde::Deserialize<'de> for Exports {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        use serde::de::{self, MapAccess, Visitor};
        use std::fmt;

        struct ExportsVisitor;

        impl<'de> Visitor<'de> for ExportsVisitor {
            type Value = Exports;

            fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                formatter.write_str("Exports")
            }

            fn visit_map<A>(self, mut map: A) -> Result<Exports, A::Error>
            where
                A: MapAccess<'de>,
            {
                let mut values = HashMap::new();
                let mut types = HashMap::new();

                while let Some(key) = map.next_key::<String>()? {
                    match key.as_str() {
                        "values" => {
                            values = map.next_value()?;
                        }
                        "types" => {
                            types = map.next_value()?;
                        }
                        _ => {
                            return Err(de::Error::custom("unknown field"));
                        }
                    }
                }

                Ok(Exports { values, types })
            }
        }

        deserializer.deserialize_struct("Exports", &["values", "types"], ExportsVisitor)
    }
}
