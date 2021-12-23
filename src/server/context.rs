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
use std::net::TcpStream;

use crate::{Header, Request, Response, Status, Version};
use crate::http::header::{ContentType, Cookie};
use crate::http::url::authority::{Addr, Userinfo};
use crate::http::values::FileHeader;
use crate::utils::errors::StarryResult;

#[derive(Debug)]
pub struct Context {
    request: Request<TcpStream>,
    response: Response,
    fields: HashMap<String, String>,
    /// 是否已经执行过response方法
    pub(crate) executed: bool,
}

/// request相关
impl Context {
    pub(crate) fn new(request: Request<TcpStream>, fields: HashMap<String, String>, compress: bool) -> Self {
        let version = request.version();
        let connection = !request.close;
        Context { request, response: Response::new(version, connection, compress), fields, executed: false }
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

    pub fn req_userinfo(&self) -> Option<Userinfo> {
        self.request.url.authority.userinfo()
    }

    pub fn req_path(&self) -> String {
        self.request.url.location.path()
    }

    pub fn req_client_addr(&self) -> Addr {
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
    pub fn req_is_web_socket(&self) -> bool {
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
    pub fn req_form<K: ?Sized>(&mut self, k: &K) -> StarryResult<Option<String>> where
        K: Borrow<K>,
        K: Hash + Eq,
        String: Borrow<K>, {
        self.request.form_value(k)
    }

    /// 返回对应于请求表单中定义的参数存在性。
    pub fn req_have_form<K: ?Sized>(&mut self, k: &K) -> StarryResult<bool> where
        K: Borrow<K>,
        K: Hash + Eq,
        String: Borrow<K>, {
        self.request.have_form_value(k)
    }

    /// 请求表单中定义的参数数量
    pub fn req_count_form(&mut self) -> StarryResult<usize> {
        self.request.count_form_value()
    }

    /// 返回对应于URI请求参数中定义参数值的引用。
    pub fn req_param<K: ?Sized>(&self, k: &K) -> Option<String> where
        K: Borrow<K>,
        K: Hash + Eq,
        String: Borrow<K>, {
        self.request.param_value(k)
    }

    /// 返回对应于URI请求参数中定义的参数存在性。
    pub fn req_have_param<K: ?Sized>(&self, k: &K) -> bool where
        K: Borrow<K>,
        K: Hash + Eq,
        String: Borrow<K>, {
        self.request.have_param_value(k)
    }

    /// URI请求参数中定义的参数数量
    pub fn req_count_param(&self) -> usize {
        self.request.count_param_value()
    }

    /// 返回对应于URI资源路径中定义参数值的引用。
    /// 键可以是映射的键类型的任何借用形式，但是借用形式上的Hash和Eq必须与键类型匹配。
    pub fn req_field<K: ?Sized>(&self, k: &K) -> Option<String> where
        K: Borrow<K>,
        K: Hash + Eq,
        String: Borrow<K>, {
        match self.fields.get(k) {
            Some(src) => Some(src.clone()),
            None => None
        }
    }

    /// 返回对应于URI资源路径中定义的参数存在性。
    pub fn req_have_field<K: ?Sized>(&self, k: &K) -> bool where
        K: Borrow<K>,
        K: Hash + Eq,
        String: Borrow<K>, {
        match self.fields.get(k) {
            Some(_) => true,
            None => false,
        }
    }

    /// URI资源路径中定义的参数数量
    pub fn req_count_fields(&self) -> usize {
        self.fields.len()
    }

    /// 返回对应于请求表单中定义参数对应附件的引用。
    pub fn req_form_file<K: ?Sized>(&mut self, k: &K) -> StarryResult<Option<FileHeader>> where
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

    pub fn resp_add_cookie(&mut self, cookie: Cookie) {
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

    pub fn resp_body(&mut self, body: Vec<u8>) {
        self.response.write(body, self.request.accept_encoding.clone())
    }

    pub fn resp_body_slice(&mut self, body: &'static [u8]) {
        self.response.write_slice(body, self.request.accept_encoding.clone())
    }

    pub fn resp_bodies(&mut self, body: Vec<u8>, content_type: ContentType) {
        self.response.write_type(body, content_type, self.request.accept_encoding.clone())
    }

    pub fn resp_body_slices(&mut self, body: &'static [u8], content_type: ContentType) {
        self.response.write_slice_type(body, content_type, self.request.accept_encoding.clone())
    }

    pub fn response(&mut self) {
        self.executed = true;
        match self.request.response(self.response.clone()) {
            Ok(()) => {}
            Err(err) => log::error!("response failed! {}", err)
        }
    }
}
