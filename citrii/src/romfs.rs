use byte_struct::*;

#[derive(ByteStructLE)]
struct Header {
    header_length: u32,
    dir_hash_table_offset: u32,
    dir_hash_table_length: u32,
    dir_table_offset: u32,
    dir_table_length: u32,
    file_hash_table_offset: u32,
    file_hash_table_length: u32,
    file_table_offset: u32,
    file_table_length: u32,
    data_offset: u32,
}

#[derive(ByteStructLE)]
struct DirectoryMetadata {
    parent_dir_offset: u32,
    next_dir_offset: u32,
    first_child_dir_offset: u32,
    first_file_offset: u32,
    same_hash_next_dir_offet: u32,
    name_length: u32,
}

#[derive(ByteStructLE)]
struct FileMetadata {
    parent_dir_offset: u32,
    next_file_offset: u32,
    data_offset: u64,
    data_length: u64,
    same_hash_next_file_offset: u32,
    name_length: u32,
}

pub fn get_romfs_file<'a>(romfs: &'a [u8], path: &[String]) -> Option<&'a [u8]> {
    const INVALID_FIELD: u32 = 0xFFFFFFFF;
    let dir_names = &path[0 .. path.len() - 1];
    let file_name = path.last().unwrap();

    let header = Header::read_bytes(&romfs[0 .. Header::BYTE_LEN]);
    let root = header.dir_table_offset as usize;
    let mut dir = DirectoryMetadata::read_bytes(&romfs[root .. root + DirectoryMetadata::BYTE_LEN]);
    for dir_name in dir_names {
        let dir_name_utf16: Vec<[u8; 2]> = dir_name.encode_utf16().map(|c|c.to_le_bytes()).collect();
        let dir_name_bytes: Vec<u8> = dir_name_utf16.iter().flatten().cloned().collect();
        let mut child_dir_offset = dir.first_child_dir_offset;
        loop {
            if child_dir_offset == INVALID_FIELD {
                return None;
            }
            let current_child_dir = (header.dir_table_offset + child_dir_offset) as usize;
            let name_begin = current_child_dir + DirectoryMetadata::BYTE_LEN;
            dir = DirectoryMetadata::read_bytes(&romfs[current_child_dir .. name_begin]);
            if dir_name_bytes == &romfs[name_begin .. name_begin + dir.name_length as usize] {
                break;
            }
            child_dir_offset = dir.next_dir_offset;
        }
    }

    let file_name_utf16: Vec<[u8; 2]> = file_name.encode_utf16().map(|c|c.to_le_bytes()).collect();
    let file_name_bytes: Vec<u8> = file_name_utf16.iter().flatten().cloned().collect();

    let mut file_offset = dir.first_file_offset;
    while file_offset != INVALID_FIELD {
        let current_file = (header.file_table_offset + file_offset) as usize;
        let name_begin = current_file + FileMetadata::BYTE_LEN;
        let file = FileMetadata::read_bytes(&romfs[current_file .. name_begin]);
        if file_name_bytes == &romfs[name_begin .. name_begin + file.name_length as usize] {
            let file_offset = (header.data_offset as u64 + file.data_offset) as usize;
            return Some(&romfs[file_offset .. file_offset + file.data_length as usize]);
        }
        file_offset = file.next_file_offset;
    }

    None
}

#[test]
fn struct_size_test() {
    assert_eq!(Header::BYTE_LEN, 0x28);
    assert_eq!(DirectoryMetadata::BYTE_LEN, 0x18);
    assert_eq!(FileMetadata::BYTE_LEN, 0x20);
}
