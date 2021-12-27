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

use crate::http::url::authority::{Addr, Userinfo};

/// Authority类型是URL的用户名和密码细节的不可变封装。
/// 现有的Authority值保证设置一个用户名(可能为空，这是[`RFC2396`]所允许的)和一个可选的密码。
///
/// Authority在url中所处定位参考如下：
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
pub struct Authority {
    /// Userinfo类型是URL的用户名和密码细节的不可变封装。
    userinfo: Option<Userinfo>,
    /// Addr类型是URL的服务器资源细节的不可变封装。
    pub(crate) addr: Addr,
}

impl Authority {
    pub(crate) fn new(userinfo: Option<Userinfo>, addr: Addr) -> Authority {
        Authority { userinfo, addr }
    }

    pub fn userinfo(&self) -> Option<Userinfo> {
        self.userinfo.clone()
    }

    pub fn addr(&self) -> Addr {
        self.addr.clone()
    }
}
