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
use std::fmt::{Debug, Formatter};

use crate::Method;
use crate::server::Extend;
use crate::server::router::Handler;
use crate::utils::concurrent::Thread;

/// 资源树根结点
#[derive(Clone, Debug)]
pub(crate) struct Root {
    pub(crate) root_option: Node,
    pub(crate) root_get: Node,
    pub(crate) root_post: Node,
    pub(crate) root_put: Node,
    pub(crate) root_delete: Node,
    pub(crate) root_head: Node,
    pub(crate) root_trace: Node,
    pub(crate) root_connect: Node,
    pub(crate) root_patch: Node,
    pub(crate) root_link: Node,
    pub(crate) root_unlink: Node,
    pub(crate) root_pri: Node,
}

impl Root {
    pub(crate) fn new() -> Self {
        Root {
            root_option: Node::new(),
            root_get: Node::new(),
            root_post: Node::new(),
            root_put: Node::new(),
            root_delete: Node::new(),
            root_head: Node::new(),
            root_trace: Node::new(),
            root_connect: Node::new(),
            root_patch: Node::new(),
            root_link: Node::new(),
            root_unlink: Node::new(),
            root_pri: Node::new(),
        }
    }

    /// 新增节点
    ///
    /// * pattern 资源样式，如`/a/b/:c/d/:e/:f/g`
    /// * method 请求方法
    /// * handler 待实现接收请求方法
    /// * filters 过滤器/拦截器数组
    pub(crate) fn add(&mut self, pattern: String, method: Method, handler: Handler, extend: Option<Extend>) {
        match method {
            Method::OPTIONS => self.root_option.add(pattern, method, handler, extend),
            Method::GET => self.root_get.add(pattern, method, handler, extend),
            Method::POST => self.root_post.add(pattern, method, handler, extend),
            Method::PUT => self.root_put.add(pattern, method, handler, extend),
            Method::DELETE => self.root_delete.add(pattern, method, handler, extend),
            Method::HEAD => self.root_head.add(pattern, method, handler, extend),
            Method::TRACE => self.root_trace.add(pattern, method, handler, extend),
            Method::CONNECT => self.root_connect.add(pattern, method, handler, extend),
            Method::PATCH => self.root_patch.add(pattern, method, handler, extend),
            Method::LINK => self.root_link.add(pattern, method, handler, extend),
            Method::UNLINK => self.root_unlink.add(pattern, method, handler, extend),
            Method::PRI => self.root_pri.add(pattern, method, handler, extend)
        }
    }

    /// 获取节点
    ///
    /// * pattern 资源样式，如`/a/b/c/d/e/f/g`
    /// * method 请求方法
    pub(crate) fn fetch(&self, pattern: String, method: Method) -> Option<(Node, HashMap<String, String>)> {
        match method {
            Method::OPTIONS => self.root_option.fetch(pattern),
            Method::GET => self.root_get.fetch(pattern),
            Method::POST => self.root_post.fetch(pattern),
            Method::PUT => self.root_put.fetch(pattern),
            Method::DELETE => self.root_delete.fetch(pattern),
            Method::HEAD => self.root_head.fetch(pattern),
            Method::TRACE => self.root_trace.fetch(pattern),
            Method::CONNECT => self.root_connect.fetch(pattern),
            Method::PATCH => self.root_patch.fetch(pattern),
            Method::LINK => self.root_link.fetch(pattern),
            Method::UNLINK => self.root_unlink.fetch(pattern),
            Method::PRI => self.root_pri.fetch(pattern)
        }
    }
}

#[derive(Clone)]
pub(crate) struct Node {
    /// 资源样式，如`/a/b/:c/d/:e/:f/g`
    pattern: Option<String>,
    /// 资源具体值，如果没有，则为`？` -> a | ?
    pattern_piece: String,
    pattern_piece_value: Option<String>,
    /// 待实现接收请求方法
    pub(crate) handler: Option<Handler>,
    /// 过滤器/拦截器数组
    pub(crate) extend: Option<Extend>,
    pub(crate) next_nodes: Vec<Node>,
}

impl ToString for Node {
    fn to_string(&self) -> String {
        let pattern = self.pattern.clone().unwrap_or("".to_string());
        let pattern_piece = self.pattern_piece.clone();
        let pattern_piece_value = self.pattern_piece_value.clone().unwrap_or("".to_string());
        format!("{}-{}-{}", pattern, pattern_piece, pattern_piece_value)
    }
}

