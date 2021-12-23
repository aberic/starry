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

use crate::Values;
// use crate::values::RequestValues;

/// Location类型是URL中服务器所属本地资源细节的不可变封装。
///
/// Location在url中所处定位参考如下：
/// ```notrust
///  abc://username:password@example.com:123/path/data?key=value&key2=value2#fragid1
///  |-|  |---------------|  |-------------| |-------| |-------------------| |-----|
///   |           |                |            |               |              |
///   |       UserInfo           Addr         Path            Query         Fragment
///   |   |--------------------------------||--------------------------------------|
///   |                  |                                      |
/// Scheme          Authority                                Location
/// ```
///
/// 参考[`RFC2396`](https://datatracker.ietf.org/doc/html/rfc2396#section-3.2)
#[derive(Clone)]
pub struct Location {
    path: String,
    query: Values,
    fragment: Option<String>,
}

impl Location {
    pub(crate) fn new() -> Location {
        Location { path: String::from("/"), query: Values::new(), fragment: None }
    }

    pub fn path(&self) -> String {
        self.path.clone()
    }

    pub fn query(&self) -> Values {
        self.query.clone()
    }

    pub fn fragment(&self) -> Option<String> {
        self.fragment.clone()
    }

    // path/data?key=value&key2=value2#test
    pub(crate) fn from_bytes(src: Vec<u8>) -> Location {
        let mut step = 1; // 1:path -> 2:query -> 3:fragment
        let mut data: Vec<u8> = vec![];
        let mut key: String = String::from("");
        let mut path: String = String::from("");
        let mut query = Values::new();
        for b in src {
            match b {
                b'/' => {
                    if step == 1 {
                        path = path + String::from_utf8_lossy(&data).as_ref() + "/";
                        data.clear()
                    } else {
                        data.push(b)
                    }
                }
                b'?' => {
                    if step == 1 {
                        path = path + String::from_utf8_lossy(&data).as_ref();
                        step = 2;
                        data.clear()
                    } else {
                        data.push(b)
                    }
                }
                b'=' => {
                    if step == 2 {
                        key = String::from_utf8_lossy(&data).to_string();
                        data.clear()
                    } else {
                        data.push(b)
                    }
                }
                b'&' => {
                    query.set(key.clone(), String::from_utf8_lossy(&data).to_string());
                    key.clear();
                    data.clear()
                }
                b'\r' => {
                    if step == 1 {
                        path = path + String::from_utf8_lossy(&data).as_ref();
                    } else if step == 2 {
                        query.set(key.clone(), String::from_utf8_lossy(&data).to_string());
                    }
                    break
                }
                _ => data.push(b)
            }
        }
        Location { path, query, fragment: None }
    }

    fn query_string(&self) -> Option<String> {
        let size = self.query.len();
        if size == 0 {
            None
        } else {
            let check = size - 1;
            let mut query: String = String::from("");
            for (i, (k, v)) in self.query.map().iter().enumerate() {
                if check == i {
                    query = query + k + "=" + v[0].as_str();
                } else {
                    query = query + k + "=" + v[0].as_str() + "&";
                }
            }
            Some(query)
        }
    }
}

impl ToString for Location {
    fn to_string(&self) -> String {
        let path;
        match self.path.as_str() {
            "/" => path = String::new(),
            _ => path = self.path.clone()
        }
        let data: String;
        match self.query_string() {
            Some(query) => match self.fragment.clone() {
                Some(fragment) => data = path + "?" + &query + "#" + &fragment,
                None => data = path + "?" + &query
            },
            None => match self.fragment.clone() {
                Some(fragment) => data = path + "#" + &fragment,
                None => data = path
            },
        }
        data
    }
}

impl fmt::Debug for Location {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.to_string())
    }
}