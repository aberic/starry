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
use std::fmt::{Debug, Display, Formatter};
use std::hash::Hash;
use std::net::SocketAddr;

use bytes::{Bytes, BytesMut};

use crate::{Method, URL, Values, Version};
use crate::header::AcceptEncoding;
use crate::http::body::Body;
use crate::http::header::{ContentType, Cookie};
use crate::http::header::Header;
use crate::http::url::{Authority, Location, Scheme};
use crate::http::url::authority::{Addr, Userinfo};
use crate::http::values::MultipartValues;
use crate::http::version::Protocol;
use crate::utils::errors::StarryResult;

// use crate::values::{MultipartValues, RequestValues};

/// Request 表示由服务器接收或由客户端发送的HTTP请求。
///
/// 请求报文格式如下所示：
/// ```request
/// ┌────────────────────────────────────────────┐
/// │ Method | Request-URI | Http-Version | CRLF │
/// ├────────────────────────────────────────────┤
/// │                    Header                  │
/// ├────────────────────────────────────────────┤
/// │                     CRLF                   │
/// ├────────────────────────────────────────────┤
/// │                     Body                   │
/// └────────────────────────────────────────────┘
/// ```
///
/// 请求行由请求Method, URL 字段和HTTP Version三部分构成, 总的来说请求行就是定义了本次请求的请求方式, 请求的地址, 以及所遵循的HTTP协议版本例如：
/// GET /example.html HTTP/1.1 (CRLF)
///
/// 客户端和服务器使用的字段语义略有不同。除了下面关于字段的说明之外，还请参阅有关HTTP Request的文档。
#[derive(Debug, Clone)]
pub struct Request {
    /// Method 指定HTTP协议中定义的方法（如GET、POST、PUT等）。
    /// 对于客户端请求，空字符串表示GET。
    pub(crate) method: Method,
    /// URL表示已解析的URL。 表示的一般形式是:[scheme][://][userinfo@][addr][/path][?查询][#片段]
    pub(crate) url: URL,
    /// 收/发请求的协议版本
    /// HTTP/1.1或HTTP/2等
    pub(crate) version: Version,
    pub(crate) header: Header,
    /// Body 是请求的主体。
    /// 对于客户端请求，主体长度为空意味着请求没有主体，比如GET请求。HTTP客户端的Transport负责调用Close方法。
    /// 对于服务器请求，请求体总是非空的，但当没有请求体时将立即返回EOF。服务器将关闭请求体。
    pub(crate) body: Body,
    /// ContentLength 记录相关内容的长度。-1表示长度未知。值>= 0表示可以从Body中读取给定的字节数。
    /// 对于客户端请求，值为0且Body非空也被视为未知。
    pub(crate) content_length: isize,
    /// 指使用了哪种压缩方式传输数据，accept-encoding表示你发送请求时告诉服务器，可以解压这些格式的数据。
    pub(crate) accept_encoding: AcceptEncoding,
    /// Close 指在响应此请求后(对于服务器)还是在发送此请求并读取其响应后(对于客户端)关闭连接。
    /// 对于服务器请求，HTTP服务器自动处理此字段，处理程序不需要此字段。
    /// 对于客户端请求，设置此字段可以防止在请求到相同主机之间重用TCP连接。
    pub(crate) close: bool,
    /// 请求中的“Host”报头字段提供来自目标URI的主机和端口信息，使源服务器在为单个IP地址上的多个主机名请求提供服务时
    /// 能够区分资源，如：Host = uri-host [":" port]，详见[`RFC7230，第2.7.1节`]
    /// 对于服务器请求，Host指定搜索URL的主机地址。
    ///
    /// 对于HTTP/1(根据[`RFC7230，第5.4节`])，客户端必须在所有HTTP/1.1请求消息中发送Host报头字段。
    /// 如果目标URI包含一个"authority"组件，那么客户端必须为Host发送一个与该"authority"组件相同的字段值，
    /// 不包括任何"userinfo"子组件及其"@"分隔符(详见[`RFC7230，第2.7.1节`])。
    /// 如果目标URI缺少或未定义"authority"组件，那么客户端必须发送一个Host报头字段，该字段值为空。
    ///
    /// 对于HTTP/2，它是伪头字段“:authority”的值。
    /// 它可以是“host:port”的形式。
    /// 对于国际域名，Host可以是Punycode或Unicode格式。
    ///
    /// 为了防止DNS重新绑定攻击，服务器处理程序应该验证Host报头具有一个处理程序认为自己具有权威性的值。
    /// 对于客户端请求，Host可选地覆盖要发送的Host头。
    /// 如果为空，则使用[`Request.URL.Authority.Addr`]的值。主机可以包含一个国际域名。
    ///
    /// 由于Host字段值是处理请求的关键信息，用户代理应该生成Host作为请求行之后的第一个报头字段。
    ///
    /// 例如，对源服务器<http://www.example.org/pub/WWW/>的GET请求将以:
    /// GET /pub/WWW/ HTTP/1.1
    /// Host: www.example.org
    ///
    /// 即使请求目标是绝对形式，客户端也必须在HTTP/1.1请求中发送Host报头字段，因为这允许通过可能没有实现Host的
    /// HTTP/1.0代理转发Host信息。
    ///
    /// 当代理接收到一个request-target绝对形式的请求时，代理必须忽略接收到的Host报头字段(如果有的话)，
    /// 而用request-target的主机信息替换它。转发此类请求的代理必须根据接收到的请求目标生成一个新的Host字段值，
    /// 而不是转发接收到的Host字段值。
    ///
    /// 由于Host报头字段充当应用程序级别的路由机制，它是恶意软件寻求毒害共享缓存或将请求重定向到意外服务器的常见目标。
    /// 如果拦截代理依赖Host字段值将请求重定向到内部服务器，或者在共享缓存中用作缓存键，而不首先验证拦截的连接针对的
    /// 是该主机的有效IP地址，那么它就特别容易受到攻击。
    /// 服务器必须用400(坏请求)状态码响应任何缺少Host报头字段的HTTP/1.1请求消息，
    /// 以及任何包含超过一个Host报头字段或含有无效字段值的Host报头字段的请求消息。
    ///
    /// [`RFC7230，第2.7.1节`]: https://datatracker.ietf.org/doc/html/rfc7230#section-2.7.1
    /// [`RFC7230，第5.4节`]: https://datatracker.ietf.org/doc/html/rfc7230#section-5.4
    /// [`Request.URL.Authority.Addr`]: crate::url::authority::Addr
    pub(crate) host: String,
    /// 指定基础数据的媒体类型
    pub(crate) content_type: Option<ContentType>,
    /// 表单包含解析后的表单数据，来自PATCH、POST或PUT中URL字段的查询参数。
    pub(crate) form_param: Values,
    /// PostForm包含来自PATCH、POST或PUT主体参数的解析表单数据。
    pub(crate) form: Values,
    /// MultipartValues 是一个已解析的多部分表单。
    /// 它的File部分存储在内存或磁盘上，内置相关方法访问。
    /// 它的Value部分被存储为字符串。两者都是通过字段名进行键控的。
    pub(crate) multipart_form: MultipartValues,
    pub(crate) cookies: Vec<Cookie>,
    pub(crate) client: Addr,
    /// body数据是否已经被解析，如被解析，则无法再次解析
    pub(crate) body_parse: bool,
}

