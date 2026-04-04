use crate::core::ast::{Decl, Type, Visibility};
use std::collections::HashMap;

#[derive(Debug, Clone, PartialEq)]
pub struct ExportEntry {
    pub ty: Type,
    pub visibility: Visibility,
    pub transparent: bool,
}

impl serde::Serialize for ExportEntry {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut s = serializer.serialize_struct("ExportEntry", 3)?;
        s.serialize_field("ty", &self.ty)?;
        s.serialize_field("visibility", &self.visibility)?;
        s.serialize_field("transparent", &self.transparent)?;
        s.end()
    }
}

impl<'de> serde::Deserialize<'de> for ExportEntry {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        use serde::de::{self, MapAccess, Visitor};
        use std::fmt;

        struct ExportEntryVisitor;

        impl<'de> Visitor<'de> for ExportEntryVisitor {
            type Value = ExportEntry;

            fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                formatter.write_str("ExportEntry")
            }

            fn visit_map<A>(self, mut map: A) -> Result<ExportEntry, A::Error>
            where
                A: MapAccess<'de>,
            {
                let mut ty = None;
                let mut visibility = None;
                let mut transparent = None;

                while let Some(key) = map.next_key::<String>()? {
                    match key.as_str() {
                        "ty" => {
                            ty = Some(map.next_value()?);
                        }
                        "visibility" => {
                            visibility = Some(map.next_value()?);
                        }
                        "transparent" => {
                            transparent = Some(map.next_value()?);
                        }
                        _ => {
                            return Err(de::Error::custom("unknown field"));
                        }
                    }
                }

                Ok(ExportEntry {
                    ty: ty.unwrap_or(Type::unit()),
                    visibility: visibility.unwrap_or(Visibility::Private),
                    transparent: transparent.unwrap_or(false),
                })
            }
        }

        deserializer.deserialize_struct(
            "ExportEntry",
            &["ty", "visibility", "transparent"],
            ExportEntryVisitor,
        )
    }
}

#[derive(Debug, Clone, Default)]
pub struct Exports {
    values: HashMap<String, ExportEntry>,
    types: HashMap<String, ExportEntry>,
    traits: HashMap<String, ExportEntry>,
}

impl Exports {
    pub fn new() -> Self {
        Exports {
            values: HashMap::new(),
            types: HashMap::new(),
            traits: HashMap::new(),
        }
    }

    pub fn add_value(&mut self, name: String, ty: Type, visibility: Visibility) {
        self.values.insert(
            name,
            ExportEntry {
                ty,
                visibility,
                transparent: false,
            },
        );
    }

    pub fn add_type(&mut self, name: String, ty: Type, visibility: Visibility, transparent: bool) {
        self.types.insert(
            name,
            ExportEntry {
                ty,
                visibility,
                transparent,
            },
        );
    }

    pub fn add_trait(&mut self, name: String, ty: Type, visibility: Visibility) {
        self.traits.insert(
            name,
            ExportEntry {
                ty,
                visibility,
                transparent: false,
            },
        );
    }

    pub fn get_value(&self, name: &str) -> Option<&ExportEntry> {
        self.values.get(name)
    }

    pub fn get_type(&self, name: &str) -> Option<&ExportEntry> {
        self.types.get(name)
    }

    pub fn get_trait(&self, name: &str) -> Option<&ExportEntry> {
        self.traits.get(name)
    }

    pub fn public_values(&self) -> impl Iterator<Item = (&String, &ExportEntry)> {
        self.values
            .iter()
            .filter(|(_, e)| matches!(e.visibility, Visibility::Public))
    }

    pub fn public_types(&self) -> impl Iterator<Item = (&String, &ExportEntry)> {
        self.types
            .iter()
            .filter(|(_, e)| matches!(e.visibility, Visibility::Public))
    }

    pub fn public_traits(&self) -> impl Iterator<Item = (&String, &ExportEntry)> {
        self.traits
            .iter()
            .filter(|(_, e)| matches!(e.visibility, Visibility::Public))
    }

    pub fn from_decl(decls: &[Decl]) -> Self {
        let mut exports = Exports::new();

        for decl in decls {
            match decl {
                Decl::TypeDecl {
                    name,
                    visibility,
                    ty,
                    transparent,
                    ..
                } => {
                    if matches!(visibility, Visibility::Public) {
                        exports.add_type(
                            name.clone(),
                            ty.clone(),
                            visibility.clone(),
                            *transparent,
                        );
                    }
                }
                Decl::ValDecl {
                    name,
                    visibility,
                    ty,
                    ..
                } => {
                    if matches!(visibility, Visibility::Public) {
                        exports.add_value(name.clone(), ty.clone(), visibility.clone());
                    }
                }
                Decl::TraitDecl {
                    name,
                    visibility,
                    params,
                    ..
                } => {
                    if matches!(visibility, Visibility::Public) {
                        let trait_type = if params.is_empty() {
                            Type::unit()
                        } else {
                            Type::inductive(name.clone(), vec![])
                        };
                        exports.add_trait(name.clone(), trait_type, visibility.clone());
                    }
                }
                Decl::ImplDecl {
                    ty,
                    trait_name: _,
                    methods,
                    ..
                } => {
                    // Export trait method implementations as values (§8.5)
                    // This allows direct calls like "add 3 5" instead of requiring
                    // explicit trait resolution syntax
                    for method in methods {
                        // Only export if the implementation type is concrete (not polymorphic)
                        if !ty.is_polymorphic() {
                            // Export as: <method_name> : <method_type>
                            // e.g., "add" for Int becomes: add : Int -> Int -> Int = prim_iadd
                            exports.add_value(
                                method.name.to_string(),
                                method.ty.clone(),
                                Visibility::Public,
                            );
                        }
                    }
                }
                Decl::UseDecl { .. } => {
                    // use declarations are not exports
                }
                Decl::NoPrelude => {
                    // no_prelude is not an export
                }
            }
        }

        exports
    }

    #[allow(dead_code)]
    pub fn from_term(term: &crate::core::ast::Term, return_type: Type) -> Self {
        let mut exports = Exports::new();

        collect_exports(term, &mut exports);

        if let crate::core::ast::Type::Arrow(_, _, ret) = return_type {
            exports.add_value("main".to_string(), *ret, Visibility::Public);
        } else {
            exports.add_value("main".to_string(), return_type, Visibility::Public);
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
            exports.add_value(var.clone(), fn_type, Visibility::Private);
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
        let mut s = serializer.serialize_struct("Exports", 3)?;
        s.serialize_field("values", &self.values)?;
        s.serialize_field("types", &self.types)?;
        s.serialize_field("traits", &self.traits)?;
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
                let mut traits = HashMap::new();

                while let Some(key) = map.next_key::<String>()? {
                    match key.as_str() {
                        "values" => {
                            values = map.next_value()?;
                        }
                        "types" => {
                            types = map.next_value()?;
                        }
                        "traits" => {
                            traits = map.next_value()?;
                        }
                        _ => {
                            return Err(de::Error::custom("unknown field"));
                        }
                    }
                }

                Ok(Exports {
                    values,
                    types,
                    traits,
                })
            }
        }

        deserializer.deserialize_struct("Exports", &["values", "types", "traits"], ExportsVisitor)
    }
}
