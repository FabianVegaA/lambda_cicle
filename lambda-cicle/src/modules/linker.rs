use super::{Module, ModuleError};
use crate::runtime::evaluator::verify_s5_prime;
use crate::traits::{check_coherence, Registry};
use std::path::{Path, PathBuf};

pub fn link(object_files: &[PathBuf], output: &PathBuf) -> Result<(), ModuleError> {
    let modules = load_modules(object_files)?;

    let registry = build_registry(&modules)?;

    check_coherence(&registry)?;

    let combined_net = combine_nets(&modules);

    verify_s5_prime(&combined_net)?;

    emit_executable(&combined_net, output)?;

    Ok(())
}

fn load_modules(paths: &[PathBuf]) -> Result<Vec<Module>, ModuleError> {
    let mut modules = Vec::new();

    for path in paths {
        let module = crate::modules::loader::load_module(path)?;
        modules.push(module);
    }

    Ok(modules)
}

fn build_registry(modules: &[Module]) -> Result<Registry, ModuleError> {
    let mut registry = Registry::new();

    for module in modules {
        for impl_ in &module.impls {
            registry.insert(impl_.clone()).map_err(|e| ModuleError {
                message: e.to_string(),
            })?;
        }
    }

    Ok(registry)
}

fn combine_nets(modules: &[Module]) -> crate::runtime::net::Net {
    let mut combined = crate::runtime::net::Net::new();

    for module in modules {
        for node in module.net.nodes() {
            combined.add_node(node.clone());
        }
        for wire in module.net.wires() {
            combined.add_wire(wire.clone());
        }
    }

    combined
}

fn emit_executable(net: &crate::runtime::net::Net, output: &Path) -> Result<(), ModuleError> {
    let bytes = serialize_net(net);

    std::fs::write(output, bytes).map_err(|e| ModuleError {
        message: format!("Failed to write output: {}", e),
    })?;

    Ok(())
}

fn serialize_net(net: &crate::runtime::net::Net) -> Vec<u8> {
    let mut bytes = Vec::new();

    bytes.extend_from_slice(b"LC01");

    let num_nodes = net.nodes().len() as u32;
    bytes.extend_from_slice(&num_nodes.to_le_bytes());

    let num_wires = net.wires().len() as u32;
    bytes.extend_from_slice(&num_wires.to_le_bytes());

    bytes
}

pub fn load_object(path: &Path) -> Result<Module, ModuleError> {
    let data = std::fs::read(path).map_err(|e| ModuleError {
        message: format!("Failed to read object: {}", e),
    })?;

    deserialize_module(&data)
}

fn deserialize_module(data: &[u8]) -> Result<Module, ModuleError> {
    if data.len() < 8 {
        return Err(ModuleError {
            message: "Invalid object file".to_string(),
        });
    }

    let magic = &data[0..4];
    if magic != b"LC01" {
        return Err(ModuleError {
            message: "Invalid magic number".to_string(),
        });
    }

    Ok(Module {
        name: "loaded".to_string(),
        exports: super::Exports::default(),
        impls: Vec::new(),
        net: crate::runtime::net::Net::new(),
    })
}
