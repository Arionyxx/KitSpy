use super::loader::load_pe;
use std::fs;
use std::path::PathBuf;

fn create_dummy_pe() -> Vec<u8> {
    let mut data = vec![0u8; 1024];
    // MZ signature
    data[0] = 0x4D;
    data[1] = 0x5A;
    
    // e_lfanew (offset to PE header) at 0x3C -> 0x80
    data[0x3C] = 0x80;
    
    // PE signature at 0x80
    let pe_header_start = 0x80;
    data[pe_header_start] = 0x50;
    data[pe_header_start + 1] = 0x45;
    data[pe_header_start + 2] = 0x00;
    data[pe_header_start + 3] = 0x00;
    
    // COFF Header at pe_header_start + 4
    // Machine: 0x14C (i386)
    data[pe_header_start + 4] = 0x4C;
    data[pe_header_start + 5] = 0x01;
    
    // NumberOfSections: 1
    data[pe_header_start + 6] = 0x01;
    data[pe_header_start + 7] = 0x00;
    
    // SizeOfOptionalHeader: 0xE0 (224 bytes)
    data[pe_header_start + 20] = 0xE0;
    data[pe_header_start + 21] = 0x00;
    
    // Characteristics: 0x02 (Executable)
    data[pe_header_start + 22] = 0x02;
    data[pe_header_start + 23] = 0x00;
    
    // Optional Header at pe_header_start + 24 (follows COFF header which is 20 bytes)
    // Magic: 0x10B (PE32)
    data[pe_header_start + 24] = 0x0B;
    data[pe_header_start + 25] = 0x01;
    
    // Section Table starts after Optional Header
    // 0x80 + 4 + 20 + 224 = 0x164
    // (pe_header_start + 4 (sig) + 20 (coff) + 224 (opt))
    // 0x80 + 248 = 0x178 ?
    // Wait.
    // PE Sig: 4 bytes
    // COFF Header: 20 bytes
    // Optional Header: 224 bytes (SizeOfOptionalHeader)
    // Total = 248 bytes.
    // Start of Section Table = pe_header_start + 248 = 0x80 + 0xF8 = 0x178.
    
    let section_offset = pe_header_start + 4 + 20 + 224;
    
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
