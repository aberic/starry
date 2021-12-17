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
use std::hash::{Hash, Hasher};
use crate::utils::errors::{Errs, StarryResult};

use crate::http::url::scheme::Inner::*;

/// Scheme url提供的一种机制，它可以由应用程序注册，然后其他程序通过url scheme来调用该应用程序
///
/// Scheme在url中所处定位参考如下：
/// ```notrust
///  abc://username:password@example.com:123/path/data?key=value&key2=value2#fragid1
///  |-|  |---------------|  |-------------||--------| |-------------------| |-----|
///   |           |                |            |               |              |
///   |       UserInfo           Addr         Path            Query         Fragment
///   |   |--------------------------------||--------------------------------------|
///   |                  |                                      |
/// Scheme          Authority                                Location
/// ```
#[derive(Clone, PartialEq, Eq)]
pub struct Scheme(Inner);

#[derive(Clone, PartialEq, Eq)]
enum Inner {
    Http,
    Https,
}

impl Scheme {
    /// http协议
    pub const HTTP: Scheme = Scheme(Http);

    /// 基于tls的http协议
    pub const HTTPS: Scheme = Scheme(Https);

    /// 通过已知字节数组获取HTTP方法
    pub fn from_bytes(src: &[u8]) -> StarryResult<Scheme> {
        match src.len() {
            4 => match src {
                b"http" => Ok(Scheme(Http)),
                _ => Err(Errs::str("invalid scheme!")),
            },
            5 => match src {
                b"https" => Ok(Scheme(Https)),
                _ => Err(Errs::str("invalid scheme!")),
            },
            _ => Err(Errs::str("invalid scheme!")),
        }
    }

    pub fn from_str(t: &str) -> StarryResult<Scheme> {
        Scheme::from_bytes(t.as_bytes())
    }

    /// 用`&str`表示当前协议
    pub fn as_str(&self) -> &str {
        match self.0 {
            Http => "http",
            Https => "https",
        }
    }

    pub fn is_tls(&self) -> bool {
        match self.0 {
            Https => true,
            _ => false,
        }
    }
}

impl AsRef<str> for Scheme {
    fn as_ref(&self) -> &str {
        self.as_str()
    }
}

impl<'a> PartialEq<&'a Scheme> for Scheme {
    fn eq(&self, other: &&'a Scheme) -> bool {
        self == *other
    }
}

impl<'a> PartialEq<Scheme> for &'a Scheme {
    fn eq(&self, other: &Scheme) -> bool {
        *self == other
    }
}

impl PartialEq<str> for Scheme {
    fn eq(&self, other: &str) -> bool {
        self.as_ref() == other
    }
}

impl PartialEq<Scheme> for str {
    fn eq(&self, other: &Scheme) -> bool {
        self == other.as_ref()
    }
}

impl<'a> PartialEq<&'a str> for Scheme {
    fn eq(&self, other: &&'a str) -> bool {
        self.as_ref() == *other
    }
}

impl<'a> PartialEq<Scheme> for &'a str {
    fn eq(&self, other: &Scheme) -> bool {
        *self == other.as_ref()
    }
}

impl fmt::Debug for Scheme {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_ref())
    }
}

/// Case-insensitive hashing
impl Hash for Scheme {
    fn hash<H>(&self, state: &mut H)
        where
            H: Hasher,
    {
        match *self {
            Scheme::HTTP => state.write_u8(1),
            Scheme::HTTPS => state.write_u8(2),
        }
    }
}

#[cfg(test)]
mod scheme_test {
    use crate::http::url::Scheme;

    #[test]
    fn scheme_eq() {
        assert_eq!(Scheme::HTTP, "http");
        assert_eq!(&Scheme::HTTPS, "https");

        assert_eq!("http", Scheme::HTTP);
        assert_eq!("https", &Scheme::HTTPS);

        assert_eq!(&Scheme::HTTP, Scheme::HTTP);
        assert_eq!(Scheme::HTTPS, &Scheme::HTTPS);
    }

    #[test]
    fn invalid_scheme() {
        assert!(Scheme::from_str("http").is_ok());
        assert!(Scheme::from_bytes(b"https").is_ok());
        assert!(Scheme::from_str("omg").is_err());
        assert!(Scheme::from_bytes(b"omg").is_err());
        assert!(Scheme::from_bytes(&[0xC0]).is_err()); // invalid utf-8
        assert!(Scheme::from_bytes(&[0x10]).is_err()); // invalid method characters
    }

    #[test]
    fn is_tls() {
        assert!(Scheme::HTTPS.is_tls());

        assert!(!Scheme::HTTP.is_tls());
    }
}
