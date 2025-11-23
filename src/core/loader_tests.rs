use super::loader::{load_pe, PeArtifact};
use anyhow::Result;
use std::fs;
use std::path::PathBuf;

fn create_dummy_pe() -> Vec<u8> {
    let mut data = vec![0u8; 1024];
    // MZ signature
    data[0] = 0x4D;
    data[1] = 0x5A;
    
    // e_lfanew (offset to PE header) at 0x3C -> 0x40
    data[0x3C] = 0x40;
    
    // PE signature at 0x40
    data[0x40] = 0x50;
    data[0x41] = 0x45;
    data[0x42] = 0x00;
    data[0x43] = 0x00;
    
    // COFF Header at 0x44
    // Machine: 0x14C (i386)
    data[0x44] = 0x4C;
    data[0x45] = 0x01;
    
    // NumberOfSections: 1
    data[0x46] = 0x01;
    data[0x47] = 0x00;
    
    // SizeOfOptionalHeader: 0xE0 (224 bytes)
    data[0x54] = 0xE0;
    data[0x55] = 0x00;
    
    // Characteristics: 0x02 (Executable)
    data[0x56] = 0x02;
    data[0x57] = 0x00;
    
    // Optional Header at 0x58 (follows COFF header which is 20 bytes)
    // Magic: 0x10B (PE32)
    data[0x58] = 0x0B;
    data[0x59] = 0x01;
    
    // Section Table starts after Optional Header
    // 0x58 + 0xE0 = 0x138
    let section_offset = 0x138;
    
    // Name: ".text"
    let name = b".text\0\0\0";
    for (i, b) in name.iter().enumerate() {
        data[section_offset + i] = *b;
    }
    
    // VirtualSize: 0x100
    data[section_offset + 8] = 0x00;
    data[section_offset + 9] = 0x01;
    
    // VirtualAddress: 0x1000
    data[section_offset + 12] = 0x00;
    data[section_offset + 13] = 0x10;
    
    // SizeOfRawData: 0x200
    data[section_offset + 16] = 0x00;
    data[section_offset + 17] = 0x02;
    
    // PointerToRawData: 0x200
    data[section_offset + 20] = 0x00;
    data[section_offset + 21] = 0x02;
    
    // Characteristics: 0x60000020 (Code | Execute | Read)
    data[section_offset + 36] = 0x20;
    data[section_offset + 39] = 0x60;

    data
}

fn setup_fixture(name: &str, content: Vec<u8>) -> PathBuf {
    let mut path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    path.push("tests");
    path.push("fixtures");
    fs::create_dir_all(&path).unwrap();
    path.push(name);
    fs::write(&path, content).unwrap();
    path
}

#[test]
fn test_load_pe_success() {
    let pe_data = create_dummy_pe();
    let path = setup_fixture("valid.exe", pe_data);
    
    let result = load_pe(&path);
    assert!(result.is_ok(), "Failed to load valid PE: {:?}", result.err());
    
    let artifact = result.unwrap();
    assert_eq!(artifact.sections.len(), 1);
    assert_eq!(artifact.sections[0].name, ".text");
    assert_eq!(artifact.sections[0].virtual_address, 0x1000);
    // We didn't populate imports, so it should be empty
    assert_eq!(artifact.imports.len(), 0);
}

#[test]
fn test_load_pe_invalid_file() {
    let path = setup_fixture("invalid.exe", b"NOT A PE FILE".to_vec());
    let result = load_pe(&path);
    assert!(result.is_err());
    // Check if error message contains reasonable info
    // Note: exact message depends on goblin/anyhow
    let err_msg = result.err().unwrap().to_string();
    assert!(err_msg.contains("Failed to parse PE file") || err_msg.contains("PE")); 
}

#[test]
fn test_load_pe_missing_file() {
    let path = PathBuf::from("non_existent_file.exe");
    let result = load_pe(&path);
    assert!(result.is_err());
    let err_msg = result.err().unwrap().to_string();
    assert!(err_msg.contains("Failed to read file"));
}
