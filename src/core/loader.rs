use anyhow::{bail, Context, Result};
use goblin::pe::PE;
use std::collections::HashMap;
use std::fs;
use std::path::Path;

#[derive(Debug, Clone)]
pub struct SectionInfo {
    pub name: String,
    pub virtual_address: u32,
    pub virtual_size: u32,
}

#[derive(Debug, Clone)]
pub struct ImportInfo {
    pub dll: String,
    pub symbols: Vec<String>,
}

#[derive(Debug, Clone)]
pub struct PeArtifact {
    pub sections: Vec<SectionInfo>,
    pub imports: Vec<ImportInfo>,
}

/// Loads a PE file from the given path and parses it into a PeArtifact.
pub fn load_pe(path: &Path) -> Result<PeArtifact> {
    let buffer = fs::read(path).with_context(|| format!("Failed to read file: {:?}", path))?;

    let pe = match PE::parse(&buffer) {
        Ok(pe) => pe,
        Err(e) => bail!("Failed to parse PE file: {}", e),
    };

    let sections = pe
        .sections
        .iter()
        .map(|section| {
            let name = section.name().unwrap_or("").to_string();
            SectionInfo {
                name,
                virtual_address: section.virtual_address,
                virtual_size: section.virtual_size,
            }
        })
        .collect();

    let mut import_map: HashMap<String, Vec<String>> = HashMap::new();
    for import in pe.imports {
        import_map
            .entry(import.dll.to_string())
            .or_default()
            .push(import.name.to_string());
    }

    let imports = import_map
        .into_iter()
        .map(|(dll, symbols)| ImportInfo { dll, symbols })
        .collect();

    Ok(PeArtifact { sections, imports })
}

// Deprecated Loader struct to maintain compatibility with existing main.rs if needed.
// In a real scenario, we would refactor main.rs to use load_pe.
pub struct Loader {
    _data: Vec<u8>,
}

impl Loader {
    #[allow(clippy::new_without_default)]
    pub fn new() -> Self {
        Self { _data: Vec::new() }
    }

    pub fn load(&self, path: &str) -> Result<()> {
        // For now, we can just call load_pe and ignore the result, or print it.
        // This is just to keep the existing interface working.
        let _ = load_pe(Path::new(path))?;
        Ok(())
    }
}
