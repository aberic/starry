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

use std::borrow::{Borrow, BorrowMut};
use std::collections::HashMap;
use std::fmt::{Display, Formatter};
use std::hash::Hash;
use std::io::{Read, Write};
use std::net::TcpStream;
use std::slice::Iter;
use std::sync::{Arc, RwLock};

use bytes::{BufMut, Bytes, BytesMut};
use bytes::buf::Writer;

use crate::{ContentType, Cookie, Header, Method, Response, URL, Values, Version};
use crate::http::body::Body;
use crate::http::content_type;
use crate::http::url::{Authority, Location, Scheme};
use crate::http::url::authority::Addr;
use crate::http::values::MultipartValues;
use crate::http::version::Protocol;
use crate::server::node::{Node, Root};
use crate::utils::errors::{Error, Errs, StarryResult};

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
#[derive(Debug)]
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
    form: Values,
    /// MultipartValues 是一个已解析的多部分表单。
    /// 它的File部分存储在内存或磁盘上，内置相关方法访问。
    /// 它的Value部分被存储为字符串。两者都是通过字段名进行键控的。
    multipart_form: MultipartValues,
    pub(crate) cookies: Vec<Cookie>,
    pub(crate) client: Addr,
    pub(crate) stream: TcpStream,
    /// body数据是否已经被解析，如被解析，则无法再次解析
    body_parse: bool,
}

impl Request {
    pub fn body(&mut self) -> Bytes {
        self.body.body()
    }
}

impl Request {
    pub(crate) fn from(stream: TcpStream, root: Arc<RwLock<Root>>) -> StarryResult<(Request, Node, HashMap<String, String>)> {
        let mut req = Request {
            method: Method::GET,
            url: URL::default(),
            version: Default::default(),
            header: Header::new(),
            body: Default::default(),
            content_length: -1,
            close: false,
            host: String::new(),
            content_type: None,
            form_param: Values::new(),
            form: Values::new(),
            multipart_form: MultipartValues::new(),
            cookies: vec![],
            client: Default::default(),
            stream,
            body_parse: false,
        };
        let (node, fields) = req.parse(root)?;
        Ok((req, node, fields))
    }