impl<'a> PartialEq<&'a Node> for Node {
    fn eq(&self, other: &&'a Node) -> bool {
        self.to_string() == other.to_string()
    }
}

impl<'a> PartialEq<Node> for &'a Node {
    fn eq(&self, other: &Node) -> bool {
        self.to_string() == other.to_string()
    }
}

impl<'a> PartialEq<Node> for Node {
    fn eq(&self, other: &Node) -> bool {
        self.to_string() == other.to_string()
    }
}

impl Node {
    fn new() -> Self {
        Node {
            pattern: None,
            pattern_piece: "".to_string(),
            pattern_piece_value: None,
            handler: None,
            extend: None,
            next_nodes: vec![],
        }
    }

    pub(crate) fn handler(&self) -> Handler {
        self.handler.unwrap()
    }

    /// 新增节点
    ///
    /// * pattern 资源样式，如`/a/b/:c/d/:e/:f/g`
    /// * method 请求方法
    /// * handler 待实现接收请求方法
    /// * extend 请求服务扩展，包括过滤器、限流、熔断降级等
    fn add(&mut self, pattern: String, method: Method, handler: Handler, extend: Option<Extend>) {
        if !pattern.starts_with("/") {
            panic!("path must begin with '/' in http server!")
        }
        let pattern_c = pattern.clone();
        let ps: Vec<&str> = pattern_c.split("/").collect();
        let pattern_split = ps[1..].to_vec();
        self.add_fn(pattern, method, pattern_split, 0, handler, extend)
    }

    /// 新增可执行服务节点
    ///
    /// * pattern 资源样式，如`/a/b/:c/d/:e/:f/g`
    /// * method 请求方法
    /// * patternSplitArr [a, b, ?, d, ?, ?, g]
    /// * index 当前匹配资源参数下标
    /// * handler 待实现接收请求方法
    /// * extend 请求服务扩展，包括过滤器、限流、熔断降级等
    fn add_fn(&mut self, pattern: String, method: Method, pattern_split: Vec<&str>,
              mut index: usize, handler: Handler, extend: Option<Extend>) {
        let mut pattern_piece = pattern_split[index].to_string();
        let pattern_piece_value;
        if pattern_piece.starts_with(":") {
            pattern_piece_value = Some(pattern_piece[1..].to_string());
            pattern_piece = "?".to_string()
        } else {
            pattern_piece_value = None
        }
        index += 1;
        // 遍历当前节点所有子项，处理存在相同资源情况
        for next_node in self.next_nodes.iter_mut() {
            // 判断当前资源是否已存在于子项中
            if next_node.pattern_piece == pattern_piece {
                next_node.add_split(pattern, method, pattern_split, index, handler, extend);
                return;
            }
        }
        // 没有子项或没有相同子项，需新建子项并处理服务后续
        self.create_next_node(pattern_piece, pattern_piece_value, pattern, method, pattern_split,
                              index, handler, extend)
    }

    fn create_next_node(&mut self, pattern_piece: String, pattern_piece_value: Option<String>,
                        pattern: String, method: Method, pattern_split: Vec<&str>, index: usize,
                        handler: Handler, extend: Option<Extend>) {
        let mut next_node = Node {
            pattern: None,
            pattern_piece,
            pattern_piece_value,
            handler: None,
            extend: None,
            next_nodes: vec![],
        };
        next_node.add_split(pattern, method, pattern_split, index, handler, extend);
        self.next_nodes.push(next_node)
    }

    /// 新增可执行服务节点
    ///
    /// * pattern 资源样式，如`/a/b/:c/d/:e/:f/g`
    /// * method 请求方法
    /// * patternSplitArr [a, b, ?, d, ?, ?, g]
    /// * index 当前匹配资源参数下标
    /// * handler 待实现接收请求方法
    /// * extend 请求服务扩展，包括过滤器、限流、熔断降级等
    fn add_split(&mut self, pattern: String, method: Method, pattern_split: Vec<&str>,
                 index: usize, handler: Handler, extend: Option<Extend>) {
        // 经过多轮递归，判断资源是否已经解析到最终步
        if pattern_split.len() == index { // 如果资源已经解析到最终步，则表明当前结点是叶子结点
            match self.pattern.clone() {
                // 如果已经存在资源，则报错资源重复
                Some(src) => panic!("http server resource {} already exist, old pattern is {}", src, pattern),
                None => { // 如果不存在资源，则新建
                    println!("http server url watch: {} {}", method.as_str(), pattern);
                    self.pattern = Some(pattern);
                    self.handler = Some(handler);
                    self.extend = extend;
                    match self.extend.clone() {
                        Some(src) => match src.limit {
                            Some(src) => { Thread::spawn(move || src.run()).unwrap(); }
                            None => {}
                        }
                        None => {}
                    }
                }
            }
        } else {
            self.add_fn(pattern, method, pattern_split, index, handler, extend)
        }
    }

