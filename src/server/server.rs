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

use std::io::Write;
use std::net::{Shutdown, TcpListener, TcpStream, ToSocketAddrs};
use std::sync::{Arc, RwLock};

use log::LevelFilter;

use crate::{Context, Request};
use crate::Extend;
use crate::server::node::Root;
use crate::server::Router;
use crate::utils::{Channel, Time};
use crate::utils::concurrent::{Thread, ThreadPool};
use crate::utils::errors::StarryResult;
use crate::utils::log::LogModule;

#[derive(Debug, Clone)]
pub struct HttpServer {
    /// 设置线程池的大小
    ///
    /// 线程池的大小是生成的工作线程的数量。默认情况下，等于CPU核数
    pool_size: usize,
    /// 设置是否在接受的连接上启用`TCP keepalive`消息，单位ms。如果request启用，默认30000
    keepalive: i64,
    /// 日志策略
    module: Option<LogModule>,
    root: Arc<RwLock<Root>>,
}

impl HttpServer {
    pub fn new() -> Self {
        HttpServer { pool_size: 0, keepalive: 30000, module: None, root: Arc::new(RwLock::new(Root::new())) }
    }

    /// 创建路由组
    /// 路由组尽量不要做成动态参数，会容易对下级数据进行拦截
    ///
    /// * pattern 资源样式，如`/a/b`
    pub fn router(&self, pattern: &str) -> Router {
        Router::new(pattern.to_string(), self.root.clone())
    }

    /// 创建路由组
    ///
    /// 路由组尽量不要做成动态参数，会容易对下级数据进行拦截
    ///
    /// 过滤操作尽量不要对数据体里的信息进行校验之类的流程，最好是对path、header和cookie进行过滤
    ///
    /// * pattern 资源样式，如`/a/b`
    /// * filters 过滤器/拦截器数组
    pub fn router_wf(&self, pattern: &str, extend: Extend) -> Router {
        Router::new_wf(pattern.to_string(), extend, self.root.clone())
    }

    pub fn set_pool_size(&mut self, pool_size: usize) {
        self.pool_size = pool_size
    }

    pub fn set_keepalive(&mut self, keepalive: i64) {
        if keepalive < 0 {
            self.keepalive = 0;
        } else {
            self.keepalive = keepalive
        }
    }

    /// http服务日志设置
    ///
    /// * level输出日志级别，默认DEBUG
    /// * log_dir 日志文件目录，默认tmp
    /// * log_file_max_size 每个日志文件保存的最大尺寸 单位：M，默认1024
    /// * file_max_count 文件最多保存多少个，默认7
    pub fn set_log(&mut self,
                   level: LevelFilter,
                   dir: String,
                   file_max_size: u64,
                   file_max_count: u32) {
        self.module = Some(LogModule {
            name: String::from("starry-http"),
            pkg: "".to_string(),
            level, // LevelFilter::Debug
            additive: true,
            dir, // String::from("tmp"),
            file_max_size, // 1024,
            file_max_count, // 7,
        })
        // module.set_log(vec![]);
    }

    fn log_init(&self) {
        self.module.clone().unwrap_or(LogModule {
            name: String::from("starry-http"),
            pkg: "".to_string(),
            level: LevelFilter::Trace,
            additive: true,
            dir: String::from("tmp"),
            file_max_size: 1024,
            file_max_count: 7,
        }).config_log(vec![])
    }

    /// 创建一个新的HttpListener，它将被绑定到指定的端口。
    ///
    /// 返回的侦听器已准备好接受连接。
    /// 绑定端口号为0将请求操作系统为该侦听器分配一个端口。分配的端口可以通过[`context::local_addr`]方法查询。
    /// 地址类型可以是[`ToSocketAddrs`] trait的任何实现。具体示例请参阅其文档
    /// 如果addr产生多个地址，bind将对每个地址进行尝试，直到其中一个成功并返回监听器。
    /// 如果没有一个地址成功创建侦听器，则返回上次尝试返回的错误(最后一个地址)。
    ///
    /// [`context::local_addr`]: crate::Context::local_addr
    /// [`ToSocketAddrs`]: std::net::ToSocketAddrs
    pub fn listener<A: ToSocketAddrs>(&self, addr: A) -> StarryResult<()> {
        self.log_init();
        let mut thread_pool_builder = ThreadPool::builder();
        if self.pool_size > 0 {
            thread_pool_builder.pool_size(self.pool_size);
        }
        thread_pool_builder.name_prefix("starry-http-pool");
        let thread_pool = thread_pool_builder.create()?;
        let tcp_listener = TcpListener::bind(addr).unwrap();
        for tcp_stream_result in tcp_listener.incoming() {
            match tcp_stream_result {
                Ok(tcp_stream) => {
                    let keepalive = self.keepalive;
                    let root = self.root.clone();
                    match thread_pool.execute(move || handle_connection(tcp_stream, root, keepalive)) {
                        Ok(()) => {}
                        Err(err) => log::error!("thread pool execute tcp stream error, {}", err)
                    }
                }
                Err(err) => log::error!("tcp listener error, {}", err)
            }
        }
        Ok(())
    }
}

