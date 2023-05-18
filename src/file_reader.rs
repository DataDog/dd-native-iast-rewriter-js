/**
 * Unless explicitly stated otherwise all files in this repository are licensed under the Apache-2.0 License.
 * This product includes software developed at Datadog (https://www.datadoghq.com/). Copyright 2022 Datadog, Inc.
 **/
use std::{fs::File, io::Read, path::Path};

pub trait FileReader<R: Read> {
    fn read(&self, path: &Path) -> std::io::Result<R>
    where
        R: Read,
        Self: Sized;
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