    /// 获取可执行服务节点
    ///
    /// * pattern 资源样式，如`/a/b/c/d/e/f/g`
    fn fetch(&self, pattern: String) -> Option<(Self, HashMap<String, String>)> {
        let pattern_c = pattern.clone();
        let ps: Vec<&str> = pattern_c.split("/").collect();
        let pattern_split = ps[1..].to_vec();
        let pattern_split_len = pattern_split.len();
        self.fetch_fn(pattern, pattern_split, pattern_split_len, 0)
    }

    /// 获取可执行服务节点
    ///
    /// * pattern 资源样式，如`/a/b/c/d/e/f/g`
    fn fetch_fn(&self, pattern: String, pattern_split: Vec<&str>, pattern_split_len: usize,
                mut index: usize) -> Option<(Self, HashMap<String, String>)> {
        let pattern_piece = pattern_split[index];
        // if pattern_piece.starts_with(":") {
        //     pattern_piece = "?";
        // }
        index += 1;
        // 遍历子结点是否存在相同资源
        for next_node in self.next_nodes.iter() {
            // 如果有，则继续
            if next_node.pattern_piece.eq(pattern_piece) {
                let (is_self, src) = next_node.fetch_split(
                    pattern.clone(), pattern_split.clone(), pattern_split_len, index);
                if is_self {
                    return Some((next_node.clone(), HashMap::new()));
                } else {
                    match src {
                        Some(res) => return Some(res),
                        None => continue
                    }
                }
            } else if next_node.pattern_piece.eq("?") {
                let (is_self, src) = next_node.fetch_split(
                    pattern.clone(), pattern_split.clone(), pattern_split_len, index);
                if is_self { // 如果是子结点，则新建fields集合，开始逆向填充
                    let mut fields = HashMap::new();
                    fields.insert(next_node.pattern_piece_value.clone().unwrap(), pattern_piece.to_string());
                    return Some((next_node.clone(), fields));
                } else { // 如果不是子结点，则执行逆向填充
                    match src {
                        Some((node, mut fields)) => {
                            fields.insert(next_node.pattern_piece_value.clone().unwrap(), pattern_piece.to_string());
                            return Some((node, fields));
                        }
                        None => continue
                    }
                }
            }
        }
        // 如果没有，则返回空
        None
    }

    /// 获取可执行服务节点
    ///
    /// * pattern 资源样式，如`/a/b/c/d/e/f/g`
    /// * patternSplitArr [a, b, ?, d, ?, ?, g]
    ///
    /// # return
    /// * bool 是否返回自身，如果返回自身，则不考虑第二个参数值
    /// * 返回值
    fn fetch_split(&self, pattern: String, pattern_split: Vec<&str>, pattern_split_len: usize,
                   index: usize) -> (bool, Option<(Self, HashMap<String, String>)>) {
        // 经过多轮递归，判断资源是否已经解析到最终步
        if pattern_split_len == index { // 如果资源已经解析到最终步，则表明当前结点是叶子结点
            (true, None)
        } else {
            (false, self.fetch_fn(pattern, pattern_split, pattern_split_len, index))
        }
    }
}

impl Debug for Node {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "pattern: {:#?}, \npattern_piece: {}, \npattern_piece_value: {:#?}, \nhandler.is_some: {:#?}\
        , \nextend: {:#?}, \nnext_nodes: {:#?}"
               , self.pattern, self.pattern_piece, self.pattern_piece_value, self.handler.is_some(),
               self.extend, self.next_nodes)
    }
}

#[cfg(test)]
mod node_test {
    use crate::{Context, Method};
    use crate::server::node::Root;

