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

use std::fmt::{Debug, Formatter};
use std::ops::Add;
use std::sync::{Arc, RwLock};

use crate::{Context, Method};
use crate::server::Extend;
use crate::server::node::Root;

/// 待实现接收请求方法
///
/// ctx 请求处理上下文结构
pub(crate) type Handler = fn(context: Box<Context>);

pub struct Router {
    /// 临时存储group值
    pattern: String,
    /// 当前路由组的全组过滤器
    extend: Option<Extend>,
    root: Arc<RwLock<Root>>,
}

impl Router {
    pub(crate) fn new(pattern: String, root: Arc<RwLock<Root>>) -> Self {
        Router { pattern, extend: None, root }
    }

    pub(crate) fn new_wf(pattern: String, extend: Extend, root: Arc<RwLock<Root>>) -> Self {
        Router { pattern, extend: Some(extend), root }
    }

    /// 新增服务资源，带过滤器
    ///
    /// 过滤操作尽量不要对数据体里的信息进行校验之类的流程，最好是对path、header和cookie进行过滤
    ///
    /// 服务资源尽量不要有重复前缀，如果存咋重复，则当匹配类型与固定类型重合时，会优先选择最先定义的资源，如下：
    /// ```res
    /// /a/b/:c/d/:e/:f/g
    /// /a/b/x/d/y/z/g
    /// ```
    /// 当接收到`/a/b/x/d/y/z/g`请求时，并不会真正触发第二行资源反馈，而是执行第一条资源
    ///
    /// 资源长度越长、重复率越高对性能影响越大，应尽可能进行简便简短的设计，使得匹配机制执行一次即可获得期望的结果
    ///
    /// * pattern 资源样式，如`/a/b/:c/d/:e/:f/g`
    /// * method 请求方法
    /// * handler 待实现接收请求方法
    /// * filters 过滤器/拦截器数组
    fn repo_wf(&self, pattern: &str, method: Method, handler: Handler, extend: Option<Extend>) {
        let extend_new;
        match extend.clone() {
            Some(src1) => {
                let mut filters = src1.filters.clone();
                match self.extend.clone() {
                    Some(mut src2) => {
                        filters.append(&mut src2.filters);
                        extend_new = Some(src1.copy_with(filters))
                    }
                    None => extend_new = Some(src1.copy())
                }
            }
            None => match self.extend.clone() {
                Some(src2) => extend_new = Some(Extend::e1(src2.filters)),
                None => extend_new = None
            }
        }

        let root_c = self.root.clone();
        let mut root_r = root_c.write().unwrap();
        root_r.add(self.pattern.clone().add(pattern), method, handler, extend_new)
    }

    /// 新增服务资源
    ///
    /// 服务资源尽量不要有重复前缀，如果存咋重复，则当匹配类型与固定类型重合时，会优先选择最先定义的资源，如下：
    /// ```res
    /// /a/b/:c/d/:e/:f/g
    /// /a/b/x/d/y/z/g
    /// ```
    /// 当接收到`/a/b/x/d/y/z/g`请求时，并不会真正触发第二行资源反馈，而是执行第一条资源
    ///
    /// 资源长度越长、重复率越高对性能影响越大，应尽可能进行简便简短的设计，使得匹配机制执行一次即可获得期望的结果
    ///
    /// * pattern 资源样式，如`/a/b/:c/d/:e/:f/g`
    /// * method 请求方法
    /// * handler 待实现接收请求方法
    fn repo(&self, pattern: &str, method: Method, handler: Handler) {
        self.repo_wf(pattern, method, handler, None)
    }

    pub fn option_wf(&self, pattern: &str, handler: Handler, extend: Extend) {
        self.repo_wf(pattern, Method::OPTIONS, handler, Some(extend))
    }

    pub fn get_wf(&self, pattern: &str, handler: Handler, extend: Extend) {
        self.repo_wf(pattern, Method::GET, handler, Some(extend))
    }

    pub fn post_wf(&self, pattern: &str, handler: Handler, extend: Extend) {
        self.repo_wf(pattern, Method::POST, handler, Some(extend))
    }

    pub fn put_wf(&self, pattern: &str, handler: Handler, extend: Extend) {
        self.repo_wf(pattern, Method::PUT, handler, Some(extend))
    }

    pub fn delete_wf(&self, pattern: &str, handler: Handler, extend: Extend) {
        self.repo_wf(pattern, Method::DELETE, handler, Some(extend))
    }

    pub fn head_wf(&self, pattern: &str, handler: Handler, extend: Extend) {
        self.repo_wf(pattern, Method::HEAD, handler, Some(extend))
    }

    pub fn trace_wf(&self, pattern: &str, handler: Handler, extend: Extend) {
        self.repo_wf(pattern, Method::TRACE, handler, Some(extend))
    }

