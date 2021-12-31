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
use crate::http::url::authority::{Addr, Userinfo};
use crate::utils::errors::{Errs, StarryResult};
use std::borrow::BorrowMut;

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
    pub(crate) fn new(scheme: Scheme, authority: Authority, location: Location) -> URL {
        URL {
            scheme,
            authority,
            location,
        }
    }

    pub(crate) fn scheme(&self) -> Scheme {
        self.scheme.clone()
    }

    pub(crate) fn authority(&self) -> Authority {
        self.authority.clone()
    }

    pub(crate) fn location(&self) -> Location {
        self.location.clone()
    }

    // http://http:password@example.com:123/path/data?key=value&key2=value2#fragid1
    // http:password@example.com:123/path/data?key=value&key2=value2#fragid1
    // https:password@example.com:123/path/data?key=value&key2=value2#fragid1
    // http://example.http:123/path/data?key=value&key2=value2#fragid1
    // http://user:password@example.com:123/path/data?key=value&key2=value2#fragid1
    // https://users:password@example.com:123/path/data?key=value&key2=value2#fragid1
    // users:password@example.com:123/path/data?key=value&key2=value2#fragid1
    pub(crate) fn from(url: &str) -> StarryResult<URL> {
        let mut scheme: Scheme = Scheme::HTTP;
        let authority: Authority;
        let mut userinfo: Option<Userinfo> = None;
        // 表示 username/host，与password/port成对
        let mut key: String = "".to_string();
        let mut addr: Addr = Addr::from("".to_string(), 0);
        let location: Location;
        let mut tmp: Vec<u8> = vec![];
        let mut read_count = 0;
        let mut cs = url.as_bytes().iter();
        // 区分http协议和用户名起始
        loop {
            match cs.next() {
                Some(src) => match read_count {
                    0 => if src.eq(&b'h') {
                        // 符合"http"协议特征，继续解析
                        tmp.push(b'h');
                        read_count += 1;
                    } else {
                        // 非"http"协议起始，直接略过协议解析，并指定协议为HTTP
                        scheme = Scheme::HTTP;
                        break;
                    }
                    1 => if src.eq(&b't') {
                        // 符合"http"协议特征，继续解析
                        tmp.push(b't');
                        read_count += 1;
                    } else {
                        // 非"http"协议起始，直接略过协议解析，并指定协议为HTTP
                        scheme = Scheme::HTTP;
                        break;
                    }
                    2 => if src.eq(&b't') {
                        // 符合"http"协议特征，继续解析
                        tmp.push(b't');
                        read_count += 1;
                    } else {
                        // 非"http"协议起始，直接略过协议解析，并指定协议为HTTP
                        scheme = Scheme::HTTP;
                        break;
                    }
                    3 => if src.eq(&b'p') {
                        // 符合"http"协议特征，继续解析
                        tmp.push(b'p');
                        read_count += 1;
                    } else {
                        // 非"http"协议起始，直接略过协议解析，并指定协议为HTTP
                        scheme = Scheme::HTTP;
                        break;
                    }
                    4 => if src.eq(&b':') {
                        // 目前符合"http://"协议特征，也符合"http:pass"用户名和密码特征
                        // 继续看后续是否连续"//"
                        match cs.next() {
                            Some(src) => if src.eq(&b'/') {
                                // 首字符为"/"，后续只判断是否为http协议
                                if cs.next().unwrap_or(&b'_').eq(&b'/') {
                                    scheme = Scheme::HTTP;
                                    tmp.clear();
                                    break;
                                } else {
                                    return Err(Errs::str("url protocol parse http failed!"));
                                }
                            } else {
                                // 首字符不为"/"，后续只做用户名解析
                                key.push_str(String::from_utf8_lossy(tmp.as_slice()).as_ref());
                                // 清空已有部分
                                tmp.clear();
                                // 追加新解析内容
                                tmp.push(*src);
                                break;
                            },
                            None => return Err(Errs::str("url protocol parse http none failed!"))
                        }
                    } else if src.eq(&b's') {
                        // 符合"https"协议特征，继续解析
                        tmp.push(b's');
                        read_count += 1;
                    } else {
                        // 非"http"协议起始，直接略过协议解析，并指定协议为HTTP
                        scheme = Scheme::HTTP;
                        tmp.push(*src);
                        break;
                    }
                    5 => if src.eq(&b':') {
                        // 目前符合"https://"协议特征，也符合"https:pass"用户名和密码特征
                        // 继续看后续是否连续"//"
                        match cs.next() {
                            Some(src) => if src.eq(&b'/') {
                                // 首字符为"/"，后续只判断是否为http协议
                                if cs.next().unwrap_or(&b'_').eq(&b'/') {
                                    scheme = Scheme::HTTPS;
                                    tmp.clear();
                                    break;
                                } else {
                                    return Err(Errs::str("url protocol parse https failed!"));
                                }
                            } else {
                                scheme = Scheme::HTTP;
                                // 首字符不为"/"，后续只做用户名解析
                                key.push_str(String::from_utf8_lossy(tmp.as_slice()).as_ref());
                                // 清空已有部分
                                tmp.clear();
                                // 追加新解析内容
                                tmp.push(*src);
                                break;
                            },
                            None => return Err(Errs::str("url protocol parse https none failed!"))
                        }
                    } else {
                        // 非"https"协议起始，直接略过协议解析，并指定协议为HTTP
                        scheme = Scheme::HTTP;
                        tmp.push(*src);
                        break;
                    },
                    _ => return Err(Errs::str("url protocol parse src failed!"))
                }
                None => return Err(Errs::str("url protocol parse from failed!"))
            }
        }
        // 继续对用户名及后续起始判断
        loop {
            match cs.next() {
                Some(src) => match src {
                    b':' => if key.is_empty() { // 用户名为空则赋值
                        // 存在两种情况
                        // 情况1：users:password@
                        // 情况2：example.com:123/
                        key.push_str(String::from_utf8_lossy(tmp.as_slice()).as_ref());
                        tmp.clear();
                    } else { // 用户名不为空还能解析到":"，则url格式错误
                        return Err(Errs::str("url protocol parse username twice!"));
                    }
                    b'@' => if key.is_empty() { // 如果用户名为空，则url格式错误
                        // 解析到"@"则只存在情况1：users:password@
                        return Err(Errs::str("url protocol parse userinfo failed!"));
                    } else { // 如果用户名不为空，则赋值password
                        // 解析到"@"则只存在情况1：users:password@
                        userinfo = Some(Userinfo::new(key.clone(), String::from_utf8_lossy(tmp.as_slice()).to_string()));
                        tmp.clear();
                        key.clear();
                        // 跳出循环继续解析addr
                        break;
                    }
                   b'/' => if key.is_empty() {
                        // 解析到"/"则只存在情况2：example.com:123/
                        // 此时如果用户名为空，则port默认80
                        key.push_str(String::from_utf8_lossy(tmp.as_slice()).as_ref());
                        addr = Addr::from(key.clone(), 80);
                        tmp.clear();
                        tmp.push(b'/');
                        // 跳出循环继续解析location
                        break;
                    } else { // 如果用户名不为空，则赋值port
                        // 解析到"/"则只存在情况2：example.com:123/
                        match String::from_utf8_lossy(tmp.as_slice()).to_string().parse::<u16>() {
                            Ok(src) => {
                                addr = Addr::from(key.clone(), src);
                                tmp.clear();
                                // 追加新解析内容
                                tmp.push(b'/');
                                // 跳出循环继续解析location
                                break;
                            }
                            Err(err) => return Err(Errs::strs("url protocol parse port failed!", err))
                        }
                    }
                    _ => tmp.push(*src)
                },
                None => return Err(Errs::str("url protocol parse username none failed!"))
            }
        }
        match userinfo {
            Some(_) => {
                // 存在userinfo，需先解析出addr
                loop {
                    match cs.next() {
                        Some(src) => match src {
                            b':' => if key.is_empty() { // 如果host为空，解析到":"说明存在port，继续
                                key.push_str(String::from_utf8_lossy(tmp.as_slice()).as_ref());
                                tmp.clear();
                            } else { // 如果host存在，解析到":"说明url格式错误
                                return Err(Errs::str("url protocol addr parse host failed!"));
                            }
                            b'/' => if key.is_empty() { // 如果host为空，解析到"/"说明不存在port
                                key.push_str(String::from_utf8_lossy(tmp.as_slice()).as_ref());
                                addr = Addr::from(key.clone(), 80);
                                tmp.clear();
                                // 追加新解析内容
                                tmp.push(b'/');
                                // 跳出循环继续解析location
                                break;
                            } else { // 如果host存在，解析到"/"说明目前在处理port
                                match String::from_utf8_lossy(tmp.as_slice()).to_string().parse::<u16>() {
                                    Ok(src) => {
                                        addr = Addr::from(key.clone(), src);
                                        tmp.clear();
                                        // 追加新解析内容
                                        tmp.push(b'/');
                                        // 跳出循环继续解析location
                                        break;
                                    }
                                    Err(err) => return Err(Errs::strs("url protocol addr parse port failed!", err))
                                }
                            }
                            _ => tmp.push(*src)
                        },
                        None => return Err(Errs::str("url protocol parse addr none failed!"))
                    }
                }
            }
            None => if addr.port() == 0 {
                // 不存在userinfo，addr已经解析出来，后续追加所有字节由location自行解析即可
                // 如果addr的port为0，则addr也未经过解析，返回错误
                return Err(Errs::str("url protocol parse userinfo&addr none failed!"));
            },
        }
        tmp.append(cs.as_ref().to_vec().borrow_mut());
        // loop {
        //     match cs.next() {
        //         Some(src) => tmp.push(*src),
        //         None => break
        //     }
        // }
        tmp.push(b'\r');
        location = Location::from_bytes(tmp);
        authority = Authority::new(userinfo, addr);
        Ok(URL::new(scheme, authority, location))
    }
}