    #[test]
    fn node_test() {
        let mut root = Root::new();
        root.add("/a/b/:c/d/:e/:f/g".to_string(), Method::POST, h1, None);
        root.add("/a/b/:c/d/e/f".to_string(), Method::POST, h1, None);
        root.add("/a/b/:c/d/:e/:f/g".to_string(), Method::GET, h2, None);
        root.add("/a/c/:c/d/:e/:f/g".to_string(), Method::GET, h2, None);
        root.add("/a/b/:c/d/:e/:f/g".to_string(), Method::PUT, h3, None);

        assert!(root.root_option.next_nodes.is_empty());
        assert!(root.root_delete.next_nodes.is_empty());
        assert!(root.root_head.next_nodes.is_empty());
        assert!(root.root_trace.next_nodes.is_empty());
        assert!(root.root_connect.next_nodes.is_empty());
        assert!(root.root_patch.next_nodes.is_empty());
        assert!(root.root_link.next_nodes.is_empty());
        assert!(root.root_unlink.next_nodes.is_empty());
        assert!(root.root_pri.next_nodes.is_empty());

        // println!("root = {:#?}", root);
    }

    #[test]
    fn node_match_test1() {
        let mut root = Root::new();
        root.add("/a/b/:c/d/:e/:f/g".to_string(), Method::POST, h1, None);
        root.add("/a/b/:c/d/e/f".to_string(), Method::POST, h1, None);
        root.add("/a/b/:c/d/:e/:f/g".to_string(), Method::GET, h2, None);
        root.add("/a/c/:c/d/:e/:f/g".to_string(), Method::GET, h2, None);
        root.add("/a/b/:c/d/:e/:f/g".to_string(), Method::PUT, h3, None);
        root.add("/a/b/:c/d/:e/:f/g/:h".to_string(), Method::PUT, h3, None);

        assert_eq!(root.root_post.next_nodes.len(), 1);
        assert_eq!(root.root_post.next_nodes[0].pattern_piece, "a");
        assert_eq!(root.root_post.next_nodes[0].next_nodes.len(), 1);
        assert_eq!(root.root_post.next_nodes[0].next_nodes[0].pattern_piece, "b");
        assert_eq!(root.root_post.next_nodes[0].next_nodes[0].next_nodes.len(), 1);
        assert_eq!(root.root_post.next_nodes[0].next_nodes[0].next_nodes[0].pattern_piece, "?");
        assert_eq!(root.root_post.next_nodes[0].next_nodes[0].next_nodes[0].next_nodes.len(), 1);
        assert_eq!(root.root_post.next_nodes[0].next_nodes[0].next_nodes[0].next_nodes[0].pattern_piece, "d");
        assert_eq!(root.root_post.next_nodes[0].next_nodes[0].next_nodes[0].next_nodes[0].next_nodes.len(), 2);
        assert_eq!(root.root_post.next_nodes[0].next_nodes[0].next_nodes[0].next_nodes[0].next_nodes[0].pattern_piece, "?");
        assert_eq!(root.root_post.next_nodes[0].next_nodes[0].next_nodes[0].next_nodes[0].next_nodes[0].next_nodes.len(), 1);
        assert_eq!(root.root_post.next_nodes[0].next_nodes[0].next_nodes[0].next_nodes[0].next_nodes[0].next_nodes[0].pattern_piece, "?");
        assert_eq!(root.root_post.next_nodes[0].next_nodes[0].next_nodes[0].next_nodes[0].next_nodes[0].next_nodes[0].next_nodes.len(), 1);
        assert_eq!(root.root_post.next_nodes[0].next_nodes[0].next_nodes[0].next_nodes[0].next_nodes[0].next_nodes[0].next_nodes[0].pattern_piece, "g");
        assert_eq!(root.root_post.next_nodes[0].next_nodes[0].next_nodes[0].next_nodes[0].next_nodes[1].pattern_piece, "e");
        assert_eq!(root.root_post.next_nodes[0].next_nodes[0].next_nodes[0].next_nodes[0].next_nodes[1].next_nodes.len(), 1);
        assert_eq!(root.root_post.next_nodes[0].next_nodes[0].next_nodes[0].next_nodes[0].next_nodes[1].next_nodes[0].pattern_piece, "f");
        // println!("root = {:#?}", root);
        assert_eq!(root.root_put.next_nodes[0].next_nodes[0].next_nodes[0].next_nodes[0].next_nodes[0].next_nodes[0].next_nodes[0].next_nodes.len(), 1);
        assert_eq!(root.root_put.next_nodes[0].next_nodes[0].next_nodes[0].next_nodes[0].next_nodes[0].next_nodes[0].next_nodes[0].next_nodes[0].pattern_piece, "?");
    }

