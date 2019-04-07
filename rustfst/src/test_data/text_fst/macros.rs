macro_rules! rel_to_abs_path {
    ($filename: expr) => {
        PathBuf::from("src/test_data/text_fst").join($filename)
    };
}
