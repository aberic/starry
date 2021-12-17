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

use bytes::Bytes;

use crate::{ContentType, Cookie, Header, Status, Version};
use crate::http::body::Body;
use crate::utils::errors::StarryResult;

/// Response 表示由服务器响应客户端发送的HTTP请求。
///
/// 响应报文格式如下所示：
/// ```response
/// ┌───────────────────────────────────────────────────┐
/// │ Http-Version | Status-Code | Reason-Phrase | CRLF │
/// ├───────────────────────────────────────────────────┤
/// │                      Header                       │
/// ├───────────────────────────────────────────────────┤
/// │                       CRLF                        │
/// ├───────────────────────────────────────────────────┤
/// │                       Body                        │
/// └───────────────────────────────────────────────────┘
/// ```
///
/// 状态行也由三部分组成，包括HTTP协议的版本，状态码，以及对状态码的文本描述。例如：
/// HTTP/1.1 200 OK （CRLF）
#[derive(Clone, Debug)]
pub struct Response {
    pub(crate) version: Version,
    pub(crate) status: Status,
    pub(crate) header: Header,
    pub(crate) body: Body,
    /// ContentLength 记录相关内容的长度。
    ///
    /// -1表示长度未知。
    ///
    /// 除非请求方法为"HEAD"，value >= 0 表示可以从Body中读取给定的字节数。
    pub(crate) content_length: usize,
    /// Close 记录头是否指示在读取Body后关闭连接。
    pub(crate) close: bool,
}

/// 组合方法集
impl Response {
    pub fn add_header(&mut self, k: String, v: String) {
        self.header.add(k, v)
    }

    pub fn add_header_str(&mut self, k: &str, v: &str) {
        self.header.add_str(k, v)
    }
    pub fn set_header(&mut self, k: String, v: String) {
        self.header.set(k, v)
    }

    pub fn set_header_str(&mut self, k: &str, v: &str) {
        self.header.set_str(k, v)
    }

    pub fn set_content_type(&mut self, src: ContentType) {
        self.header.set_content_type(src)
    }

    pub fn add_set_cookie(&mut self, cookie: Cookie) {
        self.header.add_set_cookie(cookie)
    }

    /// 获取response设置的cookies，来自Header`"Set-Cookie"`
    pub fn read_set_cookies(&self) -> StarryResult<Vec<Cookie>> {
        self.header.read_set_cookies()
    }

    pub fn write(&mut self, body: Vec<u8>) {
        self.content_length = body.len();
        self.header.set_content_length(self.content_length);
        self.header.set_content_type(ContentType::TEXT_PLAIN);
        self.body.write(body)
    }

    pub fn write_bytes(&mut self, body: &'static [u8]) {
        self.content_length = body.len();
        self.header.set_content_length(self.content_length);
        self.header.set_content_type(ContentType::TEXT_PLAIN);
        self.body.write_bytes(body)
    }

    pub fn write_type(&mut self, body: Vec<u8>, content_type: ContentType) {
        self.content_length = body.len();
        self.header.set_content_length(self.content_length);
        self.header.set_content_type(content_type);
        self.body.write(body)
    }

    pub fn write_bytes_type(&mut self, body: &'static [u8], content_type: ContentType) {
        self.content_length = body.len();
        self.header.set_content_length(self.content_length);
        self.header.set_content_type(content_type);
        self.body.write_bytes(body)
    }

    /// 返回已写入数据，该操作会清空已写入数据
    pub(crate) fn get_write_content(&mut self) -> Bytes {
        self.content_length = 0;
        self.header.del_content_length();
        self.header.del_content_type();
        self.body.get_write_content()
    }
}

/// 输出方法集
impl Response {
    pub(crate) fn new(version: Version, connection: bool) -> Self {
        let mut resp = Response {
            version,
            status: Status::OK,
            header: Header::new(),
            body: Default::default(),
            content_length: 0,
            close: !connection,
        };
        if connection {
            resp.header.set_connection();
        }
        resp
    }

    /// 返回 200 OK
    ///
    /// 成功：服务器已成功处理了请求。 通常，这表示服务器提供了请求的网页。
    pub fn success() -> Response {
        fill(Status::OK)
    }

