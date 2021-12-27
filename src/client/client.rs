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

use std::net::{TcpStream, ToSocketAddrs};

use crate::{Request, Response};
use crate::utils::errors::{Errs, StarryResult};
use std::io::Write;

pub struct HttpClient {
    stream: TcpStream,
    request: Request,
}

impl HttpClient {
    pub fn new(url: String) -> StarryResult<HttpClient> {
        match url.to_socket_addrs() {
            Ok(src) => match TcpStream::connect(src.as_ref()[0]) {
                Ok(stream) => Ok(HttpClient { stream, request: Default::default() }),
                Err(err) => Err(Errs::strings(format!("connect to {} failed!", url), err))
            }
            Err(err) => Err(Errs::strings(format!("{} trans to socket failed!", url), err))
        }
    }

    pub fn create(request: Request) -> StarryResult<HttpClient> {
        let addr = request.socket_addr()?;
        match TcpStream::connect(addr) {
            Ok(stream) => Ok(HttpClient { stream, request }),
            Err(err) => Err(Errs::strings(format!("connect to {} failed!", addr.to_string()), err))
        }
    }

    pub fn get(url: String) -> StarryResult<Response> {
        Err(Errs::str(""))
    }

    pub fn exec(request: Request) -> StarryResult<Response> {
        let mut client = HttpClient::create(request)?;
        client.do_request()
    }
}

impl HttpClient {
    /// 执行请求操作
    fn do_request(&mut self) -> StarryResult<Response> {
        log::debug!("request: {:#?}", self.request);
        Err(Errs::str(""))

        // todo 顺序处理
        // // 状态行
        // self.write(self.request.method.as_str().as_bytes())?;
        // self.write(b" ")?;
        // self.write(self.request.url.location.to_string().as_bytes())?;
        // self.write(b" ")?;
        // self.write(self.request.version.as_slice())?;
        // self.write(b"\r\n")?;
        //
        // // 头部块
        // for (key, values) in self.request.header.map() {
        //     for value in values {
        //         self.write(key.as_bytes())?;
        //         self.write(b": ")?;
        //         self.write(value.as_bytes())?;
        //         self.write(b"\r\n")?;
        //     }
        // }
        // self.write(b"\r\n")?;
        // // 数据块
        // self.write(self.request.get_write_content().as_ref())?;
        //
        // match self.stream.flush() {
        //     Ok(()) => Ok(()),
        //     Err(err) => Err(Errs::strs("stream flush failed!", err)),
        // }
    }

    fn write(&mut self, buf: &[u8]) -> StarryResult<usize> {
        match self.stream.write(buf) {
            Ok(src) => Ok(src),
            Err(err) => Err(Errs::strs("stream write to request failed!", err)),
        }
    }
}

