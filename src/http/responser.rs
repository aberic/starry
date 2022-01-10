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

use std::borrow::BorrowMut;
use std::fmt::Debug;
use std::io::{Read, Write};
use std::slice::Iter;

use bytes::{BufMut, BytesMut};
use bytes::buf::Writer;

use crate::{Request, Response, Status, Version};
use crate::http::url::Scheme;
use crate::utils::errors::{Errs, StarryResult};
use crate::utils::Strings;

pub(crate) const CLIENT_TCP_STREAM_HAD_NO_DATA: &str = "client response tcp stream had no data!";

/// [`Response`]的封装
///
/// [`Response`]: crate::Response
#[derive(Debug)]
pub struct Responser<Stream: Read + Write + Debug> {
    pub(crate) response: Response,
    pub(crate) stream: Stream,
}

impl<Stream: Read + Write + Debug> Responser<Stream> {
    pub fn body(&mut self) -> Vec<u8> {
        self.response.body()
    }
}

impl<Stream: Read + Write + Debug> Responser<Stream> {
    /// 通过写入流获取来自客户端的Request
    ///
    /// * root 资源树根结点
    /// * peer 客户端地址信息
    /// * local 本机地址信息
    pub(crate) fn from(stream: Stream, request: Request) -> StarryResult<Self> {
        let mut resper = Responser {
            response: Default::default(),
            stream,
        };
        resper.request(request)?;
        resper.parse()?;
        Ok(resper)
    }

    /// 解析返回信息
    fn parse(&mut self) -> StarryResult<()> {
        let mut data = vec![];
        let mut iter;
        // 当前读取总长度
        let mut count = 0;
        // 剩余待读取数据的总长度
        let mut size;
        match self.stream.read_to_end(&mut data) {
            Ok(src) => {
                log::trace!("response stream read size = {}", src);
                if src == 0 { // 没有数据进入
                    return Err(Errs::str(CLIENT_TCP_STREAM_HAD_NO_DATA));
                }
                size = src;
                iter = data.iter()
            }
            Err(err) => return Err(Errs::strs("read failed while parse request with error!", err))
        }

        // 解析返回行信息 HTTP/1.1 200 OK
        let len = self.parse_response_line(iter.borrow_mut())?;
        log::trace!("parse_response_line size = {}, len1 = {}", size, len);
        count += len;

        // 解析消息报头
        let len = self.parse_response_header(iter.borrow_mut());
        log::trace!("size = {}, len2 = {}", size, len);
        count += len;

        self.parse_others()?;

        if size < count {
            return Ok(());
        }
        size -= count;

        // 读取请求正文
        // 当用户使用到form数据等情况时，会解析，解析后，body内数据会被清空
        // 用户也可以主动使用body数据，但用户使用后，解析不会再自动进行
        self.fill_body(iter.borrow_mut(), size)?;

        // println!("response buffer = {}", String::from_utf8_lossy(buffer.as_slice()));
        Ok(())
    }

    /// 解析返回行信息 HTTP/1.1 200 OK
    ///
    /// usize 读取数据的总长度
    fn parse_response_line(&mut self, iter: &mut Iter<u8>) -> StarryResult<usize> {
        // 读取数据的总长度
        let mut count = 0;
        let mut step: u8 = 1; // version -> status.code -> status.msg
        let mut data: Vec<u8> = vec![];
        let mut status_op: Option<Status> = None;
        loop {
            match iter.next() {
                Some(b) => {
                    count += 1;
                    match b {
                        b' ' => match step { // HTTP/1.1...
                            1 => {
                                match Version::from_bytes(data.as_slice()) { // HTTP/1.1...
                                    Ok(src) => {
                                        self.response.version(src);
                                        data.clear();
                                        step = 2
                                    }
                                    Err(err) => return Err(Errs::strs("parse response version failed!", err))
                                }
                            }
                            2 => match Strings::from_utf8(data.clone())?.parse::<u16>() { // 200
                                Ok(src) => {
                                    status_op = Some(Status::from_code(src)?);
                                    data.clear();
                                    step = 3
                                }
                                Err(err) => return Err(Errs::strs("parse response status code failed!", err))
                            }
                            _ => return Err(Errs::str("parse response failed!"))
                        }
                        b'\r' | b'\n' => match step {
                            3 => match status_op.clone() { // OK
                                Some(status) => {
                                    let res = Strings::from_utf8(data.clone())?;
                                    if status.eq(&res) {
                                        self.response.status = status;
                                        break;
                                    } else {
                                        return Err(Errs::str("parse response failed while status code unmatch msg!"));
                                    }
                                }
                                None => return Err(Errs::str("parse response failed while status code is none!"))
                            }
                            _ => return Err(Errs::str("parse response failed while step 3!"))
                        }
                        _ => data.push(*b),
                    }
                }
                None => break
            }
        }
        Ok(count)
    }

