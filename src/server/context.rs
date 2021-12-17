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

use std::borrow::Borrow;
use std::collections::HashMap;
use std::hash::Hash;

use crate::{ContentType, Cookie, Header, Request, Response, Status, Version};
use crate::http::url::authority::{Addr, Userinfo};
use crate::http::values::FileHeader;
use crate::utils::errors::StarryResult;

#[derive(Debug)]
pub struct Context {
    request: Request,
    response: Response,
    fields: HashMap<String, String>,
}

/// request相关
impl Context {
    pub(crate) fn new(request: Request, fields: HashMap<String, String>) -> Self {
        let version = request.version();
        let connection = !request.close;
        Context { request, response: Response::new(version, connection), fields }
    }

    // pub fn get_request(&self) -> StarryResult<Request> {
    //     self.request.try_clone()
    // }

    pub fn req_header(&self) -> Header {
        self.request.header()
    }

    pub fn req_header_get<K: ?Sized>(&self, k: &K) -> Option<String> where
        K: Borrow<K>,
        K: Hash + Eq,
        String: Borrow<K>, {
        self.request.header_get(k)
    }

    pub fn userinfo(&self) -> Option<Userinfo> {
        self.request.url.authority.userinfo()
    }

    pub fn path(&self) -> String {
        self.request.url.location.path()
    }

    pub fn client_addr(&self) -> Addr {
        self.request.url.authority.addr()
    }

    /// 如果请求头指示客户端正在发起websocket握手，则IsWebsocket返回true
    ///
    /// WebSocket, Http 2的协议升级过程，都需要Connection，Upgrade两个字端来联合完成。
    /// 比如初始化WebSocket请求：
    ///
    /// ```header
    /// Host: echo.websocket.org
    /// Connection: Upgrade
    /// Upgrade: websocket
    /// ```
    pub fn is_web_socket(&self) -> bool {
        match self.req_header_get("Connection") {
            Some(src) => if src.ne("upgrade") {
                return false;
            },
            None => return false,
        }
        match self.req_header_get("Upgrade") {
            Some(src) => if src.eq("websocket") {
                return true;
            },
            None => return false,
        }
        false
    }

    pub fn req_cookies(&self) -> Vec<Cookie> {
        self.request.cookies()
    }

    pub fn req_cookie_get(&self, cookie_name: &str) -> Option<Cookie> {
        self.request.cookie_get(cookie_name)
    }

    /// 返回对应于请求表单中定义参数值的引用。
    pub fn get_form_value<K: ?Sized>(&mut self, k: &K) -> StarryResult<Option<String>> where
        K: Borrow<K>,
        K: Hash + Eq,
        String: Borrow<K>, {
        match self.request.form()?.get(k) {
            Some(src) => Ok(Some(src.clone())),
            None => Ok(None)
        }
    }

    /// 返回对应于请求表单中定义的参数存在性。
    pub fn have_form_value<K: ?Sized>(&mut self, k: &K) -> StarryResult<bool> where
        K: Borrow<K>,
        K: Hash + Eq,
        String: Borrow<K>, {
        match self.request.form()?.get(k) {
            Some(_) => Ok(true),
            None => Ok(false),
        }
    }

    /// 请求表单中定义的参数数量
    pub fn count_form_values(&mut self) -> StarryResult<usize> {
        Ok(self.request.form()?.len())
    }

    /// 返回对应于URI请求参数中定义参数值的引用。
    pub fn get_param_value<K: ?Sized>(&self, k: &K) -> Option<String> where
        K: Borrow<K>,
        K: Hash + Eq,
        String: Borrow<K>, {
        match self.request.form_param.get(k) {
            Some(src) => Some(src.clone()),
            None => None
        }
    }

    /// 返回对应于URI请求参数中定义的参数存在性。
    pub fn have_param_value<K: ?Sized>(&self, k: &K) -> bool where
        K: Borrow<K>,
        K: Hash + Eq,
        String: Borrow<K>, {
        match self.request.form_param.get(k) {
            Some(_) => true,
            None => false,
        }
    }

    /// URI请求参数中定义的参数数量
    pub fn count_param_values(&self) -> usize {
        self.request.form_param.len()
    }

    /// 返回对应于URI资源路径中定义参数值的引用。
    /// 键可以是映射的键类型的任何借用形式，但是借用形式上的Hash和Eq必须与键类型匹配。
    pub fn get_field<K: ?Sized>(&self, k: &K) -> Option<String> where
        K: Borrow<K>,
        K: Hash + Eq,
        String: Borrow<K>, {
        match self.fields.get(k) {
            Some(src) => Some(src.clone()),
            None => None
        }
    }

    /// 返回对应于URI资源路径中定义的参数存在性。
    pub fn have_field<K: ?Sized>(&self, k: &K) -> bool where
        K: Borrow<K>,
        K: Hash + Eq,
        String: Borrow<K>, {
        match self.fields.get(k) {
            Some(_) => true,
            None => false,
        }
    }

    /// URI资源路径中定义的参数数量
    pub fn count_fields(&self) -> usize {
        self.fields.len()
    }

    /// 返回对应于请求表单中定义参数对应附件的引用。
    pub fn get_form_file<K: ?Sized>(&mut self, k: &K) -> StarryResult<Option<FileHeader>> where
        K: Borrow<K>,
        K: Hash + Eq,
        String: Borrow<K>, {
        Ok(self.request.multipart_form()?.get(k))
    }
}

/// response相关
impl Context {
    pub fn get_response(&self) -> Response {
        self.response.clone()
    }

    pub fn resp_add_header(&mut self, k: String, v: String) {
        self.response.add_header(k, v)
    }

    pub fn resp_add_header_str(&mut self, k: &str, v: &str) {
        self.response.add_header_str(k, v)
    }

    pub fn resp_set_header(&mut self, k: String, v: String) {
        self.response.set_header(k, v)
    }

    pub fn resp_set_header_str(&mut self, k: &str, v: &str) {
        self.response.set_header_str(k, v)
    }

    pub fn resp_add_set_cookie(&mut self, cookie: Cookie) {
        self.response.add_set_cookie(cookie)
    }

    pub fn resp_status(&mut self, status: Status) {
        self.response.status(status)
    }

    pub fn resp_version(&mut self, version: Version) {
        self.response.version(version)
    }

    pub fn resp_content_type(&mut self, src: ContentType) {
        self.response.set_content_type(src)
    }

    pub fn resp_set_body(&mut self, body: Vec<u8>) {
        self.response.write(body)
    }

    pub fn resp_set_body_bytes(&mut self, body: &'static [u8]) {
        self.response.write_bytes(body)
    }

    pub fn response(&mut self) -> StarryResult<()> {
        self.request.response(self.response.clone())
    }
}
