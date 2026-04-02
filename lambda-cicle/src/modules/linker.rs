use super::{deserialize_module, serialize_module, Module, ModuleError};
use crate::runtime::evaluator::verify_s5_prime;
use std::collections::HashMap;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone)]
pub struct GlobalRegistry {
    impls: HashMap<(String, String), (crate::traits::Implementation, String)>,
}

impl GlobalRegistry {
    pub fn new() -> Self {
        GlobalRegistry {
            impls: HashMap::new(),
        }
    }

    pub fn insert(
        &mut self,
        trait_name: String,
        ty: String,
        impl_block: crate::traits::Implementation,
        module: String,
    ) -> Result<(), ModuleError> {
        let key = (trait_name.clone(), ty.clone());
        if self.impls.contains_key(&key) {
            return Err(ModuleError {
                message: format!(
                    "CoherenceViolation: duplicate impl of {} for {} from modules",
                    trait_name, ty
                ),
            });
        }
        self.impls.insert(key, (impl_block, module));
        Ok(())
    }
}

impl Default for GlobalRegistry {
    fn default() -> Self {
        Self::new()
    }
}

pub fn link(object_files: &[PathBuf], output: &PathBuf) -> Result<(), ModuleError> {
    let modules = load_object_files(object_files)?;

    let registry = build_global_registry(&modules)?;

    check_global_coherence(&registry)?;

    let combined_net = combine_nets(&modules);

    verify_s5_prime(&combined_net)?;

    emit_executable(&combined_net, output)?;

    Ok(())
}

fn load_object_files(paths: &[PathBuf]) -> Result<Vec<Module>, ModuleError> {
    let mut modules = Vec::new();

    for path in paths {
        let data = std::fs::read(path).map_err(|e| ModuleError {
            message: format!("Failed to read file: {}", e),
        })?;

        let module = deserialize_module(&data)?;
        modules.push(module);
    }

    Ok(modules)
}

fn build_global_registry(modules: &[Module]) -> Result<GlobalRegistry, ModuleError> {
    let mut registry = GlobalRegistry::new();

    for module in modules {
        for impl_block in &module.impls {
            registry.insert(
                impl_block.trait_name.0.clone(),
                format!("{:?}", impl_block.for_type),
                impl_block.clone(),
                module.name.clone(),
            )?;
        }
    }

    Ok(registry)
}

fn check_global_coherence(registry: &GlobalRegistry) -> Result<(), ModuleError> {
    if registry.impls.len() > 1 {
        let mut seen: std::collections::HashSet<(String, String)> =
            std::collections::HashSet::new();
        for ((trait_name, ty), (_impl_block, _module)) in &registry.impls {
            let key = (trait_name.clone(), ty.clone());
            if seen.contains(&key) {
                return Err(ModuleError {
                    message: format!(
                        "CoherenceViolation: trait '{}' for type '{}' has impls in multiple modules",
                        trait_name, ty
                    ),
                });
            }
            seen.insert(key);
        }
    }
    Ok(())
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

    bytes.extend_from_slice(b"LC10");

    let num_nodes = net.nodes().len() as u32;
    bytes.extend_from_slice(&num_nodes.to_le_bytes());

    let num_wires = net.wires().len() as u32;
    bytes.extend_from_slice(&num_wires.to_le_bytes());

    bytes
}

pub fn compile_to_object(module: &Module, output: &Path) -> Result<(), ModuleError> {
    let data = serialize_module(module)?;

    std::fs::write(output, data).map_err(|e| ModuleError {
        message: format!("Failed to write object file: {}", e),
    })?;

    Ok(())
}

pub fn load_object(path: &Path) -> Result<Module, ModuleError> {
    let data = std::fs::read(path).map_err(|e| ModuleError {
        message: format!("Failed to read object: {}", e),
    })?;

    deserialize_module(&data)
}