impl Request {
    /// 通过读出流生成客户端Request
    ///
    /// * target 即将访问的服务端地址
    pub(crate) fn new(method: Method, url: &str) -> StarryResult<Request> {
        let url = URL::from(url)?;
        let host = url.authority.addr.to_string();
        Ok(Request {
            method,
            url,
            version: Default::default(),
            header: Header::new(),
            body: Default::default(),
            content_length: -1,
            accept_encoding: AcceptEncoding::None,
            close: true,
            host,
            content_type: None,
            form_param: Values::new(),
            form: Values::new(),
            multipart_form: MultipartValues::new(),
            cookies: vec![],
            client: Default::default(),
            body_parse: false,
        })
    }

    pub fn method(&self) -> Method {
        self.method.clone()
    }

    pub fn url(&self) -> URL {
        self.url.clone()
    }

    pub fn scheme(&self) -> Scheme {
        self.url.scheme()
    }

    pub fn authority(&self) -> Authority {
        self.url.authority()
    }

    pub fn location(&self) -> Location {
        self.url.location()
    }

    pub fn userinfo(&self) -> Option<Userinfo> {
        self.url.authority.userinfo()
    }

    pub fn addr(&self) -> Addr {
        self.url.authority.addr()
    }

    pub fn socket_addr(&self) -> StarryResult<SocketAddr> {
        self.url.authority.addr.socket_addr()
    }

    pub fn path(&self) -> String {
        self.url.location.path()
    }

    pub fn version(&self) -> Version {
        self.version.clone()
    }

    pub fn version_protocol(&self) -> Protocol {
        self.version.protocol()
    }

    pub fn version_major(&self) -> u8 {
        self.version.major()
    }

    pub fn version_minor(&self) -> u8 {
        self.version.minor()
    }

    pub fn header(&self) -> Header {
        self.header.clone()
    }

    pub fn header_get<K: ?Sized>(&self, k: &K) -> Option<String> where
        K: Borrow<K>,
        K: Hash + Eq,
        String: Borrow<K>, {
        self.header.get(k)
    }

    pub fn content_type(&self) -> Option<ContentType> {
        self.content_type.clone()
    }