impl Default for URL {
    fn default() -> Self {
        URL {
            scheme: Scheme::HTTP,
            authority: Authority::new(None, Addr::new(String::new())),
            location: Location::new(),
        }
    }
}

#[cfg(test)]
mod url_test {
    use crate::URL;

    #[test]
    fn url_trans() {
        let u1 = "http://http:password@example.com:123/path/data?key=value&key2=value2#fragid1";
        let u2 = "http:password@example.com:123/path/data?key=value&key2=value2#fragid1";
        let u3 = "https:password@example.com:123/path/data?key=value&key2=value2#fragid1";
        let u4 = "http://example.http:123/path/data?key=value&key2=value2#fragid1";
        let u5 = "http://user:password@example.com:123/path/data?key=value&key2=value2#fragid1";
        let u6 = "https://users:password@example.com:123/path/data?key=value&key2=value2#fragid1";
        let u7 = "users:password@example.com:123/path/data?key=value&key2=value2#fragid1";
        let u8 = "example.com:123/path/data?key=value&key2=value2#fragid1";
        let u9 = "example.com/path/data?key=value&key2=value2#fragid1";

        println!("u1 = {:#?}", URL::from(u1));
        println!("u2 = {:#?}", URL::from(u2));
        println!("u3 = {:#?}", URL::from(u3));
        println!("u4 = {:#?}", URL::from(u4));
        println!("u5 = {:#?}", URL::from(u5));
        println!("u6 = {:#?}", URL::from(u6));
        println!("u7 = {:#?}", URL::from(u7));
        println!("u8 = {:#?}", URL::from(u8));
        println!("u9 = {:#?}", URL::from(u9));
    }
}

