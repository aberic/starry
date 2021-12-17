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

use bytes::{Buf, Bytes, BytesMut};

#[derive(Debug, Clone)]
pub(crate) struct Body {
    /// 允许且只允许初始化一次
    init: bool,
    /// Body 是接收到的请求/应答的主体。
    /// 对于客户端请求，主体长度为空意味着请求没有主体，比如GET请求。HTTP客户端的Transport负责调用Close方法。
    /// 对于服务器请求，请求体总是非空的，但当没有请求体时将立即返回EOF。服务器将关闭请求体。
    pub(crate) reader: Bytes,
    /// Body 是即将发送的请求/应答的主体。
    /// 对于客户端请求，主体长度为空意味着请求没有主体，比如GET请求。HTTP客户端的Transport负责调用Close方法。
    /// 对于服务器请求，请求体总是非空的，但当没有请求体时将立即返回EOF。服务器将关闭请求体。
    pub(crate) writer: Bytes,
}

impl Body {
    /// 获取接收的主体信息，随着read会清空body内容
    pub(crate) fn body(&mut self) -> Bytes {
        self.reader.copy_to_bytes(self.reader.len())
    }

    /// 初始化读取器
    pub(crate) fn init_reader(&mut self, bm: BytesMut) {
        if !self.init {
            self.reader = Bytes::from(bm)
        }
    }

    /// 仅对body.reader操作
    pub(crate) fn len(&self) -> usize {
        self.reader.len()
    }

    pub(crate) fn write(&mut self, src: Vec<u8>) {
        self.writer = Bytes::from(src)
    }

    pub(crate) fn write_bytes(&mut self, src: &'static [u8]) {
        self.writer = Bytes::from(src)
    }

    /// 返回已写入数据，该操作会清空已写入数据
    pub(crate) fn get_write_content(&mut self) -> Bytes {
        self.writer.copy_to_bytes(self.writer.len())
    }
}

impl Default for Body {
    fn default() -> Self {
        Body { init: false, reader: Default::default(), writer: Default::default() }
    }
}

impl ToString for Body {
    /// 仅对body.reader操作
    fn to_string(&self) -> String {
        String::from_utf8_lossy(self.reader.as_ref()).to_string()
    }
}
