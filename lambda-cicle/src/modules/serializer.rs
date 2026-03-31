use crate::modules::{Exports, Module, ModuleError};
use crate::runtime::net::Net;
use std::collections::HashMap;

const MAGIC: &[u8] = b"LC10";

#[derive(Debug, Clone)]
pub struct SerializedModule {
    pub module_name: String,
    pub type_section: Vec<TypeEntry>,
    pub trait_section: Vec<TraitEntry>,
    pub net_section: NetSection,
    pub export_table: HashMap<String, ExportLocation>,
    pub debug_section: DebugSection,
    pub content_hash: [u8; 32],
}

#[derive(Debug, Clone)]
pub struct TypeEntry {
    pub name: String,
    pub kind: TypeKind,
    pub constructors: Vec<Constructor>,
    pub multiplicity: String,
}

#[derive(Debug, Clone)]
pub enum TypeKind {
    Opaque,
    Transparent,
}

#[derive(Debug, Clone)]
pub struct Constructor {
    pub name: String,
    pub fields: Vec<String>,
}

#[derive(Debug, Clone)]
pub struct TraitEntry {
    pub name: String,
    pub methods: Vec<MethodSignature>,
    pub impls: Vec<ImplBlock>,
}

#[derive(Debug, Clone)]
pub struct MethodSignature {
    pub name: String,
    pub ty: String,
}

#[derive(Debug, Clone)]
pub struct ImplBlock {
    pub for_type: String,
    pub methods: Vec<MethodDef>,
}

#[derive(Debug, Clone)]
pub struct MethodDef {
    pub name: String,
    pub body_offset: u32,
}

#[derive(Debug, Clone)]
pub struct NetSection {
    pub nodes: Vec<NodeData>,
    pub wires: Vec<WireData>,
}

#[derive(Debug, Clone)]
pub struct NodeData {
    pub agent: u8,
    pub num_ports: u8,
    pub name: Option<String>,
}

#[derive(Debug, Clone)]
pub struct WireData {
    pub source_node: u32,
    pub source_port: u8,
    pub target_node: u32,
    pub target_port: u8,
}

#[derive(Debug, Clone)]
pub struct ExportLocation {
    pub section: String,
    pub offset: u32,
}

#[derive(Debug, Clone, Default)]
pub struct DebugSection {
    pub source_positions: HashMap<String, (u32, u32)>,
    pub original_names: HashMap<String, String>,
}

pub fn serialize_module(module: &Module) -> Result<Vec<u8>, ModuleError> {
    let mut buf = Vec::new();

    buf.extend_from_slice(MAGIC);

    let type_data = serialize_types_to_bytes(&module.exports);
    buf.extend_from_slice(&type_data.len().to_le_bytes());
    buf.extend_from_slice(&type_data);

    let trait_data = serialize_traits_to_bytes(&module.impls);
    buf.extend_from_slice(&trait_data.len().to_le_bytes());
    buf.extend_from_slice(&trait_data);

    let net_data = serialize_net_to_bytes(&module.net);
    buf.extend_from_slice(&net_data.len().to_le_bytes());
    buf.extend_from_slice(&net_data);

    let export_data = serialize_exports_to_bytes(&module.exports);
    buf.extend_from_slice(&export_data.len().to_le_bytes());
    buf.extend_from_slice(&export_data);

    let debug_data = serialize_debug_to_bytes(&module.name);
    buf.extend_from_slice(&debug_data.len().to_le_bytes());
    buf.extend_from_slice(&debug_data);

    Ok(buf)
}

fn serialize_types_to_bytes(exports: &Exports) -> Vec<u8> {
    let mut buf = Vec::new();
    let count = exports.public_types().count() as u32;
    buf.extend_from_slice(&count.to_le_bytes());

    for (name, entry) in exports.public_types() {
        let name_len = name.len() as u32;
        buf.extend_from_slice(&name_len.to_le_bytes());
        buf.extend_from_slice(name.as_bytes());
        buf.push(if entry.transparent { 1 } else { 0 });
    }

    buf
}

fn serialize_traits_to_bytes(impls: &[crate::traits::Implementation]) -> Vec<u8> {
    let mut buf = Vec::new();
    let count = impls.len() as u32;
    buf.extend_from_slice(&count.to_le_bytes());

    for imp in impls {
        let trait_name = &imp.trait_name.0;
        let name_len = trait_name.len() as u32;
        buf.extend_from_slice(&name_len.to_le_bytes());
        buf.extend_from_slice(trait_name.as_bytes());

        let method_count = imp.methods.len() as u32;
        buf.extend_from_slice(&method_count.to_le_bytes());
    }

    buf
}

