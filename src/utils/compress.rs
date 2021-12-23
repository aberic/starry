/*
 * Copyright (c) 2021. Aberic - All Rights Reserved.
 *
 * Licensed under the Apache License, Version 2.0 (the "License");
 * you may not use this file except in compliance with the License.
 * You may obtain a copy of the License at
 * http://www.apache.org/licenses/LICENSE-2.0
 * Unless required by applicable law or agreed to in writing, software
 * distributed under the License is distributed on an "AS IS" BASIS,
 * WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
 * See the License for the specific language governing permissions and
 * limitations under the License.
 */

use std::io::prelude::*;

use flate2::Compression;
use flate2::read::{DeflateDecoder, GzDecoder, ZlibDecoder};
use flate2::write::{DeflateEncoder, GzEncoder, ZlibEncoder};

use crate::utils::errors::{Errs, StarryResult};

pub struct Compress;

impl Compress {
    pub fn zlib(data: &[u8], level: Compression) -> StarryResult<Vec<u8>> {
        let mut e = ZlibEncoder::new(Vec::new(), level);
        match e.write_all(data) {
            Ok(_) => {},
            Err(err) => return Err(Errs::strs("zlib compress write failed!", err))
        }
        match e.finish() {
            Ok(src) => Ok(src),
            Err(err) => Err(Errs::strs("zlib compress failed!", err))
        }
    }

    pub fn un_zlib(data: &[u8]) -> StarryResult<Vec<u8>> {
        let mut d = ZlibDecoder::new(data);
        let mut s = vec![];
        match d.read_to_end(&mut s) {
            Ok(_) => Ok(s),
            Err(err) => Err(Errs::strs("zlib uncompress failed!", err))
        }
    }

    pub fn deflate(data: &[u8], level: Compression) -> StarryResult<Vec<u8>> {
        let mut e = DeflateEncoder::new(Vec::new(), level);
        match e.write_all(data) {
            Ok(_) => {},
            Err(err) => return Err(Errs::strs("deflate compress write failed!", err))
        }
        match e.finish() {
            Ok(src) => Ok(src),
            Err(err) => Err(Errs::strs("deflate compress failed!", err))
        }
    }

    pub fn un_deflate(data: &[u8]) -> StarryResult<Vec<u8>> {
        let mut d = DeflateDecoder::new(data);
        let mut s = vec![];
        match d.read_to_end(&mut s) {
            Ok(_) => Ok(s),
            Err(err) => Err(Errs::strs("deflate uncompress failed!", err))
        }
    }

    pub fn gzip(data: &[u8], level: Compression) -> StarryResult<Vec<u8>> {
        let mut e = GzEncoder::new(Vec::new(), level);
        match e.write_all(data) {
            Ok(_) => {},
            Err(err) => return Err(Errs::strs("gzip compress write failed!", err))
        }
        match e.finish() {
            Ok(src) => Ok(src),
            Err(err) => Err(Errs::strs("gzip compress failed!", err))
        }
    }

    pub fn un_gzip(data: &[u8]) -> StarryResult<Vec<u8>> {
        let mut d = GzDecoder::new(data);
        let mut s = vec![];
        match d.read_to_end(&mut s) {
            Ok(_) => Ok(s),
            Err(err) => Err(Errs::strs("gzip uncompress failed!", err))
        }
    }
}

#[cfg(test)]
mod compress_test {
    use flate2::Compression;
    use crate::utils::compress::Compress;

    const DD: &str = "foobar!foobar!foobar!foobar!foobar!foobar!foobar!foobar!foobar!foobar!";

    #[test]
    fn zlib_test() {
        let data = DD.as_bytes();
        let res = Compress::zlib(data, Compression::default()).unwrap();
        let data_bak = Compress::un_zlib(res.as_slice()).unwrap();
        assert!(data.len() > res.len());
        assert_eq!(String::from_utf8(data_bak).unwrap(), "foobar!foobar!foobar!foobar!foobar!foobar!foobar!foobar!foobar!foobar!");
    }

    #[test]
    fn deflate_test() {
        let data = DD.as_bytes();
        let res = Compress::deflate(data, Compression::default()).unwrap();
        let data_bak = Compress::un_deflate(res.as_slice()).unwrap();
        assert!(data.len() > res.len());
        assert_eq!(String::from_utf8(data_bak).unwrap(), "foobar!foobar!foobar!foobar!foobar!foobar!foobar!foobar!foobar!foobar!");
    }

    #[test]
    fn gzip_test() {
        let data = DD.as_bytes();
        let res = Compress::gzip(data, Compression::default()).unwrap();
        let data_bak = Compress::un_gzip(res.as_slice()).unwrap();
        assert!(data.len() > res.len());
        assert_eq!(String::from_utf8(data_bak).unwrap(), "foobar!foobar!foobar!foobar!foobar!foobar!foobar!foobar!foobar!foobar!");
    }
}


