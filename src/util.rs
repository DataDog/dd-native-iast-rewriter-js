/**
* Unless explicitly stated otherwise all files in this repository are licensed under the Apache-2.0 License.
* This product includes software developed at Datadog (https://www.datadoghq.com/). Copyright 2022 Datadog, Inc.
**/
use std::{
    fs::File,
    io::Read,
    path::{Path, PathBuf},
};
use swc::sourcemap::SourceMap;

pub fn file_name(file: &str) -> Option<&str> {
    Path::new(file).file_name().and_then(|s| s.to_str())
}

pub fn parse_source_map(source_map: Option<&str>) -> Option<SourceMap> {
    source_map.and_then(|source| SourceMap::from_reader(source.as_bytes()).ok())
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

pub trait FileReader<R: Read> {
    fn read(&self, path: &Path) -> std::io::Result<R>
    where
        R: Read,
        Self: Sized;

    fn parent(&self, path: &Path) -> Option<PathBuf> {
        path.parent().map(PathBuf::from)
    }

    fn log(&self, msg: String) {
        print!("{msg}")
    }
}

pub struct DefaultFileReader {}
impl FileReader<File> for DefaultFileReader {
    fn read(&self, path: &Path) -> std::io::Result<File>
    where
        File: Read,
        Self: Sized,
    {
        File::open(path)
    }
}
