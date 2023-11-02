use std::path::Path;

pub fn get_content_type(file_path: &str) -> &str {
    let extension = Path::new(file_path)
        .extension()
        .unwrap_or_default()
        .to_str()
        .unwrap_or_default()
        .to_lowercase();
    match extension.as_str() {
        ".pdf" => "application/pdf",
        ".txt" => "text/plain",
        ".csv" => "text/csv",
        // Add more content types as needed for other file types
        _ => "application/octet-stream",
    }
}