    pub fn try_clone(&self) -> StarryResult<Request> {
        match self.stream.try_clone() {
            Ok(stream) => Ok(Self {
                method: self.method.clone(),
                url: self.url.clone(),
                version: self.version.clone(),
                header: self.header.clone(),
                body: self.body.clone(),
                content_length: self.content_length.clone(),
                close: self.close.clone(),
                host: self.host.clone(),
                content_type: self.content_type.clone(),
                form_param: self.form_param.clone(),
                form: self.form.clone(),
                multipart_form: self.multipart_form.clone(),
                cookies: self.cookies.clone(),
                client: self.client.clone(),
                stream,
                body_parse: self.body_parse,
            }),
            Err(err) => Err(Errs::strs("stream try clone failed while in request!", err))
        }
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

    pub fn form(&mut self) -> StarryResult<Values> {
        if !self.body_parse {
            self.parse_body()?;
        }
        Ok(self.form.clone())
    }

    pub fn multipart_form(&mut self) -> StarryResult<MultipartValues> {
        if !self.body_parse {
            self.parse_body()?;
        }
        Ok(self.multipart_form.clone())
    }

    pub fn client(&self) -> Addr {
        self.client.clone()
    }

    pub(crate) fn response(&mut self, mut response: Response) -> StarryResult<()> {
        log::debug!("response: {:#?}", response);

        // 状态行
        self.write(response.version.as_slice())?;
        self.write(b" ")?;
        self.write(response.status.code().to_string().as_bytes())?;
        self.write(b" ")?;
        self.write(response.status.phrase_as_slice())?;
        self.write(b"\r\n")?;
        // 头部块
        for (key, values) in response.header.map() {
            for value in values {
                self.write(key.as_bytes())?;
                self.write(b": ")?;
                self.write(value.as_bytes())?;
                self.write(b"\r\n")?;
            }
        }
        self.write(b"\r\n")?;
        // 数据块
        self.write(response.get_write_content().as_ref())?;

        match self.stream.flush() {
            Ok(()) => Ok(()),
            Err(err) => Err(Errs::strs("stream flush failed!", err)),
        }
    }

    fn write(&mut self, buf: &[u8]) -> StarryResult<usize> {
        match self.stream.write(buf) {
            Ok(src) => Ok(src),
            Err(err) => Err(Errs::strs("stream write failed!", err)),
        }
    }

    fn write_bytes(&mut self, bw: &mut Writer<BytesMut>, src: &[u8]) -> StarryResult<usize> {
        match bw.write(src) {
            Ok(src) => Ok(src),
            Err(err) => return Err(self.interrupt(
                Response::bad_request(),
                Errs::strs("write data into body failed!", err)))
        }
    }

    /// 中断操作，返回报错和对应应答
    fn interrupt(&mut self, response: Response, error: Error) -> Error {
        match self.response(response) {
            Ok(()) => error,
            Err(err) => {
                log::error!("http response interrupt failed! error is {}", err);
                error
            }
        }
    }

    /// 解析请求行信息
    fn parse(&mut self, root: Arc<RwLock<Root>>) -> StarryResult<(Node, HashMap<String, String>)> {
        let mut buffer = [0; 1024];
        let mut iter;
        // 剩余待读取数据的总长度
        let mut size;
        match self.stream.read(&mut buffer) {
            Ok(src) => {
                log::trace!("stream read size = {}", src);
                if src == 0 { // 没有数据进入
                    return Err(Errs::str("tcp stream had no data!"))
                }
                size = src;
                iter = buffer.iter()
            }
            Err(err) => return Err(self.interrupt(
                Response::bad_request(),
                Errs::strs("read failed while parse request with error!", err)))
        }

        // 解析请求行信息 POST /path/data?key=value&key2=value2 HTTP/1.1
        let (location, count) = self.parse_request_line(iter.borrow_mut())?;
        // log::trace!("parse_request_line count = {}, method = {}, version = {}", count, self.method, self.version.to_string());
        size -= count;

        let node;
        let fields;
        // println!("path = {:#?}, method = {:#?}", location.path(), self.method());
        // 根据资源信息获取请求方法，判断请求有效性，如无效，则放弃后续解析操作
        match root.read().unwrap().fetch(location.path(), self.method()) {
            Some((src1, src2)) => {
                node = src1;
                fields = src2;
            }
            None => return Err(self.interrupt(
                Response::not_found(),
                Errs::str("server fetch node do not exist!")))
        }

        // 解析消息报头
        let count = self.parse_request_header(iter.borrow_mut());
        size -= count;

        // 根据已知结果解析请求关联参数
        self.parse_others(location)?;

        match self.header.read_cookies() {
            Ok(src) => self.cookies = src,
            Err(err) => return Err(self.interrupt(
                Response::bad_request(),
                Errs::strs("read cookies from header failed!", err)))
        }

        // 解析请求正文
        // 当请求方法为 POST/PUT/PATCH 时需要解析body，其它方法没有实体，即便有，也会被丢弃掉
        match self.method {
            Method::PATCH | Method::PUT | Method::POST => {
                iter.next(); // 请求，过滤换行符
                size -= 1;
                match self.header.get_content_length() {
                    Some(content_len) => match content_len.parse::<isize>() {
                        Ok(len) => {
                            if len > 0 { // 读取到"Content-Length"大于0
                                // Body数据存在，新建写流，将body数据进行填充
                                let mut bw = BytesMut::new().writer();
                                self.write_bytes(bw.borrow_mut(), &iter.as_ref()[0..size])?;
                                // bw.write(&iter.as_ref()[0..size]);
                                let mut count = size;
                                loop { // 将stream中数据全数读出到 self.body
                                    if count as isize == len {
                                        break;
                                    }
                                    size = self.reread_stream(&mut buffer)?;
                                    // bw.write(&buffer[0..size]);
                                    self.write_bytes(bw.borrow_mut(), &buffer[0..size])?;
                                    count += size
                                }
                                self.body.init_reader(bw.into_inner());
                            } else if len == 0 {  // 读取到"Content-Length"等于0
                                return Ok((node, fields));
                            } else if len == -1 {  // 读取到"Content-Length"小于0
                                // Body数据存在，长度未知，新建写流，将body数据进行填充
                                let mut bw = BytesMut::new().writer();
                                // bw.write(&iter.as_ref()[0..size]);
                                self.write_bytes(bw.borrow_mut(), &iter.as_ref()[0..size])?;
                                let mut buf_all = Vec::new();
                                match self.stream.read_to_end(&mut buf_all) {
                                    Ok(_) => {
                                        // bw.write(buf_all.as_slice());
                                        self.write_bytes(bw.borrow_mut(), buf_all.as_slice())?;
                                        self.body.init_reader(bw.into_inner());
                                    }
                                    Err(err) => return Err(self.interrupt(
                                        Response::length_required(),
                                        Errs::strs("parse request body while read_to_end failed!", err)))
                                }
                            } else {
                                return Err(self.interrupt(
                                    Response::length_required(),
                                    Errs::string(format!("content len {} invalid from header failed!", len))));
                            }
                            self.content_length = len;
                        }
                        Err(err) => return Err(self.interrupt(
                            Response::length_required(),
                            Errs::strings(format!("content len {} parse usize from header failed!", content_len), err)))
                    },
                    None => {
                        self.content_length = 0;
                        return Ok((node, fields));
                    }
                }
            }
            _ => return Ok((node, fields))
        }

        // 如果存在body，则需要根据content-type对body数据进行解析使用
        log::trace!("body = {}", self.body.to_string());
        self.parse_body()?;
        Ok((node, fields))
    }

    fn addr(&mut self) -> StarryResult<Addr> {
        match self.stream.peer_addr() {
            Ok(addr) => self.client = Addr::from(addr.ip().to_string(), addr.port()),
            Err(err) => return Err(Errs::err(err))
        }
        match self.stream.local_addr() {
            Ok(addr) => Ok(Addr::from(addr.ip().to_string(), addr.port())),
            Err(err) => Err(Errs::err(err))
        }
    }

    fn reread_stream(&mut self, buf: &mut [u8]) -> StarryResult<usize> {
        match self.stream.read(buf) {
            Ok(src) => {
                Ok(src)
            }
            Err(err) => return Err(self.interrupt(
                Response::bad_request(),
                Errs::strs("read failed while parse request with failed!", err)))
        }
    }

    /// 解析请求行信息 POST /path/data?key=value&key2=value2 HTTP/1.1
    ///
    /// usize 读取数据的总长度
    fn parse_request_line(&mut self, iter: &mut Iter<u8>) -> StarryResult<(Location, usize)> {
        // 读取数据的总长度
        let mut count = 0;
        let mut step: u8 = 1;
        let mut location: Location = Location::new();
        let mut data: Vec<u8> = vec![];
        loop {
            match iter.next() {
                Some(b) => {
                    count += 1;
                    match b {
                        b' ' => match step { // POST/GET/...
                            1 => {
                                match Method::from_bytes(data.as_slice()) {
                                    Ok(src) => {
                                        self.method = src;
                                        data.clear();
                                        step = 2
                                    }
                                    Err(err) => return Err(self.interrupt(
                                        Response::method_not_allowed(),
                                        Errs::strs("parse request failed!", err)))
                                }
                            }
                            2 => { // /path/data?key=value&key2=value2
                                data.push(b'\r');
                                location = Location::from_bytes(data.clone());
                                for (key, value) in location.query().map() {
                                    self.form_param.set(key, value[0].clone());
                                }
                                data.clear();
                                step = 3
                            }
                            _ => return Err(self.interrupt(
                                Response::bad_request(),
                                Errs::str("parse request failed, does not understand the syntax of the request while step2!")))
                        }
                        b'\r' => match step {
                            3 => match Version::from_bytes(data.as_slice()) { // HTTP/1.1...
                                Ok(src) => {
                                    self.version = src;
                                    break;
                                }
                                Err(err) => return Err(self.interrupt(
                                    Response::http_version_not_supported(),
                                    Errs::strs("parse request failed!", err)))
                            }
                            _ => return Err(self.interrupt(
                                Response::bad_request(),
                                Errs::str("parse request failed, does not understand the syntax of the request while step3!")))
                        }
                        _ => data.push(*b),
                    }
                }
                None => break
            }
        }
        Ok((location, count))
    }

    /// 解析消息报头
    ///
    /// usize 读取数据的总长度
    fn parse_request_header(&mut self, iter: &mut Iter<u8>) -> usize {
        let mut count = 0;
        let mut data: Vec<u8> = vec![];
        let mut key: String = String::from("");
        // 是否轮到key解析
        let mut key_time = true;
        // 即将开始value解析
        let mut pre_value_time = false;
        // 是否轮到value解析
        let mut value_time = false;
        // 是否结束解析。当连续出现两次"\n\n"后结束解析
        let mut end_time = false;
        loop {
            match iter.next() {
                Some(b) => {
                    count += 1;
                    match b {
                        b':' => if key_time {
                            key = String::from_utf8_lossy(&data).to_string();
                            key_time = false;
                            pre_value_time = true;
                            data.clear()
                        } else {
                            data.push(*b)
                        }
                        b' ' => if key_time {
                            data.push(*b)
                        } else if pre_value_time {
                            end_time = false;
                            pre_value_time = false;
                            value_time = true
                        } else if value_time {
                            data.push(*b)
                        },
                        b'\n' => {}
                        b'\r' => if end_time {
                            break;
                        } else if value_time {
                            self.header.set(key.clone(), String::from_utf8_lossy(&data).to_string());
                            key.clear();
                            data.clear();
                            key_time = true;
                            value_time = false;
                            end_time = true
                        }
                        _ => data.push(*b),
                    }
                }
                None => break
            }
        }
        count
    }

    /// 根据已知结果解析请求关联参数
    fn parse_others(&mut self, location: Location) -> StarryResult<()> {
        let userinfo;
        match self.header.get_userinfo() {
            Ok(src) => userinfo = src,
            Err(err) => return Err(self.interrupt(
                Response::bad_request(),
                Errs::strs("parse request failed, userinfo parse error!", err)))
        }
        match self.addr() {
            Ok(src) => self.url = URL::from(Scheme::HTTP, Authority::new(userinfo, src), location),
            Err(err) => return Err(self.interrupt(
                Response::bad_request(),
                Errs::strs("parse request failed, addr parse error!", err)))
        }
        self.close = self.header.check_close(&self.version, false);
        match self.version {
            Version::HTTP_10 | Version::HTTP_11 => match self.header.get_host() {
                Some(src) => self.host = src.to_string(),
                None => return Err(self.interrupt(
                    Response::expectation_failed(),
                    Errs::str("host not found in header!")))
            }
            // Version::HTTP_2 => match self.header.get_host() {
            //     Some(src) => self.host = src.to_string(),
            //     None => self.host = self.url.authority().addr().to_string()
            // }
            _ => return Err(self.interrupt(
                Response::http_version_not_supported(),
                Errs::str("parse request failed, version not support!")))
        }
        match self.header.get_content_type() {
            Some(src) => self.content_type = Some(ContentType::from_str(&src)),
            None => {}
        }
        Ok(())
    }

    /// 如果存在body，则需要根据content-type对body数据进行解析使用
    fn parse_body(&mut self) -> StarryResult<()> {
        self.body_parse = true;
        let body = self.body().to_vec();
        println!("body len = {}", self.body.len());
        let content_type;
        match self.content_type.clone() {
            Some(src) => content_type = src,
            None => return Ok(())
        }
        match content_type.inner() {
            content_type::Inner::ApplicationXWWWFormUrlEncoded => { // 11=22&44=55 / 11=22&44=55&77=&=222
                let mut step: u8 = 1; // 1:key -> 2:value
                let mut data: Vec<u8> = vec![];
                let mut key: String = String::from("");
                for b in body {
                    match b {
                        b'=' => {
                            if step == 1 {
                                key = String::from_utf8_lossy(&data).to_string();
                                step = 2;
                                data.clear()
                            } else {
                                data.push(b)
                            }
                        }
                        b'&' => {
                            if step == 1 {
                                data.push(b)
                            } else {
                                let value = String::from_utf8_lossy(&data).to_string();
                                // self.form.set(key.clone(), value.clone());
                                self.form.set(key.clone(), value);
                                step = 1;
                                data.clear()
                            }
                        }
                        _ => data.push(b)
                    }
                }
                let value = String::from_utf8_lossy(&data).to_string();
                if !key.is_empty() || !value.is_empty() {
                    // self.form.set(key.clone(), value.clone());
                    self.form.set(key.clone(), value);
                }
            }
            content_type::Inner::MultipartFormData(src) => {
                let start = format!("--{}", src);
                let end = format!("--{}--", src);
                // println!("start = {}", start);
                // println!("end   = {}", end);
                // println!();
                let mut name = String::new();
                let mut filename = String::new();
                let mut tmp = vec![];
                let mut content = vec![];
                // form-data中每个流文件都有单独的类型
                let mut content_type = ContentType::default();
                // 1:start -> 2:Content-Disposition: form-data; name="1" -> 3:"" -> 4:value
                // 1:start -> 2:Content-Disposition: form-data; name="1"; filename="file.txt" -> 3:"" -> 5:file
                let mut step: u8 = 1;
                for b in body {
                    match b {
                        b'\n' => { // 发现换行符启动验证
                            let src = String::from_utf8_lossy(tmp.as_slice()).to_string();
                            match step {
                                1 => if src.eq(start.as_str()) {
                                    step = 2;
                                } else if src.eq(end.as_str()) {
                                    break;
                                } else {
                                    return Err(self.interrupt(
                                        Response::bad_request(),
                                        Errs::str("boundary can not matched!")));
                                },
                                2 => {
                                    let mut vs = src.split(";");
                                    match vs.next() {
                                        Some(res) => if res.ne("Content-Disposition: form-data") {
                                            return Err(self.interrupt(
                                                Response::bad_request(),
                                                Errs::str("Content-Disposition: form-data can not matched!")));
                                        }
                                        None => {
                                            return Err(self.interrupt(
                                                Response::bad_request(),
                                                Errs::str("multipart/form-data body disposition parse failed!")));
                                        }
                                    }
                                    match vs.next() { // name
                                        Some(src) => {
                                            let res = src.trim();
                                            let len = res.len() - 1;
                                            name = res[6..len].to_string();
                                        }
                                        None => return Err(self.interrupt(
                                            Response::bad_request(),
                                            Errs::str("multipart/form-data body name parse failed!")))
                                    }
                                    match vs.next() { // filename
                                        Some(src) => {
                                            let res = src.trim();
                                            let len = res.len() - 1;
                                            filename = res[10..len].to_string();
                                            step = 5
                                        }
                                        None => step = 3
                                    }
                                }
                                3 => step = 4,
                                4 => if src.eq(start.as_str()) || src.eq(end.as_str()) {
                                    content = content[..content.len() - 1].to_owned();
                                    let value = String::from_utf8_lossy(content.as_slice()).to_string();
                                    // self.form.set(name.clone(), value.clone());
                                    self.form.set(name.clone(), value);
                                    content.clear();
                                    step = 2;
                                } else {
                                    tmp.push(b);
                                    content.append(&mut tmp)
                                }
                                5 => {
                                    content_type = ContentType::from_str(&src[14..]);
                                    step = 6
                                    // if src.eq("Content-Type: application/octet-stream") {
                                    //     step = 6
                                    // } else {
                                    //     self.response(Status::BAD_REQUEST);
                                    //     return Err(Errs::str("multipart/form-data body file type parse failed!"));
                                    // }
                                }
                                6 => {
                                    step = 7
                                }
                                7 => if src.eq(start.as_str()) || src.eq(end.as_str()) {
                                    content = content[..content.len() - 1].to_owned();
                                    self.multipart_form.insert_obj(name.clone(),
                                                                   filename.clone(),
                                                                   content.clone(),
                                                                   content_type.clone());
                                    content.clear();
                                    step = 2
                                } else {
                                    tmp.push(b);
                                    content.append(&mut tmp)
                                }
                                _ => {}
                            }
                            tmp.clear()
                        }
                        b'\r' => {} // 回车符什么也不做
                        _ => tmp.push(b)
                    }
                }
            }
            _ => {}
        }
        // for (filename, file_header) in self.multipart_form.file_map() {
        //     println!("file name = {}", filename);
        //     println!("file content = \n{}", String::from_utf8_lossy(file_header.content().as_slice()));
        // }
        Ok(())
    }
}

impl Display for Request {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "\nmethod: {:#?}, \nurl: {:#?}, \nversion: {:#?}, \nheader: {:#?}, \nbody: {}, \
        \ncontent_length: {}, \nclose: {}, \nhost: {}, \ncontent_type: {:#?}, \nform: {:#?}, \
        \npost_form: {:#?}, \nmultipart_form: {:#?}, \nstream: {:#?}\n"
               , self.method, self.url, self.version, self.header,
               self.body.to_string(), self.content_length, self.close,
               self.host, self.content_type, self.form_param, self.form, self.multipart_form,
               self.stream)
    }
}
