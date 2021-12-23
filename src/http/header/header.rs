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

use crate::{Values, Version};
use crate::header::AcceptEncoding;
use crate::http::header::{ContentType, Cookie};
use crate::http::url::authority::Userinfo;
use crate::utils::errors::StarryResult;

pub type Header = Values;

impl Header {
    /// read_cookie 解析头header中的所有“Cookie”值，并返回成功解析的Cookie。
    /// 如果filter不为空，则只返回该名称的cookie
    pub(crate) fn read_cookies(&self) -> StarryResult<Vec<Cookie>> {
        Cookie::read_cookies(self.vec("Cookie"), "")
    }

    pub(crate) fn read_set_cookies(&self) -> StarryResult<Vec<Cookie>> {
        Cookie::read_set_cookies(self.vec("Set-Cookie"))
    }

    pub fn add_set_cookie(&mut self, cookie: Cookie) {
        self.add(String::from("Set-Cookie"), cookie.to_string())
    }

    pub(crate) fn set_content_length(&mut self, content_length: usize) {
        self.set("Content-Length".to_string(), content_length.to_string())
    }

    pub(crate) fn get_content_length(&self) -> Option<String> {
        self.get("Content-Length")
    }

    pub(crate) fn del_content_length(&mut self) -> Option<String> {
        self.del("Content-Length")
    }

    pub(crate) fn get_userinfo(&self) -> StarryResult<Option<Userinfo>> {
        match self.get("Authorization") {
            Some(src) => match Userinfo::from_basic(src.to_string()) {
                Ok(src) => Ok(Some(src)),
                Err(err) => Err(err)
            },
            None => Ok(None)
        }
    }

    pub(crate) fn get_host(&self) -> Option<String> {
        self.get("Host")
    }

    pub(crate) fn set_connection(&mut self) {
        self.set_str("Connection", "keep-alive")
    }

    pub(crate) fn set_content_type(&mut self, src: ContentType) {
        self.set_str("Content-Type", src.as_str())
    }

    pub(crate) fn get_content_type(&self) -> Option<String> {
        self.get("Content-Type")
    }

    pub(crate) fn del_content_type(&mut self) -> Option<String> {
        self.del("Content-Type")
    }

    pub(crate) fn set_accept_encoding(&mut self, encode_type: AcceptEncoding) {
        if encode_type.ne("") && encode_type.ne("br") {
            self.set("Accept-Encoding".to_string(), encode_type.to_string())
        }
    }

    pub(crate) fn get_accept_encoding(&self) -> Option<AcceptEncoding> {
        match self.get("Accept-Encoding") {
            Some(src) => AcceptEncoding::best(src),
            None => None
        }
    }

    // pub(crate) fn del_accept_encoding(&mut self) -> Option<String> {
    //     self.del("Accept-Encoding")
    // }

    pub(crate) fn check_close(&mut self, version: &Version, remove: bool) -> bool {
        if version.major() < 1 {
            return true;
        }
        let header_has_close = self.contain("close");
        if version.eq("HTTP/1.0") {
            return header_has_close || !self.contain("keep-alive");
        }

        if header_has_close && remove {
            self.del("Connection").unwrap();
        }
        header_has_close
    }
}