    #[test]
    fn node_match_test2() {
        let mut root = Root::new();
        root.add("/a/b/:c/d/:e/:f/g".to_string(), Method::GET, h2, None);
        root.add("/a/c/:c/d/:e/:f/g".to_string(), Method::GET, h2, None);
        root.add("/d/b/:c/d/:e/:f/g".to_string(), Method::GET, h3, None);

        assert_eq!(root.root_get.next_nodes.len(), 2);
        assert_eq!(root.root_get.next_nodes[0].pattern_piece, "a");
        assert_eq!(root.root_get.next_nodes[1].pattern_piece, "d");
        assert_eq!(root.root_get.next_nodes[0].next_nodes.len(), 2);
        assert_eq!(root.root_get.next_nodes[0].next_nodes[0].pattern_piece, "b");
        assert_eq!(root.root_get.next_nodes[0].next_nodes[1].pattern_piece, "c");
        assert_eq!(root.root_get.next_nodes[0].next_nodes[0].next_nodes.len(), 1);
        assert_eq!(root.root_get.next_nodes[0].next_nodes[0].next_nodes[0].pattern_piece, "?");
        assert_eq!(root.root_get.next_nodes[0].next_nodes[0].next_nodes[0].next_nodes.len(), 1);
        assert_eq!(root.root_get.next_nodes[0].next_nodes[0].next_nodes[0].next_nodes[0].pattern_piece, "d");
        assert_eq!(root.root_get.next_nodes[0].next_nodes[0].next_nodes[0].next_nodes[0].next_nodes.len(), 1);
        assert_eq!(root.root_get.next_nodes[0].next_nodes[0].next_nodes[0].next_nodes[0].next_nodes[0].pattern_piece, "?");
        assert_eq!(root.root_get.next_nodes[0].next_nodes[0].next_nodes[0].next_nodes[0].next_nodes[0].next_nodes.len(), 1);
        assert_eq!(root.root_get.next_nodes[0].next_nodes[0].next_nodes[0].next_nodes[0].next_nodes[0].next_nodes[0].pattern_piece, "?");
        assert_eq!(root.root_get.next_nodes[0].next_nodes[0].next_nodes[0].next_nodes[0].next_nodes[0].next_nodes[0].next_nodes.len(), 1);
        assert_eq!(root.root_get.next_nodes[0].next_nodes[0].next_nodes[0].next_nodes[0].next_nodes[0].next_nodes[0].next_nodes[0].pattern_piece, "g");
    }

    #[test]
    fn node_fetch_test() {
        let mut root = Root::new();
        root.add("/a/b/:c/d/:e/:f/g".to_string(), Method::POST, h1, None);
        root.add("/a/b/:c/d/e/f".to_string(), Method::POST, h2, None);
        root.add("/a/b/:c/d/:e/:f/g".to_string(), Method::GET, h1, None);
        root.add("/a/c/:c/d/:e/:f/g".to_string(), Method::GET, h2, None);
        root.add("/d/b/:c/d/:e/:f/g".to_string(), Method::GET, h3, None);
        root.add("/a/b/:c/d/:e/:f/g".to_string(), Method::PUT, h1, None);
        root.add("/a/b/:c/d/:e/:f/g/:h".to_string(), Method::PUT, h1, None);

        // let (n1, _fields) = root.fetch("/a/b/c/d/e/f/g".to_string(), Method::POST).unwrap();
        // assert_eq!(n1.handler, root.root_post.next_nodes[0].next_nodes[0].next_nodes[0].next_nodes[0].next_nodes[0].next_nodes[0].next_nodes[0].handler);

        let (n1, _fields) = root.fetch("/a/b/c/d/e/f/g/h".to_string(), Method::PUT).unwrap();
        assert_eq!(n1, root.root_put.next_nodes[0].next_nodes[0].next_nodes[0].next_nodes[0].next_nodes[0].next_nodes[0].next_nodes[0].next_nodes[0]);
    }

    fn h1(_context: &mut Context) {}

    fn h2(_context: &mut Context) {}

    fn h3(_context: &mut Context) {}
}

