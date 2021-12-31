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

use crate::utils::cryptos::Base64;
use crate::utils::cryptos::base64::Base64Decoder;
use crate::utils::errors::{Errs, StarryResult};
use crate::utils::Strings;

/// Userinfo类型是URL的用户名和密码细节的不可变封装。
/// 现有的Userinfo值保证设置一个用户名(可能为空，这是[`RFC2396`]所允许的)和一个可选的密码。
///
/// Userinfo在url中所处定位参考如下：
/// ```notrust
///  abc://username:password@example.com:123/path/data?key=value&key2=value2#fragid1
///  |-|  |---------------|  |-------------||--------| |-------------------| |-----|
///   |           |                |            |               |              |
///   |       UserInfo           Addr         Path            Query         Fragment
///   |   |--------------------------------||--------------------------------------|
///   |                  |                                      |
/// Scheme          Authority                                Location
/// ```
///
/// [`RFC2396`]: https://datatracker.ietf.org/doc/html/rfc2396#section-3.2
#[derive(Clone, Debug)]
pub struct Userinfo {
    username: String,
    password: String,
}

impl Userinfo {
    pub(crate) fn new(username: String, password: String) -> Self {
        Self{ username, password }
    }

    pub fn username(&self) -> String {
        self.username.clone()
    }

    pub fn password(&self) -> String {
        self.password.clone()
    }

    /// 通过已知参数获取Userinfo
    pub(crate) fn from_basic(mut authorization: String) -> StarryResult<Userinfo> {
        authorization = authorization[6..].to_string();
        let bytes = Base64::decode(authorization)?;
        let mut username = String::from("");
        let password;
        let mut data = vec![];
        for b in bytes {
            match b {
                b':' => {
                    username = Strings::from_utf8(data.clone())?;
                    data.clear()
                }
                _ => data.push(b)
            }
        }
        if username.is_empty() {
            return Err(Errs::str("username can not be empty!"));
        }
        password = Strings::from_utf8(data.clone())?;
        Ok(Userinfo { username, password })
    }
}

#[cfg(test)]
mod userinfo_test {
    use std::ops::Add;
    use crate::http::url::authority::Userinfo;
    use crate::utils::cryptos::Base64;
    use crate::utils::cryptos::base64::Base64Encoder;

    impl Userinfo {
        /// 通过已知参数获取Userinfo
        fn from(username: String, password: String) -> Userinfo {
            Userinfo { username, password }
        }

        /// 通过已知参数获取Userinfo
        fn from_str(username: &str, password: &str) -> Userinfo {
            Userinfo { username: String::from(username), password: String::from(password) }
        }

        /// 字符串返回标准形式的“username:password”编码的用户信息。
        fn to_string(&self) -> String {
            self.username.clone().add(":").add(&self.password)
        }

        fn base64(&self) -> String {
            let src = self.to_string();
            let bytes = src.as_bytes();
            Base64::encode(bytes)
        }
    }

    #[test]
    fn to_string() {
        let u = Userinfo::from_str("user", "pass");
        assert_eq!(u.to_string(), String::from("user:pass"));

        let u = Userinfo::from(String::from("user"), String::from("pass"));
        assert_eq!(u.to_string(), String::from("user:pass"))
    }

    #[test]
    fn base64() {
        let u = Userinfo::from_str("user", "password");
        assert_eq!(u.base64(), "dXNlcjpwYXNzd29yZA==");
    }
}