fn serialize_net_to_bytes(net: &Net) -> Vec<u8> {
    let mut buf = Vec::new();

    let node_count = u32::try_from(net.nodes().len()).unwrap_or(0);
    buf.extend_from_slice(&node_count.to_le_bytes());

    for node in net.nodes() {
        let agent_byte: u8 = match &node.agent {
            crate::runtime::net::Agent::Lambda => 0,
            crate::runtime::net::Agent::App => 1,
            crate::runtime::net::Agent::Delta => 2,
            crate::runtime::net::Agent::Epsilon => 3,
            crate::runtime::net::Agent::Constructor(name) => {
                buf.push(4);
                buf.extend_from_slice(&(name.len() as u32).to_le_bytes());
                buf.extend_from_slice(name.as_bytes());
                4
            }
            crate::runtime::net::Agent::Prim(_) => {
                buf.push(5);
                5
            }
            crate::runtime::net::Agent::PrimVal(_) => {
                buf.push(6);
                6
            }
            crate::runtime::net::Agent::PrimIO(_) => {
                buf.push(7);
                7
            }
            crate::runtime::net::Agent::IOToken => {
                buf.push(8);
                8
            }
        };
        if agent_byte < 4 {
            buf.push(agent_byte);
        }
        buf.push(u8::try_from(node.num_ports()).unwrap_or(0));
    }

    let wire_count = u32::try_from(net.wires().len()).unwrap_or(0);
    buf.extend_from_slice(&wire_count.to_le_bytes());

    for wire in net.wires() {
        buf.extend_from_slice(&(wire.source.node.0 as u32).to_le_bytes());
        buf.push(u8::try_from(wire.source.index.0).unwrap_or(0));
        buf.extend_from_slice(&(wire.target.node.0 as u32).to_le_bytes());
        buf.push(u8::try_from(wire.target.index.0).unwrap_or(0));
    }

    buf
}

fn serialize_exports_to_bytes(exports: &Exports) -> Vec<u8> {
    let mut buf = Vec::new();

    let value_count = exports.public_values().count() as u32;
    buf.extend_from_slice(&value_count.to_le_bytes());

    for (name, _) in exports.public_values() {
        buf.extend_from_slice(&(name.len() as u32).to_le_bytes());
        buf.extend_from_slice(name.as_bytes());
    }

    let type_count = exports.public_types().count() as u32;
    buf.extend_from_slice(&type_count.to_le_bytes());

    for (name, entry) in exports.public_types() {
        buf.extend_from_slice(&(name.len() as u32).to_le_bytes());
        buf.extend_from_slice(name.as_bytes());
        buf.push(if entry.transparent { 1 } else { 0 });
    }

    buf
}

fn serialize_debug_to_bytes(module_name: &str) -> Vec<u8> {
    let mut buf = Vec::new();
    buf.extend_from_slice(&(module_name.len() as u32).to_le_bytes());
    buf.extend_from_slice(module_name.as_bytes());
    buf
}

pub fn deserialize_module(data: &[u8]) -> Result<Module, ModuleError> {
    if data.len() < 4 {
        return Err(ModuleError {
            message: "Invalid object file: too short".to_string(),
        });
    }

    let magic = &data[0..4];
    if magic != MAGIC {
        return Err(ModuleError {
            message: format!("Invalid magic number: {:x?}", magic),
        });
    }

    Ok(Module {
        name: "deserialized".to_string(),
        exports: Exports::new(),
        impls: Vec::new(),
        net: Net::new(),
    })
}

pub fn get_export_hash(exports: &Exports) -> [u8; 32] {
    use std::collections::hash_map::DefaultHasher;
    use std::hash::{Hash, Hasher};

    let mut hasher = DefaultHasher::new();

    for (name, entry) in exports.public_values() {
        name.hash(&mut hasher);
        format!("{:?}", entry.ty).hash(&mut hasher);
    }

    for (name, entry) in exports.public_types() {
        name.hash(&mut hasher);
        entry.transparent.hash(&mut hasher);
    }

    let hash = hasher.finish();
    let mut result = [0u8; 32];
    result[..8].copy_from_slice(&hash.to_le_bytes());
    result
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::modules::Exports;

    #[test]
    fn test_serialize_basic_module() {
        let module = Module {
            name: "Test".to_string(),
            exports: Exports::new(),
            impls: Vec::new(),
            net: Net::new(),
        };

        let data = serialize_module(&module);
        assert!(data.is_ok());
    }

    #[test]
    fn test_roundtrip() {
        let module = Module {
            name: "Test".to_string(),
            exports: Exports::new(),
            impls: Vec::new(),
            net: Net::new(),
        };

        let data = serialize_module(&module).unwrap();
        let result = deserialize_module(&data);
        assert!(result.is_ok());
    }
}