    pub fn accept_encoding(&self) -> AcceptEncoding {
        self.accept_encoding.clone()
    }

    pub fn cookies(&self) -> Vec<Cookie> {
        self.cookies.clone()
    }

    pub fn req_cookies(&self) -> Vec<Cookie> {
        self.cookies.clone()
    }

    pub fn cookie_get(&self, cookie_name: &str) -> Option<Cookie> {
        for cookie in self.cookies.iter() {
            if cookie.name.eq(cookie_name) {
                return Some(cookie.clone());
            }
        }
        None
    }

    pub fn body(&mut self) -> Vec<u8> {
        self.body_parse = true;
        self.body.body().to_vec()
    }

    /// 返回对应于URI请求参数中定义参数值的引用。
    pub fn form_set(&mut self, k: String, v: String) {
        self.form.set(k, v)
    }

    pub fn multipart_form_insert(&mut self, name: String, filename: String, content: Vec<u8>, content_type: ContentType) {
        self.multipart_form.insert_obj(name, filename, content, content_type);
    }

    /// 返回对应于URI请求参数中定义参数值的引用。
    pub fn param_value<K: ?Sized>(&self, k: &K) -> Option<String> where
        K: Borrow<K>,
        K: Hash + Eq,
        String: Borrow<K>, {
        match self.form_param.get(k) {
            Some(src) => Some(src.clone()),
            None => None
        }
    }

    /// 返回对应于URI请求参数中定义的参数存在性。
    pub fn have_param_value<K: ?Sized>(&self, k: &K) -> bool where
        K: Borrow<K>,
        K: Hash + Eq,
        String: Borrow<K>, {
        match self.form_param.get(k) {
            Some(_) => true,
            None => false,
        }
    }

    /// URI请求参数中定义的参数数量
    pub fn count_param_value(&self) -> usize {
        self.form_param.len()
    }

    pub fn client(&self) -> Addr {
        self.client.clone()
    }
}

impl Request {
    pub fn set_client(&mut self, client: Addr) {
        self.client = client
    }

    pub fn set_url(&mut self, url: URL) {
        self.url = url
    }

    pub fn set_host(&mut self, host: String) {
        self.host = host
    }

    pub(crate) fn set_method(&mut self, method: Method) {
        self.method = method
    }

    pub(crate) fn set_form_param(&mut self, k: String, v: String) {
        self.form_param.set(k, v)
    }

    pub(crate) fn set_header(&mut self, k: String, v: String) {
        self.header.set(k, v);
    }

    pub(crate) fn set_cookies(&mut self, cookies: Vec<Cookie>) {
        self.cookies = cookies
    }

    pub(crate) fn set_version(&mut self, version: Version) {
        self.version = version
    }

    pub(crate) fn set_content_length(&mut self, content_length: isize) {
        self.content_length = content_length
    }

    pub(crate) fn set_content_type(&mut self, content_type: Option<ContentType>) {
        self.content_type = content_type
    }

    pub(crate) fn set_accept_encoding(&mut self, accept_encoding: AcceptEncoding) {
        self.accept_encoding = accept_encoding
    }

    pub(crate) fn set_body(&mut self, bm: BytesMut) {
        self.body.init_reader(bm);
    }

    /// 返回已写入数据，该操作会清空已写入数据
    pub(crate) fn get_write_content(&mut self) -> Bytes {
        self.body.get_write_content()
    }
}

impl Default for Request {
    fn default() -> Self {
        Request {
            method: Method::GET,
            url: URL::default(),
            version: Default::default(),
            header: Header::new(),
            body: Default::default(),
            content_length: -1,
            accept_encoding: AcceptEncoding::None,
            close: true,
            host: String::new(),
            content_type: None,
            form_param: Values::new(),
            form: Values::new(),
            multipart_form: MultipartValues::new(),
            cookies: vec![],
            client: Default::default(),
            body_parse: false,
        }
    }
}

impl Display for Request {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "\nmethod: {:#?}, \nurl: {:#?}, \nversion: {:#?}, \nheader: {:#?}, \nbody: {}, \
        \ncontent_length: {}, \nclose: {}, \nhost: {}, \ncontent_type: {:#?}, \nform: {:#?}, \
        \npost_form: {:#?}, \nmultipart_form: {:#?}"
               , self.method, self.url, self.version, self.header,
               self.body.to_string(), self.content_length, self.close,
               self.host, self.content_type, self.form_param, self.form, self.multipart_form)
    }
}

#[cfg(test)]
mod request_test {
    use crate::http::url::Location;
    use crate::{Request, Method};

    #[test]
    fn request_trans() {
        let request = Request::new(Method::GET, "http://user:password@localhost:7878/path/test/test1/hello/world?key=value&key2=value2#fragid1").unwrap();
        println!("request = {}", request);
    }
}
