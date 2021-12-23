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
use std::fmt::{Debug, Formatter};
use std::hash::Hash;

use crate::http::header::ContentType;

// pub trait RequestValues<Value> {
//     fn new() -> Self;
//     /// 将键值对插入到映射中。
//     /// 如果映射没有这个键，则返回None。
//     /// 如果映射确实存在此键，则更新值，并返回旧值。
//     fn insert(&mut self, name: String, value: Value) -> Option<Value>;
//
//     /// 返回对对应于键的值的引用。
//     /// 键可以是映射的键类型的任何借用形式，但是借用形式上的Hash和Eq必须与键类型匹配。
//     fn get(&self, k: &String) -> Option<&Value>;
//
//     /// 返回对对应于键的值的引用。
//     /// 键可以是映射的键类型的任何借用形式，但是借用形式上的Hash和Eq必须与键类型匹配。
//     fn get_str(&self, k: &str) -> Option<&Value>;
//
//     /// 返回对对应于键存在性。
//     fn have_str(&self, k: &str) -> bool;
//
//     /// 删除键
//     /// 返回对对应于键的值的引用。
//     fn del(&mut self, k: &String) -> Option<Value>;
//
//     /// 删除键
//     /// 返回对对应于键的值的引用。
//     fn del_str(&mut self, k: &str) -> Option<Value>;
//
//     fn len(&self) -> usize;
//
//     fn map(&self) -> HashMap<String, Value>;
// }

/// Values 将字符串键映射到值列表。
/// 它通常用于查询参数和表单值。
/// 不像http.header映射，值映射中的键是大小写敏感的。
#[derive(Clone, Debug)]
pub struct Values(HashMap<String, Vec<String>>);

// impl Values {
//     pub fn new() -> Values {
//         Values(HashMap::new())
//     }
// }

impl Values {
    pub fn new() -> Values {
        Values(HashMap::new())
    }

    /// 将键值对插入到映射中。
    /// 如果映射确实存在此键，则更新值
    pub fn set(&mut self, k: String, v: String) {
        self.0.insert(k, vec![v]);
    }

    /// 将键值对插入到映射中。
    /// 如果映射确实存在此键，则更新值
    pub fn set_str(&mut self, k: &str, v: &str) {
        self.set(k.to_string(), v.to_string())
    }

    /// 将键值对插入到映射中。
    /// 如果映射确实存在此键，则追加值
    pub fn add(&mut self, k: String, v: String) {
        match self.0.get(&k) {
            Some(src) => {
                let mut res = src.clone();
                res.push(v);
                self.0.insert(k, res);
            }
            None => {
                self.0.insert(k, vec![v]);
            }
        }
    }

    /// 将键值对插入到映射中。
    /// 如果映射确实存在此键，则追加值
    pub fn add_str(&mut self, k: &str, v: &str) {
        self.add(k.to_string(), v.to_string())
    }

    /// 返回对应于键的值的引用。
    /// 键可以是映射的键类型的任何借用形式，但是借用形式上的Hash和Eq必须与键类型匹配。
    pub fn get<K: ?Sized>(&self, k: &K) -> Option<String> where
        K: Borrow<K>,
        K: Hash + Eq,
        String: Borrow<K>, {
        match self.0.get(k) {
            Some(src) => Some(src[0].clone()),
            None => None
        }
    }

    /// 返回对应于键的值的引用。
    /// 键可以是映射的键类型的任何借用形式，但是借用形式上的Hash和Eq必须与键类型匹配。
    pub fn vec<K: ?Sized>(&self, k: &K) -> Option<Vec<String>> where
        K: Borrow<K>,
        K: Hash + Eq,
        String: Borrow<K>, {
        match self.0.get(k) {
            Some(src) => Some(src.clone()),
            None => None
        }
    }

    /// 返回对应于键存在性。
    pub fn contain<K: ?Sized>(&self, k: &K) -> bool where
        K: Borrow<K>,
        K: Hash + Eq,
        String: Borrow<K>, {
        match self.0.get(k) {
            Some(_) => true,
            None => false,
        }
    }

    /// 删除键
    /// 返回对应于键的值的引用。
    pub fn del<K: ?Sized>(&mut self, k: &K) -> Option<String> where
        K: Borrow<K>,
        K: Hash + Eq,
        String: Borrow<K>, {
        match self.0.remove(k) {
            Some(src) => Some(src[0].clone()),
            None => None
        }
    }

    pub fn len(&self) -> usize {
        self.0.len()
    }

    pub fn map(&self) -> HashMap<String, Vec<String>> {
        self.0.clone()
    }
}

/// MultipartValues 是一个已解析的多部分表单。
/// 它的File部分存储在内存或磁盘上，内置相关方法访问。
/// 它的Value部分被存储为字符串。两者都是通过字段名进行键控的。
#[derive(Clone, Debug)]
pub struct MultipartValues(HashMap<String, FileHeader>);

impl MultipartValues {
    pub fn new() -> MultipartValues {
        MultipartValues(HashMap::new())
    }

    /// 将键值对插入到映射中。
    /// 如果映射没有这个键，则返回None。
    /// 如果映射确实存在此键，则更新值，并返回旧值。
    pub fn insert(&mut self, name: String, value: FileHeader) -> Option<FileHeader> {
        self.0.insert(name, value)
    }

