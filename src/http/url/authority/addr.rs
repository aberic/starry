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

use std::fmt::{Display, Formatter};
use std::ops::Add;

/// Addr类型是URL的服务器资源细节的不可变封装。
///
/// Addr在url中所处定位参考如下：
/// ```notrust
///  abc://username:password@example.com:123/path/data?key=value&key2=value2#fragid1
///  |-|  |---------------|  |-------------||--------| |-------------------| |-----|
///   |           |                |            |               |              |
///   |       UserInfo           Addr         Path            Query         Fragment
///   |   |--------------------------------||--------------------------------------|
///   |                  |                                      |
/// Scheme          Authority                                Location
/// ```
#[derive(Clone, Debug)]
pub struct Addr {
    host: String,
    port: u16,
}

impl Addr {
    pub fn host(&self) -> String {
        self.host.clone()
    }

    pub fn port(&self) -> u16 {
        self.port
    }

    /// 通过已知参数获取Addr
    pub fn new(host: String) -> Addr {
        Addr { host, port: 80 }
    }

    /// 通过已知参数获取Addr
    pub(crate) fn from(host: String, port: u16) -> Addr {
        Addr { host, port }
    }

    /// 字符串返回标准形式的“host:port”的Addr
    pub(crate) fn to_string(&self) -> String {
        self.host.clone().add(":").add(self.port.to_string().as_str())
    }
}

impl Default for Addr {
    fn default() -> Self {
        Addr { host: "".to_string(), port: 0 }
    }
}

impl Display for Addr {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.to_string())
    }
}

#[cfg(test)]
mod addr_test {
    use crate::http::url::authority::Addr;

    impl Addr {
        /// 通过已知参数获取Addr
        fn from_str(host: &str, port: u16) -> Addr {
            let host = String::from(host);
            Addr { host, port }
        }
    }

    #[test]
    fn to_string() {
        let a = Addr::from_str("127.0.0.1", 8888);
        assert_eq!(a.to_string(), String::from("127.0.0.1:8888"));

        let a = Addr::from(String::from("127.0.0.1"), 8888);
        assert_eq!(a.to_string(), String::from("127.0.0.1:8888"));
    }
}

