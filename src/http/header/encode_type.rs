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

#[derive(Debug, Clone, Eq, PartialEq)]
pub enum AcceptEncoding {
    GZip,
    Deflate,
    Br,
    ZLib,
    None,
}

impl AcceptEncoding {
    pub(crate) fn best(src: String) -> Option<Self> {
        let mut res = None;
        let split = src.trim().split(",");
        for src in split {
            let this = AcceptEncoding::from(src);
            if this.ne("") {
                if this.eq(&AcceptEncoding::GZip) {
                    res = Some(AcceptEncoding::GZip);
                    break;
                } else if this.eq(&AcceptEncoding::Deflate) {
                    res = Some(AcceptEncoding::Deflate);
                } else if this.eq(&AcceptEncoding::ZLib) && res.clone().unwrap_or(AcceptEncoding::None).ne(&AcceptEncoding::Deflate) {
                    res = Some(AcceptEncoding::ZLib)
                }
            }
        }
        res
    }

    // pub(crate) fn support_str() -> &'static str {
    //     "gzip, deflate, zlib"
    // }
    //
    // pub(crate) fn support() -> String {
    //     AcceptEncoding::support_str().to_string()
    // }

    /// 用`&str`表示当前Type
    pub(crate) fn as_str(&self) -> &str {
        match self {
            AcceptEncoding::GZip => "gzip",
            AcceptEncoding::Deflate => "deflate",
            AcceptEncoding::Br => "br",
            AcceptEncoding::ZLib => "zlib",
            AcceptEncoding::None => "",
        }
    }
}

impl From<&str> for AcceptEncoding {
    fn from(src: &str) -> Self {
        match src {
            "gzip" => AcceptEncoding::GZip,
            "deflate" => AcceptEncoding::Deflate,
            "br" => AcceptEncoding::Br,
            "zlib" => AcceptEncoding::ZLib,
            _ => AcceptEncoding::None
        }
    }
}

impl From<String> for AcceptEncoding {
    fn from(src: String) -> Self {
        AcceptEncoding::from(src.as_str())
    }
}

impl ToString for AcceptEncoding {
    fn to_string(&self) -> String {
        self.as_str().to_string()
    }
}

impl<'a> PartialEq<&'a AcceptEncoding> for AcceptEncoding {
    fn eq(&self, other: &&'a AcceptEncoding) -> bool {
        self.as_str().eq(other.as_str())
    }
}

impl<'a> PartialEq<AcceptEncoding> for &'a AcceptEncoding {
    fn eq(&self, other: &AcceptEncoding) -> bool {
        self.as_str().eq(other.as_str())
    }
}

impl PartialEq<str> for AcceptEncoding {
    fn eq(&self, other: &str) -> bool {
        self.as_str() == other
    }
}

impl PartialEq<AcceptEncoding> for str {
    fn eq(&self, other: &AcceptEncoding) -> bool {
        self == other.as_str()
    }
}

impl<'a> PartialEq<&'a str> for AcceptEncoding {
    fn eq(&self, other: &&'a str) -> bool {
        self.as_str() == *other
    }
}

impl<'a> PartialEq<AcceptEncoding> for &'a str {
    fn eq(&self, other: &AcceptEncoding) -> bool {
        *self == other.as_str()
    }
}