    pub fn connect_wf(&self, pattern: &str, handler: Handler, extend: Extend) {
        self.repo_wf(pattern, Method::CONNECT, handler, Some(extend))
    }

    pub fn patch_wf(&self, pattern: &str, handler: Handler, extend: Extend) {
        self.repo_wf(pattern, Method::PATCH, handler, Some(extend))
    }

    pub fn link_wf(&self, pattern: &str, handler: Handler, extend: Extend) {
        self.repo_wf(pattern, Method::LINK, handler, Some(extend))
    }

    pub fn unlink_wf(&self, pattern: &str, handler: Handler, extend: Extend) {
        self.repo_wf(pattern, Method::UNLINK, handler, Some(extend))
    }

    pub fn pri_wf(&self, pattern: &str, handler: Handler, extend: Extend) {
        self.repo_wf(pattern, Method::PRI, handler, Some(extend))
    }

    pub fn option(&self, pattern: &str, handler: Handler) {
        self.repo(pattern, Method::OPTIONS, handler)
    }

    pub fn get(&self, pattern: &str, handler: Handler) {
        self.repo(pattern, Method::GET, handler)
    }

    pub fn post(&self, pattern: &str, handler: Handler) {
        self.repo(pattern, Method::POST, handler)
    }

    pub fn put(&self, pattern: &str, handler: Handler) {
        self.repo(pattern, Method::PUT, handler)
    }

    pub fn delete(&self, pattern: &str, handler: Handler) {
        self.repo(pattern, Method::DELETE, handler)
    }

    pub fn head(&self, pattern: &str, handler: Handler) {
        self.repo(pattern, Method::HEAD, handler)
    }

    pub fn trace(&self, pattern: &str, handler: Handler) {
        self.repo(pattern, Method::TRACE, handler)
    }

    pub fn connect(&self, pattern: &str, handler: Handler) {
        self.repo(pattern, Method::CONNECT, handler)
    }

    pub fn patch(&self, pattern: &str, handler: Handler) {
        self.repo(pattern, Method::PATCH, handler)
    }

    pub fn link(&self, pattern: &str, handler: Handler) {
        self.repo(pattern, Method::LINK, handler)
    }

    pub fn unlink(&self, pattern: &str, handler: Handler) {
        self.repo(pattern, Method::UNLINK, handler)
    }

    pub fn pri(&self, pattern: &str, handler: Handler) {
        self.repo(pattern, Method::PRI, handler)
    }
}

impl Debug for Router {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "pattern: {:#?}, \nroot: {:#?}", self.pattern, self.root)
    }
}

#[cfg(test)]
mod router_test {
    use std::collections::HashMap;
    use std::sync::{Arc, RwLock};

    use crate::{Context, Method};
    use crate::server::node::{Node, Root};
    use crate::server::Router;

    impl Router {
        fn fetch_mock(&self, pattern: String, method: Method) -> Option<(Node, HashMap<String, String>)> {
            self.root.read().unwrap().fetch(pattern, method)
        }
    }

    #[test]
    fn router_test() {
        let router = Router::new("/m/n".to_string(), Arc::new(RwLock::new(Root::new())));
        router.get("/test1/:a", h1);
        router.get("/test1/:a/c", h2);
        router.get("/test1/:a/c/d", h3);
        router.get("/a/c/d", h4);
        // println!("router = {:#?}", router)
        let (n1, _fields) = router.fetch_mock("/m/n/test1/:a".to_string(), Method::GET).unwrap();
        let (n2, _fields) = router.fetch_mock("/m/n/test1/:a/c".to_string(), Method::GET).unwrap();
        let (n3, _fields) = router.fetch_mock("/m/n/test1/:a/c/d".to_string(), Method::GET).unwrap();
        let (n4, _fields) = router.fetch_mock("/m/n/a/c/d".to_string(), Method::GET).unwrap();

        assert_eq!(n1.handler, router.root.read().unwrap().root_get.next_nodes[0].next_nodes[0].next_nodes[0].next_nodes[0].handler);
        assert_eq!(n2.handler, router.root.read().unwrap().root_get.next_nodes[0].next_nodes[0].next_nodes[0].next_nodes[0].next_nodes[0].handler);
        assert_eq!(n3.handler, router.root.read().unwrap().root_get.next_nodes[0].next_nodes[0].next_nodes[0].next_nodes[0].next_nodes[0].next_nodes[0].handler);
        assert_eq!(n4.handler, router.root.read().unwrap().root_get.next_nodes[0].next_nodes[0].next_nodes[1].next_nodes[0].next_nodes[0].handler);
    }

    fn h1(_context: Box<Context>) {}

    fn h2(_context: Box<Context>) {}

    fn h3(_context: Box<Context>) {}

    fn h4(_context: Box<Context>) {}
}
