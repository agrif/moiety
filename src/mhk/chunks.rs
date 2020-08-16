use serde_derive::Deserialize;

#[derive(Debug, Deserialize)]
pub struct MHWK {
    pub signature: [u8; 4],
    pub file_size: u32,
}

#[derive(Debug, Deserialize)]
pub struct RSRC {
    pub signature: [u8; 4],
    pub version: u16,
    pub compaction: u16,
    pub file_size: u32,
    pub resource_dir_offset: u32,
    pub file_table_offset: u16,
    pub file_table_size: u16,
}

#[derive(Debug, Deserialize)]
pub struct TypeTableEntry {
    pub resource_type: [u8; 4],
    pub resource_table_offset: u16,
    pub name_table_offset: u16,
}

#[derive(Debug, Deserialize)]
pub struct NameTableEntry {
    pub name_offset: u16,
    pub file_table_index: u16,
}

#[derive(Debug, Deserialize)]
pub struct ResourceTableEntry {
    pub resource_id: u16,
    pub file_table_index: u16,
}

#[derive(Debug, Deserialize)]
pub struct FileTableEntry {
    pub offset: u32,
    pub size_low: u16,
    pub size_high: u8,
    pub flags: u8,
    pub unknown0: u16,
}
