/*
 * Copyright (c) 2020. Aberic - All Rights Reserved.
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

use std::fmt::{Display, Formatter, Result};
use crate::utils::errors::children::StringError;

trait GeorgeStringErr<M, N>: Sized {
    fn string(_: M, _: N) -> Self;
}

trait GeorgeString<M>: Sized {
    fn string(_: M) -> Self;
}

/// 索引触发Error,实现std::fmt::Debug的trait
#[derive(Debug, Clone)]
pub enum Error {
    StringError(StringError),
}

impl std::error::Error for Error {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match &self {
            Error::StringError(ref e) => Some(e),
        }
    }
}

impl Display for Error {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        match &self {
            Error::StringError(ref e) => e.fmt(f),
        }
    }
}

impl From<StringError> for Error {
    fn from(s: StringError) -> Self {
        Error::StringError(s)
    }
}

impl<T: ToString> GeorgeStringErr<String, T> for Error {
    fn string(msg: String, err: T) -> Self {
        err_strings(msg, err.to_string())
    }
}

impl<T: ToString> GeorgeStringErr<&str, T> for Error {
    fn string(msg: &str, err: T) -> Self {
        err_strs(msg, err.to_string())
    }
}

impl GeorgeString<String> for Error {
    fn string(msg: String) -> Self {
        err_string(msg)
    }
}

impl GeorgeString<&str> for Error {
    fn string(msg: &str) -> Self {
        err_str(msg)
    }
}

pub struct Errs;

impl Errs {
    pub fn err<Err: std::error::Error>(err: Err) -> Error {
        err_string(err.to_string())
    }

    pub fn string(msg: String) -> Error {
        err_string(msg)
    }

    pub fn str(msg: &str) -> Error {
        err_str(msg)
    }

    pub fn strs<Err: ToString>(msg: &str, err: Err) -> Error {
        err_strs(msg, err)
    }

    pub fn strings<Err: ToString>(msg: String, err: Err) -> Error {
        err_strings(msg, err)
    }
}

fn err_string(msg: String) -> Error {
    Error::StringError(StringError { error_msg: msg })
}

fn err_str(msg: &str) -> Error {
    Error::StringError(StringError {
        error_msg: msg.to_string(),
    })
}

fn err_strs<Err: ToString>(msg: &str, err: Err) -> Error {
    Error::StringError(StringError {
        error_msg: format!("{}: {}", msg, err.to_string()),
    })
}

fn err_strings<Err: ToString>(msg: String, err: Err) -> Error {
    Error::StringError(StringError {
        error_msg: format!("{}: {}", msg, err.to_string()),
    })
}