/// 针对本次stream进行处理
fn handle_connection(tcp_stream: TcpStream, root: Arc<RwLock<Root>>, mut keepalive: i64) {
    log::trace!("handle_connection");
    match tcp_stream.try_clone() {
        Ok(src) => {
            let (close, shutdown) = exec_stream(src, root.clone());
            if shutdown { // 如果连接关闭，直接返回
                return;
            }
            if close { // 如果不保持连接，当前keepalive置0
                keepalive = 0;
            }
            match tcp_stream.try_clone() {
                Ok(src) => {
                    // 双线异步循环执行超时检查和stream解析
                    loop_exec(src, root.clone(), keepalive)
                }
                Err(err) => log::error!("request tcp stream clone in handle connection 1 failed! {}", err.to_string())
            }
        }
        Err(err) => log::error!("request tcp stream clone in handle connection 2 failed! {}", err.to_string())
    }
}

/// 执行stream解析操作
///
/// * 是否关闭连接
/// * 是否关闭连接
fn exec_stream(tcp_stream: TcpStream, root: Arc<RwLock<Root>>) -> (bool, bool) {
    match Request::from(tcp_stream, root.clone()) {
        // request分预解析和解析两个过程，预解析用于判断请求有效性，如无效，则放弃后续解析操作
        Ok((request, node, fields)) => {
            let close = request.close;
            log::debug!("method = {}, path = {}, from = {}", request.method(), request.path(), request.client());
            let mut context = Box::new(Context::new(request, fields));
            log::trace!("context = {:#?}", context);
            match node.extend.clone() {
                Some(extend) => extend.exec(context.as_mut()), // 扩展执行，自我诊断
                None => {}
            }
            if !context.executed {
                node.handler()(context.as_mut())
            }
            (close, false)
        }
        Err(err) => {
            if err.to_string().eq("tcp stream had no data!") {
                (true, true)
            } else {
                log::info!("request from failed! {}", err.to_string());
                (true, false)
            }
        }
    }
}

/// 双线异步循环执行超时检查和stream解析
fn loop_exec(mut tcp_stream: TcpStream, root: Arc<RwLock<Root>>, keepalive: i64) {
    // 创建一个可以将stream接收信号同步更新至检查超时线程的通道
    let channel = Arc::new(Channel::unbounded());
    match tcp_stream.try_clone() {
        Ok(src) => {
            let channel = channel.clone();
            // 新启线程检查当前stream是否超时
            match Thread::spawn(move || check_keepalive(src, keepalive, channel)) {
                Ok(_) => {}
                Err(err) => {
                    log::error!("loop exec check stream failed! {}", err.to_string());
                    match tcp_stream.write("HTTP/1.1 500 Internal Server Error\r\n\r\n".as_bytes()) {
                        Ok(_) => {}
                        Err(err) => {
                            log::error!("loop exec check stream with write failed! {}", err.to_string());
                            return;
                        }
                    }
                }
            };
        }
        Err(err) => log::error!("request tcp stream clone in loop exec 1 failed! {}", err.to_string())
    }
    // 当前线程开启循环读取stream操作
    loop {
        match tcp_stream.try_clone() {
            Ok(src) => {
                let (_, shutdown) = exec_stream(src, root.clone());
                if shutdown { // 如果连接关闭，直接返回
                    channel.send(Check::Break);
                    return;
                }
                // 处理完当前stream后需要更新超时时间起始参考值
                let channel = channel.clone();
                channel.send(Check::Update);
            }
            Err(err) => log::error!("request tcp stream clone in loop exec 2 failed! {}", err.to_string())
        }
    }
}