    pub fn insert_obj(&mut self, name: String, filename: String, content: Vec<u8>, content_type: ContentType) -> Option<FileHeader> {
        self.insert(name, FileHeader {
            filename,
            size: content.len(),
            content,
            content_type,
        })
    }

    /// 返回对对应于键的值的引用。
    /// 键可以是映射的键类型的任何借用形式，但是借用形式上的Hash和Eq必须与键类型匹配。
    pub fn get<K: ?Sized>(&self, k: &K) -> Option<FileHeader> where
        K: Borrow<K>,
        K: Hash + Eq,
        String: Borrow<K>, {
        match self.0.get(k) {
            Some(src) => Some(src.clone()),
            None => None
        }
    }

    /// 返回对对应于键存在性。
    pub fn have<K: ?Sized>(&self, k: &K) -> bool where
        K: Borrow<K>,
        K: Hash + Eq,
        String: Borrow<K>, {
        match self.0.get(k) {
            Some(_) => true,
            None => false,
        }
    }

    /// 删除键
    /// 返回对对应于键的值的引用。
    pub fn del<K: ?Sized>(&mut self, k: &K) -> Option<FileHeader> where
        K: Borrow<K>,
        K: Hash + Eq,
        String: Borrow<K>, {
        self.0.remove(k)
    }

    pub fn len(&self) -> usize {
        self.0.len()
    }

    pub fn map(&self) -> HashMap<String, FileHeader> {
        self.0.clone()
    }
}

/// 描述请求的文件部分
#[derive(Clone)]
pub struct FileHeader {
    filename: String,
    size: usize,
    content: Vec<u8>,
    /// 指定基础数据的媒体类型
    content_type: ContentType,
}

impl FileHeader {
    pub fn new(filename: String,
               size: usize,
               content: Vec<u8>,
               content_type: ContentType) -> FileHeader {
        FileHeader {
            filename,
            size,
            content,
            content_type,
        }
    }

    pub fn filename(&self) -> String {
        self.filename.clone()
    }

    pub fn size(&self) -> usize {
        self.size
    }

    pub fn content(&self) -> Vec<u8> {
        self.content.clone()
    }
}

impl Debug for FileHeader {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "filename: {}, \nsize: {}, \ncontent: {}, \ncontent_type: {}"
               , self.filename, self.size, String::from_utf8_lossy(self.content.as_slice()),
               self.content_type.as_str())
    }
}

#[cfg(test)]
mod values_test {
    use crate::{MultipartValues, Values};
    use crate::header::ContentType;
    use crate::http::values::FileHeader;

    #[test]
    fn values() {
        let mut vs = Values::new();
        vs.set(String::from("a"), String::from("b"));
        vs.set(String::from("c"), String::from("d"));
        vs.set(String::from("e"), String::from("f"));
        assert_eq!(Some(String::from("b")), vs.get(&String::from("a")));
        assert_eq!(Some(String::from("b")), vs.get("a"));
        for (key, value) in vs.map() {
            if key.eq("a") {
                assert_eq!("b", value[0].as_str());
            } else if key.eq("c") {
                assert_eq!("d", value[0].as_str());
            } else if key.eq("e") {
                assert_eq!("f", value[0].as_str());
            }
        }
        assert!(vs.contain("a"));
        assert!(!vs.contain("x"));
        assert_eq!(3, vs.len());
        vs.del("a");
        assert!(!vs.contain("a"));
        assert_eq!(2, vs.len());
        vs.del(&String::from("c"));
        assert!(!vs.contain("c"));
        assert_eq!(1, vs.len());
    }

    #[test]
    fn values_add() {
        let mut vs = Values::new();
        vs.add(String::from("a"), String::from("b"));
        vs.add(String::from("a"), String::from("d"));
        vs.add(String::from("a"), String::from("f"));
        assert_eq!(Some(String::from("b")), vs.get("a"));
        for (key, values) in vs.map() {
            assert_eq!(String::from("a"), key);
            assert_eq!(String::from("b"), values[0]);
            assert_eq!(String::from("d"), values[1]);
            assert_eq!(String::from("f"), values[2]);
        }
    }

    #[test]
    fn file_headers() {
        let mut vs1 = MultipartValues::new();
        vs1.insert(
            String::from("a"),
            FileHeader::new(String::from("b"), 0, vec![],
                            ContentType::default()));
        vs1.insert(
            String::from("c"),
            FileHeader::new(String::from("d"), 0, vec![],
                            ContentType::default()));
        vs1.insert(
            String::from("e"),
            FileHeader::new(String::from("f"), 0, vec![],
                            ContentType::default()));
        assert_eq!("b", vs1.get(&String::from("a")).unwrap().filename().as_str());
        assert_eq!("b", vs1.get("a").unwrap().filename().as_str());
        for (key, value) in vs1.map() {
            if key.eq("a") {
                assert_eq!("b", value.filename().as_str());
            } else if key.eq("c") {
                assert_eq!("d", value.filename().as_str());
            } else if key.eq("e") {
                assert_eq!("f", value.filename().as_str());
            }
        }
        assert!(vs1.have("a"));
        assert!(!vs1.have("x"));
        assert_eq!(3, vs1.len());
        vs1.del("a");
        assert!(!vs1.have("a"));
        assert_eq!(2, vs1.len());
        vs1.del(&String::from("c"));
        assert!(!vs1.have("c"));
        assert_eq!(1, vs1.len());
    }
}