    /// 解析消息报头
    ///
    /// usize 读取数据的总长度
    fn parse_response_header(&mut self, iter: &mut Iter<u8>) -> usize {
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
                            self.response.set_header(key.clone(), String::from_utf8_lossy(&data).to_string());
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
    pub(crate) fn parse_others(&mut self) -> StarryResult<()> {
        self.response.close = self.response.header.check_close(&self.response.version, false);
        match self.response.header.read_cookies() {
            Ok(src) => {
                self.response.cookies = src;
                Ok(())
            }
            Err(err) => Err(Errs::strs("read cookies from header failed!", err))
        }
    }

    fn fill_body(&mut self, iter: &mut Iter<u8>, mut size: usize) -> StarryResult<()> {
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

        match self.response.header.get_content_length() {
            Some(content_len) => match content_len.parse::<isize>() {
                Ok(mut len) => {
                    if len > 0 { // 读取到"Content-Length"大于0
                        // 因为前面内容多读取了一个字节，所以len = len - 1
                        len = len - 1;
                        // Body数据存在，新建写流，将body数据进行填充
                        let mut bw = BytesMut::new().writer();
                        self.write_bytes(bw.borrow_mut(), bytes.as_slice())?;
                        self.write_bytes(bw.borrow_mut(), &iter.as_ref()[0..size])?;
                        self.response.set_body(bw.into_inner());
                    } else if len == 0 {  // 读取到"Content-Length"等于0
                        return Ok(());
                    } else if len == -1 {  // 读取到"Content-Length"小于0
                        // Body数据存在，长度未知，新建写流，将body数据进行填充
                        let mut bw = BytesMut::new().writer();
                        self.write_bytes(bw.borrow_mut(), bytes.as_slice())?;
                        self.write_bytes(bw.borrow_mut(), &iter.as_ref()[0..size])?;
                        self.response.set_body(bw.into_inner());
                    } else {
                        return Err(Errs::string(format!("content len {} invalid from header failed!", len)));
                    }
                    Ok(())
                }
                Err(err) => return Err(Errs::strings(format!("content len {} parse usize from header failed!", content_len), err))
            },
            None => return Ok(())
        }
    }

    fn write_bytes(&mut self, bw: &mut Writer<BytesMut>, src: &[u8]) -> StarryResult<usize> {
        match bw.write(src) {
            Ok(src) => Ok(src),
            Err(err) => return Err(Errs::strs("write data into body failed!", err))
        }
    }

    fn reread_stream(&mut self, buf: &mut [u8]) -> StarryResult<usize> {
        match self.stream.read(buf) {
            Ok(src) => {
                Ok(src)
            }
            Err(err) => return Err(Errs::strs("reread failed while parse response with failed!", err))
        }
    }
}

impl<Stream: Read + Write + Debug> Responser<Stream> {
    /// 执行请求操作
    pub(crate) fn request(&mut self, mut request: Request) -> StarryResult<()> {
        // log::debug!("request: {:#?}", request);

        // 状态行
        self.write(request.method.as_str().as_bytes())?;
        self.write(b" ")?;
        self.write(request.url.location.to_string().as_bytes())?;
        self.write(b" ")?;
        self.write(request.version.as_slice())?;
        self.write(b"\r\n")?;

        // 头部块
        for (key, values) in request.header.map() {
            for value in values {
                self.write(key.as_bytes())?;
                self.write(b": ")?;
                self.write(value.as_bytes())?;
                self.write(b"\r\n")?;
            }
        }
        self.write(b"\r\n")?;
        // 数据块
        self.write(request.get_write_content().as_ref())?;

        match self.stream.flush() {
            Ok(()) => Ok(()),
            Err(err) => Err(Errs::strs("stream flush failed!", err)),
        }
    }

    fn write(&mut self, buf: &[u8]) -> StarryResult<usize> {
        match self.stream.write(buf) {
            Ok(src) => Ok(src),
            Err(err) => Err(Errs::strs("stream write to request failed!", err)),
        }
    }
}
