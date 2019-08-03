macro_rules! rel_to_abs_path {
    ($filename: expr) => {
        PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("src/test_data/text_fst")
            .join($filename)
    };
}
