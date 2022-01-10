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

use std::collections::HashMap;
use std::net::{Shutdown, TcpStream};
use std::sync::{Arc, RwLock};
use std::sync::atomic::{AtomicBool, Ordering};

use crate::{Method, Request, Response};
use crate::http::responser::Responser;
use crate::utils::{Channel, Time};
use crate::utils::concurrent::Thread;
use crate::utils::errors::{Errs, StarryResult};

lazy_static! {
        static ref CLIENT_STREAM_MAP: Arc<RwLock<HashMap<String, Arc<RwLock<Channel<TcpStreamer>>>>>> =
            Arc::new(RwLock::new(HashMap::new()));
    }

pub struct HttpClient {
    /// 是否启用http压缩，如gzip、deflate等
    compress: bool,
    request: Request,
}

impl HttpClient {
    pub fn new(request: Request) -> Self {
        HttpClient { compress: false, request }
    }

    /// 创建自定义客户端
    ///
    /// * compress 是否启用http压缩，如gzip、deflate等
    pub fn create(compress: bool, request: Request) -> Self {
        HttpClient { compress, request }
    }

    pub fn get(url: &str) -> StarryResult<Response> {
        let request = default_request(Method::GET, url)?;
        exec(request)
    }
}

fn default_request(method: Method, url: &str) -> StarryResult<Request> {
    let mut request = Request::new(method, url)?;
    request.header.set_str("Host", request.host.as_str());
    match request.url.authority.userinfo() {
        Some(src) => request.header.set_str("Authorization", format!("Basic {}", src.base64()).as_str()),
        None => {}
    }
    if !request.close {
        request.header.set_connection();
    }
    Ok(request)
}

/// 新建
fn stream(request: Request) -> StarryResult<TcpStream> {
    let addr = request.socket_addr_ipv4()?;
    log::trace!("request stream addr = {}", addr.to_string());
    match TcpStream::connect(addr) {
        Ok(stream) => Ok(stream),
        Err(err) => Err(Errs::strings(format!("client connect to {} failed!", addr.to_string()), err))
    }
}

fn exec(request: Request) -> StarryResult<Response> {
    let mut client = HttpClient::new(request);
    client.send()
}

impl HttpClient {
    /// 执行请求操作
    pub fn send(&mut self) -> StarryResult<Response> {
        log::debug!("request: {:#?}", self.request);

        // 判断是否需要复用stream
        if self.request.close { // 如果不用复用
            Ok(Responser::from(stream(self.request.clone())?, self.request.clone())?.response)
        } else { // 如果复用
            let keepalive = 30000;
            // 通过复用方式直接获取可用stream
            let tcp_streamer = self.obtain(keepalive)?;
            match tcp_streamer.inner.try_clone() {
                Ok(stream) => {
                    let responser = Responser::from(stream, self.request.clone())?;
                    if responser.response.close {
                        tcp_streamer.channel.send(Check::Break)?;
                    } else {
                        CLIENT_STREAM_MAP.read().unwrap().get(&self.request.host).unwrap().write().unwrap().send(tcp_streamer).unwrap_or(());
                    }
                    Ok(responser.response)
                }
                Err(err) => Err(Errs::strs("stream clone in streamer send failed!", err))
            }
        }
    }

    /// 通过复用方式直接获取可用stream
    fn obtain(&self, keepalive: i64) -> StarryResult<TcpStreamer> {
        // 如果要复用，则先判断是否存在已有stream（建立复用池）
        let map_read = CLIENT_STREAM_MAP.read().unwrap();
        let channel_op = map_read.get(&self.request.host);
        // 则先判断是否存在已有stream池
        match channel_op {
            // 如果存在stream池，则继续取出
            Some(channel) => {
                loop {
                    match channel.read().unwrap().try_recv() {
                        // 如果池中存在可用stream，则取出
                        Ok(src) => {
                            if src.alive.load(Ordering::Acquire) {
                                src.channel.send(Check::Update)?;
                                return match src.try_clone() {
                                    Ok(src) => Ok(src),
                                    Err(err) => Err(Errs::strs("streamer clone while obtain failed!", err))
                                };
                            }
                        }
                        // 如果池中没有可用stream，则新建，再取出
                        Err(_) => {
                            let tcp_stream = stream(self.request.clone())?;
                            return TcpStreamer::from(tcp_stream, keepalive);
                        }
                    }
                }
            }
            // 如果不存在stream池，则创建新stream池，并新建可用stream
            None => {
                // 创建新stream池
                self.create_stream_channel();
                // 新建
                let tcp_stream = stream(self.request.clone())?;
                TcpStreamer::from(tcp_stream, keepalive)
            }
        }
    }

    /// 创建新stream池
    fn create_stream_channel(&self) {
        let mut map_write = CLIENT_STREAM_MAP.write().unwrap();
        match map_write.get(&self.request.host) {
            Some(_) => {}
            None => {
                let ch = Arc::new(RwLock::new(Channel::bounded(10)));
                map_write.insert(self.request.host.clone(), ch.clone());
            }
        }
    }
}

struct TcpStreamer {
    inner: TcpStream,
    alive: Arc<AtomicBool>,
    keepalive: i64,
    channel: Arc<Channel<Check>>,
}

impl TcpStreamer {
    pub(crate) fn from(tcp_stream: TcpStream, keepalive: i64) -> StarryResult<Self> {
        let peer_addr;
        match tcp_stream.peer_addr() {
            Ok(src) => peer_addr = src.to_string(),
            Err(err) => return Err(Errs::strs("tcp stream get peer addr in streamer failed!", err))
        }
        let s = Self {
            inner: tcp_stream,
            alive: Arc::new(AtomicBool::new(true)),
            keepalive,
            channel: Arc::new(Channel::bounded(1)),
        };
        let streamer = s.try_clone()?;
        Thread::spawn(move || check_keepalive(streamer, keepalive, peer_addr))?;
        Ok(s)
    }

    fn try_clone(&self) -> StarryResult<Self> {
        match self.inner.try_clone() {
            Ok(src) => Ok(Self { inner: src, alive: self.alive.clone(), keepalive: self.keepalive, channel: self.channel.clone() }),
            Err(err) => Err(Errs::err(err))
        }
    }
}

/// 检查当前stream是否超时
fn check_keepalive(streamer: TcpStreamer, keepalive: i64, peer_addr: String) {
    let mut time = Time::now();
    time.add_milliseconds(keepalive);
    let mut expect_time = time.num_milliseconds();
    loop {
        let mut time_now = Time::now();
        if expect_time <= time_now.num_milliseconds() {
            log::trace!("expect_time = {}", Time::format_data(Time::from_milliseconds(expect_time), "%Y-%m-%d %H:%M:%S"));
            streamer.alive.store(false, Ordering::Relaxed);
            break;
        } else {
            match streamer.channel.try_recv() {
                Ok(src) => match src {
                    Check::Update => {
                        log::trace!("server channel receive update!");
                        time_now.add_milliseconds(keepalive);
                        expect_time = time_now.num_milliseconds();
                    }
                    Check::Break => break
                }
                Err(_) => {}
            }
        }
    }
    log::debug!("check keepalive stream {} shutdown!", peer_addr);
    match streamer.inner.shutdown(Shutdown::Both) {
        Ok(_) => log::trace!("tcp stream {} shutdown success!", peer_addr),
        Err(err) => log::error!("tcp stream {} shutdown failed! {}", peer_addr, err.to_string())
    }
}

/// 超时检查
pub(crate) enum Check {
    /// 更新超时起始参考值
    Update,
    /// 退出检查
    Break,
}

