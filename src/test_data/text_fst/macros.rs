

macro_rules! rel_to_abs_path {
    ($filename: expr) => {
        PathBuf::from(file!()).parent().unwrap().join($filename)
    };
}