    /// 返回 400 Bad Request
    ///
    /// 错误请求：服务器不理解请求的语法。
    pub fn bad_request() -> Response {
        fill(Status::BAD_REQUEST)
    }

    /// 返回 401 Unauthorized
    ///
    /// 未授权：请求要求身份验证。对于需要登录的网页，服务器可能返回此响应。
    pub fn unauthorized() -> Response {
        fill(Status::UNAUTHORIZED)
    }

    /// 返回 403 Forbidden
    ///
    /// 禁止：服务器拒绝请求。
    pub fn forbidden() -> Response {
        fill(Status::FORBIDDEN)
    }

    /// 返回 404 Not Found
    ///
    /// 未找到：服务器找不到请求的网页。
    pub fn not_found() -> Response {
        fill(Status::NOT_FOUND)
    }

    /// 返回 405 Method Not Allowed
    ///
    /// 方法禁用：禁用请求中指定的方法。
    pub fn method_not_allowed() -> Response {
        fill(Status::METHOD_NOT_ALLOWED)
    }

    /// 返回 406 Not Acceptable
    ///
    /// 不接受：无法使用请求的内容特性响应请求的网页。
    pub fn not_acceptable() -> Response {
        fill(Status::NOT_ACCEPTABLE)
    }

    /// 返回 408 Request Timeout
    ///
    /// 请求超时：服务器等候请求时发生超时。
    pub fn request_timeout() -> Response {
        fill(Status::REQUEST_TIMEOUT)
    }

    /// 505 HTTP Version Not Supported
    /// HTTP 版本不受支持：服务器不支持请求中所用的 HTTP 协议版本。
    pub fn http_version_not_supported() -> Response {
        fill(Status::HTTP_VERSION_NOT_SUPPORTED)
    }

    /// 411 Length Required
    /// 需要有效长度：服务器不接受不含有效内容长度标头字段的请求。
    pub fn length_required() -> Response {
        fill(Status::LENGTH_REQUIRED)
    }

    /// 417 Expectation Failed
    /// 未满足期望值：服务器未满足"期望"请求标头字段的要求。
    pub fn expectation_failed() -> Response {
        fill(Status::EXPECTATION_FAILED)
    }

    pub fn custom(status: Status) -> Response {
        fill(status)
    }

    pub(crate) fn version(&mut self, version: Version) {
        self.version = version
    }

    pub(crate) fn status(&mut self, status: Status) {
        self.status = status
    }
}

fn fill(status: Status) -> Response {
    Response {
        version: Default::default(),
        status,
        header: Header::new(),
        body: Default::default(),
        content_length: 0,
        close: true,
    }
}

#[cfg(test)]
mod response_test {
    use std::ops::Add;

    use crate::Response;

    impl Response {
        fn string(&mut self) -> String {
            String::from_utf8_lossy(self.bytes().as_slice()).to_string()
        }

        fn bytes(&mut self) -> Vec<u8> {
            // 状态行
            let mut status_line = self.version.to_string()
                .add(" ")
                .add(self.status.code().to_string().as_str())
                .add(" ")
                .add(self.status.phrase())
                .into_bytes();
            // 头部块
            let mut header_block = vec![];
            for (key, values) in self.header.map() {
                for value in values {
                    let tmp = String::new();
                    let tmp = tmp.add(&key).add(": ").add(&value);
                    header_block.append(&mut tmp.into_bytes());
                    header_block.push(b'\r');
                    header_block.push(b'\n');
                }
            }
            status_line.push(b'\r');
            status_line.push(b'\n');
            status_line.append(&mut header_block);
            status_line.push(b'\r');
            status_line.push(b'\n');
            status_line.append(&mut self.body.body().to_vec());
            status_line
        }
    }

    #[test]
    fn status_to_str() {
        let mut r1 = Response::bad_request();
        assert_eq!(r1.string(), "HTTP/1.1 400 Bad Request\r\n\r\n");

        let mut r2 = Response::success();
        assert_eq!(r2.string(), "HTTP/1.1 200 OK\r\n\r\n");
    }
}
