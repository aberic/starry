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

use std::fmt;

use crate::utils::errors::{Errs, StarryResult};

use self::Protocol::*;

/// 收/发请求的协议版本
///
/// HTTP/1.1或HTTP/2等
#[derive(Clone, PartialEq, Eq)]
pub struct Version {
    protocol: Protocol,
    major: u8,
    minor: u8,
}

#[derive(Clone, PartialEq, Eq)]
pub enum Protocol {
    Http10,
    Http11,
    Http2,
}

impl Version {
    /// `HTTP/1.0`
    pub const HTTP_10: Version = Version {
        protocol: Http10,
        major: 1,
        minor: 0,
    };

    /// `HTTP/1.1`
    pub const HTTP_11: Version = Version {
        protocol: Http11,
        major: 1,
        minor: 1,
    };

    /// `HTTP/2.0`
    pub const HTTP_2: Version = Version {
        protocol: Http2,
        major: 2,
        minor: 0,
    };

    pub fn protocol(&self) -> Protocol {
        self.protocol.clone()
    }

    pub fn major(&self) -> u8 {
        self.major
    }

    pub fn minor(&self) -> u8 {
        self.minor
    }

    pub fn from_str(src: &str) -> StarryResult<Version> {
        match src {
            "HTTP/1.0" => Ok(Version::HTTP_10),
            "HTTP/1.1" => Ok(Version::HTTP_11),
            "HTTP/2.0" => Ok(Version::HTTP_2),
            _ => Err(Errs::string(format!("version is not support except HTTP/1.0 HTTP/1.1 HTTP/2.0!"))),
        }
    }

    /// 通过已知字节数组获取HTTP方法
    pub(crate) fn from_bytes(src: &[u8]) -> StarryResult<Version> {
        match src {
            b"HTTP/1.0" => Ok(Version::HTTP_10),
            b"HTTP/1.1" => Ok(Version::HTTP_11),
            b"HTTP/2.0" => Ok(Version::HTTP_2),
            _ => Err(Errs::string(format!("version is not support except HTTP/1.0 HTTP/1.1 HTTP/2.0!"))),
        }
    }

    pub fn as_slice(&self) -> &[u8] {
        match self.protocol {
            Http10 => b"HTTP/1.0",
            Http11 => b"HTTP/1.1",
            Http2 => b"HTTP/2.0",
        }
    }
}

impl AsRef<str> for Version {
    fn as_ref(&self) -> &str {
        match self.protocol {
            Http10 => "HTTP/1.0",
            Http11 => "HTTP/1.1",
            Http2 => "HTTP/2.0",
        }
    }
}

impl ToString for Version {
    fn to_string(&self) -> String {
        String::from(self.as_ref())
    }
}

impl<'a> PartialEq<&'a Version> for Version {
    fn eq(&self, other: &&'a Version) -> bool {
        self == *other
    }
}

impl<'a> PartialEq<Version> for &'a Version {
    fn eq(&self, other: &Version) -> bool {
        *self == other
    }
}

impl PartialEq<str> for Version {
    fn eq(&self, other: &str) -> bool {
        self.as_ref() == other
    }
}

impl PartialEq<Version> for str {
    fn eq(&self, other: &Version) -> bool {
        self == other.as_ref()
    }
}

impl<'a> PartialEq<&'a str> for Version {
    fn eq(&self, other: &&'a str) -> bool {
        self.as_ref() == *other
    }
}

impl<'a> PartialEq<Version> for &'a str {
    fn eq(&self, other: &Version) -> bool {
        *self == other.as_ref()
    }
}

impl Default for Version {
    fn default() -> Version {
        Version::HTTP_11
    }
}

impl fmt::Debug for Version {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_ref())
    }
}

#[cfg(test)]
mod version_test {
    use crate::Version;

    #[test]
    fn scheme_eq() {
        assert_eq!(Version::HTTP_2, Version::HTTP_2);
        assert_eq!(Version::HTTP_10, "HTTP/1.0");
        assert_eq!(&Version::HTTP_11, "HTTP/1.1");

        assert_eq!("HTTP/2.0", Version::HTTP_2);
        assert_eq!("HTTP/1.1", &Version::HTTP_11);

        assert_eq!(&Version::HTTP_11, Version::HTTP_11);
        assert_eq!(Version::HTTP_10, &Version::HTTP_10);
    }

    #[test]
    fn to_string() {
        let v1 = Version::from_str("HTTP/2.0").unwrap();
        assert_eq!(v1.to_string(), "HTTP/2.0");

        let v2 = Version::default();
        assert_eq!(v2.to_string(), "HTTP/1.1");
    }
}
