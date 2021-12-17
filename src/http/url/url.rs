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

use crate::http::url::{Authority, Location, Scheme};
use crate::http::url::authority::Addr;

/// URL表示已解析的URL。
/// 表示的一般形式是:\[scheme]\[://]\[userinfo@]\[addr]\[/path]\[?查询]\[#片段]，参考如下：
///
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
pub struct URL {
    /// Scheme url提供的一种机制，它可以由应用程序注册，然后其他程序通过url scheme来调用该应用程序
    pub(crate) scheme: Scheme,
    /// Authority类型是URL的用户名和密码细节的不可变封装。
    pub(crate) authority: Authority,
    /// Location类型是URL中服务器所属本地资源细节的不可变封装。
    pub(crate) location: Location,
}

impl URL {
    pub fn default() -> URL {
        URL{
            scheme: Scheme::HTTP,
            authority: Authority::new(None, Addr::new(String::new())),
            location: Location::new()
        }
    }

    pub fn scheme(&self) -> Scheme {
        self.scheme.clone()
    }

    pub fn authority(&self) -> Authority {
        self.authority.clone()
    }

    pub fn location(&self) -> Location {
        self.location.clone()
    }

    // http://http:password@example.com:123/path/data?key=value&key2=value2#fragid1
    // http://example.http:123/path/data?key=value&key2=value2#fragid1
    // http://user:password@example.com:123/path/data?key=value&key2=value2#fragid1
    // https://users:password@example.com:123/path/data?key=value&key2=value2#fragid1
    pub(crate) fn from(scheme: Scheme, authority: Authority, location: Location) -> URL {
        URL {
            scheme,
            authority,
            location,
        }
    }
}

