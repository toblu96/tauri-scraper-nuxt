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

/// Gets the file hash from a file content.
///
/// Can be used for files without a specific file version.
pub fn get_file_hash(path: &str) -> Result<String, String> {
    println!("hell ho");
    Ok("jop".to_string())
}
