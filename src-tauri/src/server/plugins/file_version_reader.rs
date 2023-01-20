use sha2::{Digest, Sha256};

/// Gets a file version from the file properties.
///
/// Can be used for `.exe` and `.dll` files.
pub fn get_file_version_from_file_properties(path: &str) -> Result<String, String> {
    // make sure path has double backslashes on windows
    let version = version_info::get_file_version(&path);

    match version {
        Some((a, b, c, d)) => return Ok(format!("{}.{}.{}.{}", a, b, c, d)),
        None => return Err("Could not read version.".to_string()),
    }
}

/// Gets the file hash from the file metadata.
///
/// Can be used for files without a specific file version.
pub fn get_file_meta_hash(path: &str) -> Result<String, String> {
    let file = std::fs::File::open(&path);

    match file {
        Ok(file) => {
            let metadata = file.metadata();
            match metadata {
                Ok(meta) => {
                    let mut hasher = Sha256::new();
                    hasher.update(format!("{meta:?}"));
                    let hash = hasher.finalize();

                    Ok(format!("{:x}", hash))
                }
                Err(err) => Err(format!("[Get File Version] {}", err.to_string())),
            }
        }
        Err(err) => Err(format!("[Get File Version] {}", err.to_string())),
    }
}
