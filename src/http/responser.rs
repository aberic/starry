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

use std::io::{Read, Write};
use std::fmt::Debug;
use crate::{Response, Request};
use crate::utils::errors::{StarryResult, Errs};

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
        // todo stream解析
        let mut buffer = [0; 1024];
        let mut iter;
        // 剩余待读取数据的总长度
        let mut size;
        match self.stream.read(&mut buffer) {
            Ok(src) => {
                log::trace!("response stream read size = {}", src);
                if src == 0 { // 没有数据进入
                    return Err(Errs::str(CLIENT_TCP_STREAM_HAD_NO_DATA));
                }
                size = src;
                iter = buffer.iter()
            }
            Err(err) => return Err(Errs::strs("read failed while parse request with error!", err))
        }
        println!("response buffer = {}", String::from_utf8_lossy(buffer.as_slice()));
        Ok(())
    }
}

impl<Stream: Read + Write + Debug> Responser<Stream> {
    /// 执行请求操作
    pub(crate) fn request(&mut self, mut request: Request) -> StarryResult<()> {
        log::debug!("request: {:#?}", request);

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
