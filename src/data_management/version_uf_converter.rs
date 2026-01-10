

pub fn convert_old_userfiles() {

    // loop through the files in ./data/users and convert them to the new format
    let path = std::path::Path::new("./data/users");

    // loop through all files in the directory
    for entry in std::fs::read_dir(path).unwrap() {
        // todo
    }

}