/// 检查当前stream是否超时
fn check_keepalive(tcp_stream: TcpStream, keepalive: i64, channel: Arc<Channel<Check>>) {
    let mut time = Time::now();
    time.add_milliseconds(keepalive);
    let mut expect_time = time.num_milliseconds();
    loop {
        let mut time_now = Time::now();
        if expect_time <= time_now.num_milliseconds() {
            log::trace!("expect_time = {}", Time::format_data(Time::from_milliseconds(expect_time), "%Y-%m-%d %H:%M:%S"));
            break;
        } else {
            match channel.try_recv() {
                Ok(src) => match src {
                    Check::Update => {
                        log::trace!("channel receive update!");
                        time_now.add_milliseconds(keepalive);
                        expect_time = time_now.num_milliseconds();
                    }
                    Check::Break => break
                }
                Err(_) => {}
            }
        }
    }
    log::debug!("check stream {} shutdown!", tcp_stream.peer_addr().unwrap().to_string());
    match tcp_stream.shutdown(Shutdown::Both) {
        Ok(_) => log::trace!("tcp stream shutdown success!"),
        Err(err) => log::error!("tcp stream shutdown failed! {}", err.to_string())
    }
}

/// 超时检查
enum Check {
    /// 更新超时起始参考值
    Update,
    /// 退出检查
    Break,
}

#[cfg(test)]
mod server_test {
    use std::collections::HashMap;

    use crate::{Context, HttpServer, Method};
    use crate::server::node::Node;

    impl HttpServer {
        pub(crate) fn fetch(&self, pattern: String, method: Method) -> Option<(Node, HashMap<String, String>)> {
            self.root.read().unwrap().fetch(pattern, method)
        }
    }

    #[test]
    fn server_test_single() {
        let server = HttpServer::new();
        let router = server.router("/m/n");
        router.get("/test1/:a/c/d/:b", h1);
        router.get("/a/c/d/:b", h2);

        // println!("server = {:#?}", server);
        // println!("router = {:#?}", router1);

        let (n1, fields1) = server.fetch("/m/n/test1/x/c/d/y".to_string(), Method::GET).unwrap();
        let (n2, fields2) = server.fetch("/m/n/a/c/d/m".to_string(), Method::GET).unwrap();

        assert_eq!(fields1.get("a").unwrap(), "x");
        assert_eq!(fields1.get("b").unwrap(), "y");
        assert_eq!(fields2.get("b").unwrap(), "m");

        assert_eq!(n1, server.root.read().unwrap().root_get.next_nodes[0].next_nodes[0].next_nodes[0].next_nodes[0].next_nodes[0].next_nodes[0].next_nodes[0]);
        assert_eq!(n2, server.root.read().unwrap().root_get.next_nodes[0].next_nodes[0].next_nodes[1].next_nodes[0].next_nodes[0].next_nodes[0]);
    }

    #[test]
    fn server_test_multi() {
        let server = HttpServer::new();
        router1(server.clone());
        router2(server.clone());

        // println!("server = {:#?}", server);
        // println!("router = {:#?}", router1);

        let (n1, _fields) = server.fetch("/m/n/a/c".to_string(), Method::GET).unwrap();
        let (n2, _fields) = server.fetch("/x/y/test1/:a/c/d/:b".to_string(), Method::GET).unwrap();
        let (n3, _fields) = server.fetch("/x/y/a/c/d/:b".to_string(), Method::GET).unwrap();

        assert_eq!(n1, server.root.read().unwrap().root_get.next_nodes[0].next_nodes[0].next_nodes[1].next_nodes[0]);
        assert_eq!(n2, server.root.read().unwrap().root_get.next_nodes[1].next_nodes[0].next_nodes[0].next_nodes[0].next_nodes[0].next_nodes[0].next_nodes[0]);
        assert_eq!(n3, server.root.read().unwrap().root_get.next_nodes[1].next_nodes[0].next_nodes[1].next_nodes[0].next_nodes[0].next_nodes[0]);
    }

    fn router1(server: HttpServer) {
        let router1 = server.router("/m/n");
        router1.get("/test1/:a/c", h3);
        router1.get("/a/c", h4);
    }

    fn router2(server: HttpServer) {
        let router2 = server.router("/x/y");
        router2.get("/test1/:a/c/d/:b", h1);
        router2.get("/a/c/d/:b", h2);
    }

    fn h1(_context: &mut Context) {}

    fn h2(_context: &mut Context) {}

    fn h3(_context: &mut Context) {}

    fn h4(_context: &mut Context) {}
}


