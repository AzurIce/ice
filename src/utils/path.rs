use std::path::Path;

pub fn split_parent_and_file(path: String) -> (String, String) {
    let path = Path::new(&path);
    let parent_path = path.parent().unwrap().to_str().unwrap();
    let file_path = path.file_name().unwrap().to_str().unwrap();
    (parent_path.to_string(), file_path.to_string())
}
