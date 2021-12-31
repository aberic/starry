/*
 * Copyright (c) $originalComment.match("Copyright \(c\) (\d+)", 1, "-")2021. Aberic - All Rights Reserved.
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

use std::borrow::{Borrow, BorrowMut};
use std::collections::HashMap;
use std::fmt::Debug;
use std::hash::Hash;
use std::io::{Read, Write};
use std::slice::Iter;
use std::sync::{Arc, RwLock};

use bytes::{BufMut, BytesMut};
use bytes::buf::Writer;

use crate::{Header, Inner, Method, MultipartValues, Request, Response, URL, Values, Version};
use crate::http::header::{AcceptEncoding, ContentType, Cookie};
use crate::http::url::{Authority, Location, Scheme};
use crate::http::url::authority::{Addr, Userinfo};
use crate::http::values::FileHeader;
use crate::http::version::Protocol;
use crate::server::node::{Node, Root};
use crate::utils::errors::{Error, Errs, StarryResult};

pub(crate) const SERVER_TCP_STREAM_HAD_NO_DATA: &str = "server request tcp stream had no data!";

/// [`Request`]的封装
///
/// [`Request`]: crate::Request
#[derive(Debug)]
pub struct Requester<Stream: Read + Write + Debug> {
    pub(crate) request: Request,
    pub(crate) stream: Stream,
}

impl<Stream: Read + Write + Debug> Requester<Stream> {
    /// 通过写入流获取来自客户端的Request
    ///
    /// * root 资源树根结点
    /// * peer 客户端地址信息
    /// * local 本机地址信息
    pub(crate) fn from(stream: Stream, root: Arc<RwLock<Root>>, peer: Addr, local: Addr) -> StarryResult<(Self, Node, HashMap<String, String>)> {
        let mut req = Requester {
            request: Default::default(),
            stream,
        };
        let (node, fields) = req.parse(root, peer, local)?;
        Ok((req, node, fields))
    }
}

impl<Stream: Read + Write + Debug> Requester<Stream> {
    pub fn method(&self) -> Method {
        self.request.method()
    }

    pub fn url(&self) -> URL {
        self.request.url()
    }

    pub fn scheme(&self) -> Scheme {
        self.request.scheme()
    }

    pub fn authority(&self) -> Authority {
        self.request.authority()
    }

    pub fn location(&self) -> Location {
        self.request.location()
    }

    pub fn userinfo(&self) -> Option<Userinfo> {
        self.request.userinfo()
    }

    pub fn addr(&self) -> Addr {
        self.request.addr()
    }

    pub fn path(&self) -> String {
        self.request.path()
    }

    pub fn version(&self) -> Version {
        self.request.version()
    }

    pub fn version_protocol(&self) -> Protocol {
        self.request.version_protocol()
    }

    pub fn version_major(&self) -> u8 {
        self.request.version_major()
    }

    pub fn version_minor(&self) -> u8 {
        self.request.version_minor()
    }

    pub fn header(&self) -> Header {
        self.request.header()
    }

    pub fn header_get<K: ?Sized>(&self, k: &K) -> Option<String> where
        K: Borrow<K>,
        K: Hash + Eq,
        String: Borrow<K>, {
        self.request.header_get(k)
    }

    pub fn content_type(&self) -> Option<ContentType> {
        self.request.content_type()
    }

    pub fn accept_encoding(&self) -> AcceptEncoding {
        self.request.accept_encoding()
    }

    pub fn cookies(&self) -> Vec<Cookie> {
        self.request.cookies()
    }

    pub fn req_cookies(&self) -> Vec<Cookie> {
        self.request.cookies()
    }

    pub fn cookie_get(&self, cookie_name: &str) -> Option<Cookie> {
        for cookie in self.cookies().iter() {
            if cookie.name.eq(cookie_name) {
                return Some(cookie.clone());
            }
        }
        None
    }

    pub fn body(&mut self) -> Vec<u8> {
        self.request.body()
    }

    pub fn form(&mut self) -> StarryResult<Values> {
        if !self.request.body_parse {
            self.parse_body()?;
        }
        Ok(self.request.form.clone())
    }

    pub fn multipart_form(&mut self) -> StarryResult<MultipartValues> {
        if !self.request.body_parse {
            self.parse_body()?;
        }
        Ok(self.request.multipart_form.clone())
    }

    /// 返回对应于请求表单中定义参数值的引用。
    pub fn form_value<K: ?Sized>(&mut self, k: &K) -> StarryResult<Option<String>> where
        K: Borrow<K>,
        K: Hash + Eq,
        String: Borrow<K>, {
        if !self.request.body_parse {
            self.parse_body()?;
        }
        match self.request.form.get(k) {
            Some(src) => Ok(Some(src.clone())),
            None => Ok(None)
        }
    }

    /// 返回对应于请求表单中定义的参数存在性。
    pub fn have_form_value<K: ?Sized>(&mut self, k: &K) -> StarryResult<bool> where
        K: Borrow<K>,
        K: Hash + Eq,
        String: Borrow<K>, {
        if !self.request.body_parse {
            self.parse_body()?;
        }
        match self.request.form.get(k) {
            Some(_) => Ok(true),
            None => Ok(false),
        }
    }

    /// 请求表单中定义的参数数量
    pub fn count_form_value(&mut self) -> StarryResult<usize> {
        if !self.request.body_parse {
            self.parse_body()?;
        }
        Ok(self.request.form.len())
    }

    /// 返回对应于URI请求参数中定义参数值的引用。
    pub fn param_value<K: ?Sized>(&self, k: &K) -> Option<String> where
        K: Borrow<K>,
        K: Hash + Eq,
        String: Borrow<K>, {
        self.request.param_value(k)
    }

    /// 返回对应于URI请求参数中定义的参数存在性。
    pub fn have_param_value<K: ?Sized>(&self, k: &K) -> bool where
        K: Borrow<K>,
        K: Hash + Eq,
        String: Borrow<K>, {
        self.request.have_param_value(k)
    }

    /// URI请求参数中定义的参数数量
    pub fn count_param_value(&self) -> usize {
        self.request.count_param_value()
    }

    /// 返回对应于请求表单中定义参数对应附件的引用。
    pub fn form_file_value<K: ?Sized>(&mut self, k: &K) -> StarryResult<Option<FileHeader>> where
        K: Borrow<K>,
        K: Hash + Eq,
        String: Borrow<K>, {
        if !self.request.body_parse {
            self.parse_body()?;
        }
        Ok(self.request.multipart_form.get(k))
    }

    pub fn client(&self) -> Addr {
        self.request.client()
    }
}

impl<Stream: Read + Write + Debug> Requester<Stream> {
    /// 解析请求行信息
    fn parse(&mut self, root: Arc<RwLock<Root>>, peer: Addr, local: Addr) -> StarryResult<(Node, HashMap<String, String>)> {
        let mut buffer = [0; 1024];
        let mut iter;
        // 当前读取总长度
        let mut count = 0;
        // 剩余待读取数据的总长度
        let mut size;
        match self.stream.read(&mut buffer) {
            Ok(src) => {
                log::trace!("request stream read size = {}", src);
                if src == 0 { // 没有数据进入
                    return Err(Errs::str(SERVER_TCP_STREAM_HAD_NO_DATA));
                }
                size = src;
                iter = buffer.iter()
            }
            Err(err) => return Err(self.interrupt(
                Response::bad_request(),
                Errs::strs("read failed while parse request with error!", err)))
        }

        // 解析请求行信息 POST /path/data?key=value&key2=value2 HTTP/1.1
        let (location, len) = self.parse_request_line(iter.borrow_mut())?;
        // log::trace!("parse_request_line count = {}, method = {}, version = {}", count, self.method, self.version.to_string());
        log::trace!("size = {}, len1 = {}", size, len);
        count += len;

        let node;
        let fields;
        // println!("path = {:#?}, method = {:#?}", location.path(), self.method());
        // 根据资源信息获取请求方法，判断请求有效性，如无效，则放弃后续解析操作
        match root.read().unwrap().fetch(location.path(), self.method()) {
            Some((src1, src2)) => {
                node = src1;
                fields = src2;
            }
            None => return Err(self.interrupt(
                Response::not_found(),
                Errs::str("server fetch node do not exist!")))
        }

        // 解析消息报头
        let len = self.parse_request_header(iter.borrow_mut());
        log::trace!("size = {}, len2 = {}", size, len);
        count += len;

        // 根据已知结果解析请求关联参数
        self.parse_others(location, peer, local)?;

        if size < count {
            return Ok((node, fields))
        }
        size -= count;

        // 读取请求正文
        // 当请求方法为 POST/PUT/PATCH 时需要读取到body中，其它方法没有实体，即便有，也会被丢弃掉
        // 该方法不会解析body内容，body内容解析根据实际情况进行
        // 当用户使用到form数据等情况时，会解析，解析后，body内数据会被清空
        // 用户也可以主动使用body数据，但用户使用后，解析不会再自动进行
        self.fill_body(iter.borrow_mut(), size)?;

        // log::trace!("body = {}", self.body.to_string());
        // self.parse_body()?;
        Ok((node, fields))
    }

    /// 解析请求行信息 POST /path/data?key=value&key2=value2 HTTP/1.1
    ///
    /// usize 读取数据的总长度
    fn parse_request_line(&mut self, iter: &mut Iter<u8>) -> StarryResult<(Location, usize)> {
        // 读取数据的总长度
        let mut count = 0;
        let mut step: u8 = 1;
        let mut location: Location = Location::new();
        let mut data: Vec<u8> = vec![];
        loop {
            match iter.next() {
                Some(b) => {
                    count += 1;
                    match b {
                        b' ' => match step { // POST/GET/...
                            1 => {
                                match Method::from_bytes(data.as_slice()) {
                                    Ok(src) => {
                                        self.request.set_method(src);
                                        data.clear();
                                        step = 2
                                    }
                                    Err(err) => return Err(self.interrupt(
                                        Response::method_not_allowed(),
                                        Errs::strs("parse request failed!", err)))
                                }
                            }
                            2 => { // /path/test/test1/hello/world?key=value&key2=value2\r
                                data.push(b'\r');
                                log::trace!("location = {}", String::from_utf8_lossy(data.as_slice()));
                                location = Location::from_bytes(data.clone());
                                for (key, value) in location.query().map() {
                                    self.request.set_form_param(key, value[0].clone());
                                }
                                data.clear();
                                step = 3
                            }
                            _ => return Err(self.interrupt(
                                Response::bad_request(),
                                Errs::str("parse request failed, does not understand the syntax of the request while step2!")))
                        }
                        b'\r' | b'\n' => match step {
                            3 => match Version::from_bytes(data.as_slice()) { // HTTP/1.1...
                                Ok(src) => {
                                    self.request.set_version(src);
                                    break;
                                }
                                Err(err) => return Err(self.interrupt(
                                    Response::http_version_not_supported(),
                                    Errs::strs("parse request failed!", err)))
                            }
                            _ => return Err(self.interrupt(
                                Response::bad_request(),
                                Errs::str("parse request failed, does not understand the syntax of the request while step3!")))
                        }
                        _ => data.push(*b),
                    }
                }
                None => break
            }
        }
        Ok((location, count))
    }

    /// 解析消息报头
    ///
    /// usize 读取数据的总长度
    fn parse_request_header(&mut self, iter: &mut Iter<u8>) -> usize {
        // 本次读取的字节数
        let mut count = 0;
        let mut data: Vec<u8> = vec![];
        let mut key: String = String::from("");
        // 是否轮到key解析
        let mut key_time = true;
        // 即将开始value解析
        let mut pre_value_time = false;
        // 是否轮到value解析
        let mut value_time = false;
        // 是否结束解析。当连续出现两次"\n\r"后结束解析
        let mut end_time = false;
        loop {
            match iter.next() {
                Some(b) => {
                    count += 1;
                    match b {
                        b':' => if key_time {
                            key = String::from_utf8_lossy(&data).to_string();
                            key_time = false;
                            pre_value_time = true;
                            data.clear()
                        } else {
                            data.push(*b)
                        }
                        b' ' => if key_time {
                            data.push(*b)
                        } else if pre_value_time {
                            end_time = false;
                            pre_value_time = false;
                            value_time = true
                        } else if value_time {
                            data.push(*b)
                        },
                        b'\r' => {}
                        b'\n' => if end_time {
                            break;
                        } else if value_time {
                            self.request.set_header(key.clone(), String::from_utf8_lossy(&data).to_string());
                            key.clear();
                            data.clear();
                            key_time = true;
                            value_time = false;
                            end_time = true
                        }
                        _ => data.push(*b),
                    }
                }
                None => break
            }
        }
        count
    }

    /// 根据已知结果解析请求关联参数
    pub(crate) fn parse_others(&mut self, location: Location, peer: Addr, local: Addr) -> StarryResult<()> {
        let userinfo;
        match self.request.header.get_userinfo() {
            Ok(src) => userinfo = src,
            Err(err) => return Err(self.interrupt(
                Response::bad_request(),
                Errs::strs("parse request failed, userinfo parse error!", err)))
        }
        self.request.set_client(peer);
        self.request.set_url(URL::new(Scheme::HTTP, Authority::new(userinfo, local), location));
        self.request.close = self.request.header.check_close(&self.request.version, false);
        match self.request.version() {
            Version::HTTP_10 | Version::HTTP_11 => match self.request.header.get_host() {
                Some(src) => self.request.set_host(src.to_string()),
                None => return Err(self.interrupt(
                    Response::expectation_failed(),
                    Errs::str("host not found in header!")))
            }
            // Version::HTTP_2 => match self.header.get_host() {
            //     Some(src) => self.host = src.to_string(),
            //     None => self.host = self.url.authority().addr().to_string()
            // }
            _ => return Err(self.interrupt(
                Response::http_version_not_supported(),
                Errs::str("parse request failed, version not support!")))
        }
        match self.request.header.get_content_type() {
            Some(src) => self.request.set_content_type(Some(ContentType::from_str(&src))),
            None => {}
        }
        match self.request.header.get_accept_encoding() {
            Some(src) => self.request.set_accept_encoding(src),
            None => {}
        }
        match self.request.header.read_cookies() {
            Ok(src) => self.request.set_cookies(src),
            Err(err) => return Err(self.interrupt(
                Response::bad_request(),
                Errs::strs("read cookies from header failed!", err)))
        }
        Ok(())
    }

    fn fill_body(&mut self, iter: &mut Iter<u8>, mut size: usize) -> StarryResult<()> {
        match self.method() {
            Method::PATCH | Method::PUT | Method::POST => {
                let mut bytes = vec![];
                loop {
                    match iter.next() {
                        Some(b) => {
                            match b {
                                b'\r' | b'\n' => size -= 1,
                                _ => {
                                    bytes.push(*b);
                                    size -= 1;
                                    break;
                                }
                            }
                        }
                        None => return Ok(())
                    }
                }
                // iter.next(); // 请求，过滤换行符
                // size -= 1;

                match self.request.header().get_content_length() {
                    Some(content_len) => match content_len.parse::<isize>() {
                        Ok(mut len) => {
                            if len > 0 { // 读取到"Content-Length"大于0
                                // 因为前面内容多读取了一个字节，所以len = len - 1
                                len = len - 1;
                                // Body数据存在，新建写流，将body数据进行填充
                                let mut bw = BytesMut::new().writer();
                                self.write_bytes(bw.borrow_mut(), bytes.as_slice())?;
                                self.write_bytes(bw.borrow_mut(), &iter.as_ref()[0..size])?;
                                let mut buffer = [0; 1024];
                                let mut count = size;
                                loop { // 将stream中数据全数读出到 self.body
                                    if count as isize == len || size == 0 {
                                        break;
                                    }
                                    size = self.reread_stream(&mut buffer)?;
                                    self.write_bytes(bw.borrow_mut(), &buffer[0..size])?;
                                    count += size
                                }
                                self.request.set_body(bw.into_inner());
                            } else if len == 0 {  // 读取到"Content-Length"等于0
                                return Ok(());
                            } else if len == -1 {  // 读取到"Content-Length"小于0
                                // Body数据存在，长度未知，新建写流，将body数据进行填充
                                let mut bw = BytesMut::new().writer();
                                self.write_bytes(bw.borrow_mut(), bytes.as_slice())?;
                                self.write_bytes(bw.borrow_mut(), &iter.as_ref()[0..size])?;
                                let mut buf_all = Vec::new();
                                match self.stream.read_to_end(&mut buf_all) {
                                    Ok(_) => {
                                        // bw.write(buf_all.as_slice());
                                        self.write_bytes(bw.borrow_mut(), buf_all.as_slice())?;
                                        self.request.set_body(bw.into_inner());
                                    }
                                    Err(err) => return Err(self.interrupt(
                                        Response::length_required(),
                                        Errs::strs("parse request body while read_to_end failed!", err)))
                                }
                            } else {
                                return Err(self.interrupt(
                                    Response::length_required(),
                                    Errs::string(format!("content len {} invalid from header failed!", len))));
                            }
                            self.request.set_content_length(len);
                            Ok(())
                        }
                        Err(err) => return Err(self.interrupt(
                            Response::length_required(),
                            Errs::strings(format!("content len {} parse usize from header failed!", content_len), err)))
                    },
                    None => {
                        self.request.set_content_length(0);
                        return Ok(());
                    }
                }
            }
            _ => Ok(())
        }
    }

    /// 如果存在body，则需要根据content-type对body数据进行解析使用
    fn parse_body(&mut self) -> StarryResult<()> {
        self.request.body_parse = true;
        let body = self.body().to_vec();
        log::trace!("body len = {}", body.len());
        let content_type;
        match self.request.content_type() {
            Some(src) => content_type = src,
            None => return Ok(())
        }
        match content_type.inner() {
            Inner::ApplicationXWWWFormUrlEncoded => { // 11=22&44=55 / 11=22&44=55&77=&=222
                let mut step: u8 = 1; // 1:key -> 2:value
                let mut data: Vec<u8> = vec![];
                let mut key: String = String::from("");
                for b in body {
                    match b {
                        b'=' => {
                            if step == 1 {
                                key = String::from_utf8_lossy(&data).to_string();
                                step = 2;
                                data.clear()
                            } else {
                                data.push(b)
                            }
                        }
                        b'&' => {
                            if step == 1 {
                                data.push(b)
                            } else {
                                let value = String::from_utf8_lossy(&data).to_string();
                                // self.form.set(key.clone(), value.clone());
                                self.request.form_set(key.clone(), value);
                                step = 1;
                                data.clear()
                            }
                        }
                        _ => data.push(b)
                    }
                }
                let value = String::from_utf8_lossy(&data).to_string();
                if !key.is_empty() || !value.is_empty() {
                    // self.form.set(key.clone(), value.clone());
                    self.request.form_set(key.clone(), value);
                }
            }
            Inner::MultipartFormData(src) => {
                let start = format!("--{}", src);
                let end = format!("--{}--", src);
                // println!("start = {}", start);
                // println!("end   = {}", end);
                // println!();
                let mut name = String::new();
                let mut filename = String::new();
                let mut tmp = vec![];
                let mut content = vec![];
                // form-data中每个流文件都有单独的类型
                let mut content_type = ContentType::default();
                // 1:start -> 2:Content-Disposition: form-data; name="1" -> 3:"" -> 4:value
                // 1:start -> 2:Content-Disposition: form-data; name="1"; filename="file.txt" -> 3:"" -> 5:file
                let mut step: u8 = 1;
                for b in body {
                    match b {
                        b'\n' => { // 发现换行符启动验证
                            let src = String::from_utf8_lossy(tmp.as_slice()).to_string();
                            match step {
                                1 => if src.eq(start.as_str()) {
                                    step = 2;
                                } else if src.eq(end.as_str()) {
                                    break;
                                } else {
                                    return Err(self.interrupt(
                                        Response::bad_request(),
                                        Errs::str("boundary can not matched!")));
                                },
                                2 => {
                                    let mut vs = src.split(";");
                                    match vs.next() {
                                        Some(res) => if res.ne("Content-Disposition: form-data") {
                                            return Err(self.interrupt(
                                                Response::bad_request(),
                                                Errs::str("Content-Disposition: form-data can not matched!")));
                                        }
                                        None => {
                                            return Err(self.interrupt(
                                                Response::bad_request(),
                                                Errs::str("multipart/form-data body disposition parse failed!")));
                                        }
                                    }
                                    match vs.next() { // name
                                        Some(src) => {
                                            let res = src.trim();
                                            let len = res.len() - 1;
                                            name = res[6..len].to_string();
                                        }
                                        None => return Err(self.interrupt(
                                            Response::bad_request(),
                                            Errs::str("multipart/form-data body name parse failed!")))
                                    }
                                    match vs.next() { // filename
                                        Some(src) => {
                                            let res = src.trim();
                                            let len = res.len() - 1;
                                            filename = res[10..len].to_string();
                                            step = 5
                                        }
                                        None => step = 3
                                    }
                                }
                                3 => step = 4,
                                4 => if src.eq(start.as_str()) || src.eq(end.as_str()) {
                                    content = content[..content.len() - 1].to_owned();
                                    let value = String::from_utf8_lossy(content.as_slice()).to_string();
                                    // self.form.set(name.clone(), value.clone());
                                    self.request.form_set(name.clone(), value);
                                    content.clear();
                                    step = 2;
                                } else {
                                    tmp.push(b);
                                    content.append(&mut tmp)
                                }
                                5 => {
                                    content_type = ContentType::from_str(&src[14..]);
                                    step = 6
                                    // if src.eq("Content-Type: application/octet-stream") {
                                    //     step = 6
                                    // } else {
                                    //     self.response(Status::BAD_REQUEST);
                                    //     return Err(Errs::str("multipart/form-data body file type parse failed!"));
                                    // }
                                }
                                6 => {
                                    step = 7
                                }
                                7 => if src.eq(start.as_str()) || src.eq(end.as_str()) {
                                    content = content[..content.len() - 1].to_owned();
                                    self.request.multipart_form_insert(name.clone(),
                                                                       filename.clone(),
                                                                       content.clone(),
                                                                       content_type.clone());
                                    content.clear();
                                    step = 2
                                } else {
                                    tmp.push(b);
                                    content.append(&mut tmp)
                                }
                                _ => {}
                            }
                            tmp.clear()
                        }
                        b'\r' => {} // 回车符什么也不做
                        _ => tmp.push(b)
                    }
                }
            }
            _ => {}
        }
        // for (filename, file_header) in self.multipart_form.file_map() {
        //     println!("file name = {}", filename);
        //     println!("file content = \n{}", String::from_utf8_lossy(file_header.content().as_slice()));
        // }
        Ok(())
    }

    fn write_bytes(&mut self, bw: &mut Writer<BytesMut>, src: &[u8]) -> StarryResult<usize> {
        match bw.write(src) {
            Ok(src) => Ok(src),
            Err(err) => return Err(self.interrupt(
                Response::bad_request(),
                Errs::strs("write data into body failed!", err)))
        }
    }

    fn reread_stream(&mut self, buf: &mut [u8]) -> StarryResult<usize> {
        match self.stream.read(buf) {
            Ok(src) => {
                Ok(src)
            }
            Err(err) => return Err(self.interrupt(
                Response::bad_request(),
                Errs::strs("reread failed while parse request with failed!", err)))
        }
    }
}

impl<Stream: Read + Write + Debug> Requester<Stream> {
    /// 执行回复操作
    pub(crate) fn response(&mut self, mut response: Response) -> StarryResult<()> {
        log::debug!("response: {:#?}", response);

        // let mut tmp = vec![];
        // let _ = self.stream.read_to_end(&mut tmp).unwrap_or(0);

        // 状态行
        self.write(response.version.as_slice())?;
        self.write(b" ")?;
        self.write(response.status.code().to_string().as_bytes())?;
        self.write(b" ")?;
        self.write(response.status.phrase_as_slice())?;
        self.write(b"\r\n")?;

        // 头部块
        for (key, values) in response.header.map() {
            for value in values {
                self.write(key.as_bytes())?;
                self.write(b": ")?;
                self.write(value.as_bytes())?;
                self.write(b"\r\n")?;
            }
        }
        self.write(b"\r\n")?;
        // 数据块
        self.write(response.get_write_content().as_ref())?;

        match self.stream.flush() {
            Ok(()) => Ok(()),
            Err(err) => Err(Errs::strs("stream flush failed!", err)),
        }
    }

    fn write(&mut self, buf: &[u8]) -> StarryResult<()> {
        match self.stream.write_all(buf) {
            Ok(_) => Ok(()),
            Err(err) => Err(Errs::strs("stream write to response failed!", err)),
        }
    }

    /// 中断操作，返回报错和对应应答
    fn interrupt(&mut self, response: Response, error: Error) -> Error {
        match self.response(response) {
            Ok(()) => error,
            Err(err) => {
                log::error!("http response interrupt failed! error is {}", err);
                error
            }
        }
    }
}

#[cfg(test)]
mod requester_test {
    use std::borrow::BorrowMut;
    use std::fs::File;
    use std::io::Read;

    use crate::http::url::authority::Addr;
    use crate::Requester;

    impl Requester<File> {
        fn new_mock(file: File) -> Self {
            Requester {
                request: Default::default(),
                stream: file,
            }
        }
    }

    #[test]
    fn parse_test() {
        let mut file = File::open("examples/request_test").unwrap();
        let mut buffer = [0; 1024];
        let mut iter;
        // 剩余待读取数据的总长度
        let mut size = file.read(&mut buffer).unwrap();
        iter = buffer.iter();
        let mut req = Requester::new_mock(file.try_clone().unwrap());
        // 解析请求行信息 POST /path/data?key=value&key2=value2 HTTP/1.1
        let (location, count) = req.parse_request_line(iter.borrow_mut()).unwrap();
        size -= count;
        assert_eq!(location.path(), "/", "location path is {}", location.path());
        assert_eq!(location.query().get("key").unwrap(), "value");
        assert_eq!(location.query().get("key2").unwrap(), "value2");
        let count = req.parse_request_header(iter.borrow_mut());
        size -= count;
        assert_eq!(req.request.header.get("Authorization").unwrap(), "Basic dXNlcjpwYXNzd29yZA==");
        assert_eq!(req.request.header.get("Accept").unwrap(), "*/*");
        assert_eq!(req.request.header.get("Content-Type").unwrap(), "multipart/form-data; boundary=--------------------------928695049288495294588005");
        assert_eq!(req.request.header.get("Accept-Encoding").unwrap(), "gzip, deflate, br");
        assert_eq!(req.request.header.get("Host").unwrap(), "localhost:7878");
        let peer = Addr::new("127.0.0.1".to_string());
        let local = Addr::new("127.0.0.2".to_string());
        req.parse_others(location, peer, local).unwrap();
        assert_eq!("127.0.0.1:80", req.request.client.to_string());
        assert_eq!("http", req.request.url.scheme.as_str());
        assert_eq!("user", req.request.url.authority.userinfo().unwrap().username());
        assert_eq!("password", req.request.url.authority.userinfo().unwrap().password());
        assert_eq!(false, req.request.close);
        assert_eq!("localhost:7878", req.request.host);
        assert_eq!("multipart/form-data", req.content_type().unwrap().as_str());
        req.request.set_cookies(req.request.header.read_cookies().unwrap());
        assert_eq!(req.request.cookies.get(0).unwrap().name, "Cookie_3");
        assert_eq!(req.request.cookies.get(0).unwrap().value, "value3");
        assert_eq!(req.request.cookies.get(1).unwrap().name, "Cookie_4");
        assert_eq!(req.request.cookies.get(1).unwrap().value, "value4");
        req.fill_body(iter.borrow_mut(), size).unwrap();
        req.parse_body().unwrap();
        assert_eq!(req.param_value("key").unwrap(), "value");
        assert_eq!(req.param_value("key2").unwrap(), "value2");
        assert_eq!(req.form_value("1").unwrap().unwrap(), "2\n3");
        assert_eq!(req.form_value("7").unwrap().unwrap(), "5");
        assert_eq!(req.form_file_value("4").unwrap().unwrap().filename(), "test2.txt");
        assert_eq!(req.form_file_value("4").unwrap().unwrap().content(), "test1
test2

test3
test4

test5


".as_bytes().to_vec());
    }

    // fn parse_bench_init(peer: Addr, local: Addr) {
    //     let mut file = File::open("examples/request_test").unwrap();
    //     let mut buffer = [0; 1024];
    //     let mut iter;
    //     // 剩余待读取数据的总长度
    //     let mut size = file.read(&mut buffer).unwrap();
    //     iter = buffer.iter();
    //     let mut req = Request::new(file.try_clone().unwrap());
    //     // 解析请求行信息 POST /path/data?key=value&key2=value2 HTTP/1.1
    //     let (location, count) = req.parse_request_line(iter.borrow_mut()).unwrap();
    //     size -= count;
    //     let count = req.parse_request_header(iter.borrow_mut());
    //     size -= count;
    //     req.parse_others(location, peer, local).unwrap();
    //     req.cookies = req.header.read_cookies().unwrap();
    //     req.fill_body(iter.borrow_mut(), size).unwrap();
    //     req.parse_body().unwrap();
    // }
    //
    // #[bench]
    // fn parse_bench(b: &mut Bencher) {
    //     let peer = Addr::new("127.0.0.1".to_string());
    //     let local = Addr::new("127.0.0.2".to_string());
    //     b.iter(|| parse_bench_init(peer, local))
    // }
}

