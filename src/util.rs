use anyhow::{Error, Result};
use std::{
    fs::{File, OpenOptions},
    path::Path,
};
use swc::sourcemap::SourceMap;

pub fn create_file(file: &Path) -> Result<File> {
    std::fs::create_dir_all(file.parent().unwrap())
        .and_then(|_| {
            OpenOptions::new()
                .create(true)
                .truncate(true)
                .write(true)
                .open(file)
        })
        .map_err(Error::new)
}

pub fn file_name(file: &str) -> Option<&str> {
    Path::new(file).file_name().and_then(|s| s.to_str())
}

pub fn parse_source_map(source_map: Option<&str>) -> Option<SourceMap> {
    return source_map.and_then(|source| SourceMap::from_reader(source.as_bytes()).ok());
}

pub fn rnd_string(length: usize) -> String {
    let chars: Vec<char> = "abcdefghijklmnopqrstuvwxyz".chars().collect();
    let mut result = String::with_capacity(length);

    unsafe {
        for _ in 0..length {
            result.push(*chars.get_unchecked(fastrand::usize(0..chars.len())));
        }
    }

    result
}
