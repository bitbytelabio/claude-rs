use std::path::Path;
use std::fs;
use crate::Result;

pub enum FileType {
    Pdf,
    Txt,
    Csv,
    // Add more file types as needed
    Binary,
}

impl std::fmt::Display for FileType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let file_type = match self {
            FileType::Pdf => "application/pdf",
            FileType::Txt => "text/plain",
            FileType::Csv => "text/csv",
            // Add more file types as needed

            FileType::Binary => "application/octet-stream",
        };
        write!(f, "{}", file_type)
    }
}

pub fn get_content_type(path: &Path) -> FileType {
    let extension = path
        .extension()
        .unwrap_or_default()
        .to_str()
        .unwrap_or_default()
        .to_lowercase();
    match extension.as_str() {
        "pdf" => FileType::Pdf,
        "txt" => FileType::Txt,
        "csv" => FileType::Csv,
        // Add more content types as needed for other file types
        _ => FileType::Binary,
    }
}

pub fn get_file_size(file_path: &str) -> Result<u64> {
    let metadata = fs::metadata(file_path)?;
    let file_size = metadata.len();
    Ok(file_size)
}
