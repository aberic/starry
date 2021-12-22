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

use std::fmt;
use std::ops::Add;
use crate::utils::errors::{Errs, StarryResult};

#[derive(Clone, PartialEq, Eq)]
pub struct Status(u16, &'static str);

impl Status {
    // http状态返回代码 1xx（临时响应）
    // 表示临时响应并需要请求者继续执行操作的状态代码。
    /// 100 Continue
    ///
    /// 继续：请求者应当继续提出请求。 服务器返回此代码表示已收到请求的第一部分，正在等待其余部分。
    ///
    /// 参见[[RFC7231, Section 6.2.1](https://tools.ietf.org/html/rfc7231#section-6.2.1)]
    pub const CONTINUE: Status = Status(100, "Continue");
    /// 101 Switching Protocols
    ///
    /// 切换协议：请求者已要求服务器切换协议，服务器已确认并准备切换。
    ///
    /// 参见[[RFC7231, Section 6.2.2](https://tools.ietf.org/html/rfc7231#section-6.2.2)]
    pub const SWITCHING_PROTOCOLS: Status = Status(101, "Switching Protocols");
    /// 102 Processing
    /// [[RFC2518](https://tools.ietf.org/html/rfc2518)]
    pub const PROCESSING: Status = Status(102, "Processing");

    // http状态返回代码 2xx （成功）
    // 表示成功处理了请求的状态代码。
    /// 200 OK
    ///
    /// 成功：服务器已成功处理了请求。 通常，这表示服务器提供了请求的网页。
    ///
    /// 参见[[RFC7231, Section 6.3.1](https://tools.ietf.org/html/rfc7231#section-6.3.1)]
    pub const OK: Status = Status(200, "OK");
    /// 201 Created
    ///
    /// 已创建：请求成功并且服务器创建了新的资源。
    ///
    /// 参见[[RFC7231, Section 6.3.2](https://tools.ietf.org/html/rfc7231#section-6.3.2)]
    pub const CREATED: Status = Status(201, "Created");
    /// 202 Accepted
    ///
    /// 已接受：服务器已接受请求，但尚未处理。
    ///
    /// 参见[[RFC7231, Section 6.3.3](https://tools.ietf.org/html/rfc7231#section-6.3.3)]
    pub const ACCEPTED: Status = Status(202, "Accepted");
    /// 203 Non-Authoritative Information
    ///
    /// 非授权信息：服务器已成功处理了请求，但返回的信息可能来自另一来源。
    ///
    /// 参见[[RFC7231, Section 6.3.4](https://tools.ietf.org/html/rfc7231#section-6.3.4)]
    pub const NON_AUTHORITATIVE_INFORMATION: Status = Status(203, "Non Authoritative Information");
    /// 204 No Content
    ///
    /// 无内容：服务器成功处理了请求，但没有 返回任何内容。
    ///
    /// 参见[[RFC7231, Section 6.3.5](https://tools.ietf.org/html/rfc7231#section-6.3.5)]
    pub const NO_CONTENT: Status = Status(204, "No Content");
    /// 205 Reset Content
    ///
    /// 重置内容：服务器成功处理了请求，但没有返回任何内容。
    ///
    /// 参见[[RFC7231, Section 6.3.6](https://tools.ietf.org/html/rfc7231#section-6.3.6)]
    pub const RESET_CONTENT: Status = Status(205, "Reset Content");
    /// 206 Partial Content
    ///
    /// 部分内容：服务器成功处理了部分 GET 请求。
    ///
    /// 参见[[RFC7233, Section 4.1](https://tools.ietf.org/html/rfc7233#section-4.1)]
    pub const PARTIAL_CONTENT: Status = Status(206, "Partial Content");
    /// 207 Multi-Status
    ///
    /// 多状态：之后的消息体将是一个XML消息，并且可能依照之前子请求数量的不同，包含一系列独立的响应代码
    ///
    /// 参见[[RFC4918](https://tools.ietf.org/html/rfc4918)]
    pub const MULTI_STATUS: Status = Status(207, "Multi-Status");
    /// 208 Already Reported
    ///
    /// 参见[[RFC5842](https://tools.ietf.org/html/rfc5842)]
    pub const ALREADY_REPORTED: Status = Status(208, "Already Reported");
    /// 226 IM Used
    ///
    /// 服务器已经完成了对资源的GET请求，响应是应用于当前实例的一个或多个实例操作的结果的表示。
    /// 实际的当前实例可能不可用，除非根据具体的实例操作，将此响应与其他以前或将来的响应结合使用。
    /// 如果是，则结果实例的头是结合来自status-226响应的头和其他实例的头的结果，遵循HTTP/1.1规范10 [13.5.3]节中的规则。
    ///
    /// 请求必须包含一个A-IM报头字段列出至少一个实例操作。响应必须包含Etag报头字段，给出当前实例的实体标签。
    ///
    /// 一个状态码为226的响应可以被缓存存储并用于回应后续的请求，这取决于HTTP过期机制和任何缓存控制头，以及章节[10.6]的要求。
    ///
    /// 一个状态码为226的响应可以被缓存使用，与基实例的缓存条目一起，为当前实例创建一个缓存条目。
    ///
    /// 参见[[RFC3229](https://tools.ietf.org/html/rfc3229)]
    ///
    /// [13.5.3]: https://datatracker.ietf.org/doc/html/rfc3229#section-13.5.3
    /// [10.6]: https://datatracker.ietf.org/doc/html/rfc3229#section-10.6
    pub const IM_USED: Status = Status(226, "IM Used");

    // http状态返回代码 3xx （重定向）
    // 表示要完成请求，需要进一步操作。 通常，这些状态代码用来重定向。
    /// 300 Multiple Choices
    ///
    /// 多种选择：针对请求，服务器可执行多种操作。 服务器可根据请求者 (user agent) 选择一项操作，或提供操作列表
    /// 供请求者选择。
    ///
    /// 参见[[RFC7231, Section 6.4.1](https://tools.ietf.org/html/rfc7231#section-6.4.1)]
    pub const MULTIPLE_CHOICES: Status = Status(300, "Multiple Choices");
    /// 301 Moved Permanently
    ///
    /// 永久移动：请求的网页已永久移动到新位置。 服务器返回此响应（对 GET 或 HEAD 请求的响应）时，会自动将请求者
    /// 转到新位置。
    ///
    /// 参见[[RFC7231, Section 6.4.2](https://tools.ietf.org/html/rfc7231#section-6.4.2)]
    pub const MOVED_PERMANENTLY: Status = Status(301, "Moved Permanently");
    /// 302 Found
    ///
    /// 临时移动：服务器目前从不同位置的网页响应请求，但请求者应继续使用原有位置来进行以后的请求。
    ///
    /// 参见[[RFC7231, Section 6.4.3](https://tools.ietf.org/html/rfc7231#section-6.4.3)]
    pub const FOUND: Status = Status(302, "Found");
    /// 303 See Other
    ///
    /// 查看其他位置：请求者应当对不同的位置使用单独的 GET 请求来检索响应时，服务器返回此代码。
    ///
    /// 参见[[RFC7231, Section 6.4.4](https://tools.ietf.org/html/rfc7231#section-6.4.4)]
    pub const SEE_OTHER: Status = Status(303, "See Other");
    /// 304 Not Modified
    ///
    /// 未修改：自从上次请求后，请求的网页未修改过。 服务器返回此响应时，不会返回网页内容。
    ///
    /// 参见[[RFC7232, Section 4.1](https://tools.ietf.org/html/rfc7232#section-4.1)]
    pub const NOT_MODIFIED: Status = Status(304, "Not Modified");
    /// 305 Use Proxy
    ///
    /// 使用代理：请求者只能使用代理访问请求的网页。 如果服务器返回此响应，还表示请求者应使用代理。
    ///
    /// 参见[[RFC7231, Section 6.4.5](https://tools.ietf.org/html/rfc7231#section-6.4.5)]
    pub const USE_PROXY: Status = Status(305, "Use Proxy");
    /// 307 Temporary Redirect
    ///
    /// 临时重定向：服务器目前从不同位置的网页响应请求，但请求者应继续使用原有位置来进行以后的请求。
    ///
    /// 参见[[RFC7231, Section 6.4.7](https://tools.ietf.org/html/rfc7231#section-6.4.7)]
    pub const TEMPORARY_REDIRECT: Status = Status(307, "Temporary Redirect");
    /// 308 Permanent Redirect
    ///
    /// 参见[[RFC7238](https://tools.ietf.org/html/rfc7238)]
    pub const PERMANENT_REDIRECT: Status = Status(308, "Permanent Redirect");

    // http状态返回代码 4xx（请求错误）
    // 这些状态代码表示请求可能出错，妨碍了服务器的处理。
    /// 400 Bad Request
    ///
    /// 错误请求：服务器不理解请求的语法。
    ///
    /// 参见[[RFC7231, Section 6.5.1](https://tools.ietf.org/html/rfc7231#section-6.5.1)]
    pub const BAD_REQUEST: Status = Status(400, "Bad Request");
    /// 401 Unauthorized
    ///
    /// 未授权：请求要求身份验证。对于需要登录的网页，服务器可能返回此响应。
    ///
    /// 参见[[RFC7235, Section 3.1](https://tools.ietf.org/html/rfc7235#section-3.1)]
    pub const UNAUTHORIZED: Status = Status(401, "Unauthorized");
    /// 402 Payment Required
    ///
    /// 该状态码是为了将来可能的需求而预留的
    ///
    /// 参见[[RFC7231, Section 6.5.2](https://tools.ietf.org/html/rfc7231#section-6.5.2)]
    pub const PAYMENT_REQUIRED: Status = Status(402, "Payment Required");
    /// 403 Forbidden
    ///
    /// 禁止：服务器拒绝请求。
    ///
    /// 参见[[RFC7231, Section 6.5.3](https://tools.ietf.org/html/rfc7231#section-6.5.3)]
    pub const FORBIDDEN: Status = Status(403, "Forbidden");
    /// 404 Not Found
    ///
    /// 未找到：服务器找不到请求的网页。
    ///
    /// 参见[[RFC7231, Section 6.5.4](https://tools.ietf.org/html/rfc7231#section-6.5.4)]
    pub const NOT_FOUND: Status = Status(404, "Not Found");
    /// 405 Method Not Allowed
    ///
    /// 方法禁用：禁用请求中指定的方法。
    ///
    /// 参见[[RFC7231, Section 6.5.5](https://tools.ietf.org/html/rfc7231#section-6.5.5)]
    pub const METHOD_NOT_ALLOWED: Status = Status(405, "Method Not Allowed");
    /// 406 Not Acceptable
    ///
    /// 不接受：无法使用请求的内容特性响应请求的网页。
    ///
    /// 参见[[RFC7231, Section 6.5.6](https://tools.ietf.org/html/rfc7231#section-6.5.6)]
    pub const NOT_ACCEPTABLE: Status = Status(406, "Not Acceptable");
    /// 407 Proxy Authentication Required
    ///
    /// 需要代理授权：此状态代码与 401（未授权）类似，但指定请求者应当授权使用代理。
    ///
    /// 参见[[RFC7235, Section 3.2](https://tools.ietf.org/html/rfc7235#section-3.2)]
    pub const PROXY_AUTHENTICATION_REQUIRED: Status = Status(407, "Proxy Authentication Required");
    /// 408 Request Timeout
    ///
    /// 请求超时：服务器等候请求时发生超时。
    ///
    /// 参见[[RFC7231, Section 6.5.7](https://tools.ietf.org/html/rfc7231#section-6.5.7)]
    pub const REQUEST_TIMEOUT: Status = Status(408, "Request Timeout");
    /// 409 Conflict
    ///
    /// 冲突：服务器在完成请求时发生冲突。 服务器必须在响应中包含有关冲突的信息。
    ///
    /// 参见[[RFC7231, Section 6.5.8](https://tools.ietf.org/html/rfc7231#section-6.5.8)]
    pub const CONFLICT: Status = Status(409, "Conflict");
    /// 410 Gone
    ///
    /// 已删除：如果请求的资源已永久删除，服务器就会返回此响应。
    ///
    /// 参见[[RFC7231, Section 6.5.9](https://tools.ietf.org/html/rfc7231#section-6.5.9)]
    pub const GONE: Status = Status(410, "Gone");
    /// 411 Length Required
    ///
    /// 需要有效长度：服务器不接受不含有效内容长度标头字段的请求。
    ///
    /// 参见[[RFC7231, Section 6.5.10](https://tools.ietf.org/html/rfc7231#section-6.5.10)]
    pub const LENGTH_REQUIRED: Status = Status(411, "Length Required");
    /// 412 Precondition Failed
    ///
    /// 未满足前提条件：服务器未满足请求者在请求中设置的其中一个前提条件。
    ///
    /// 参见[[RFC7232, Section 4.2](https://tools.ietf.org/html/rfc7232#section-4.2)]
    pub const PRECONDITION_FAILED: Status = Status(412, "Precondition Failed");
    /// 413 Payload Too Large
    ///
    /// 请求实体过大：服务器无法处理请求，因为请求实体过大，超出服务器的处理能力。
    ///
    /// 参见[[RFC7231, Section 6.5.11](https://tools.ietf.org/html/rfc7231#section-6.5.11)]
    pub const PAYLOAD_TOO_LARGE: Status = Status(413, "Payload Too Large");
    /// 414 URI Too Long
    ///
    /// 请求的 URI 过长：请求的 URI（通常为网址）过长，服务器无法处理。
    ///
    /// 参见[[RFC7231, Section 6.5.12](https://tools.ietf.org/html/rfc7231#section-6.5.12)]
    pub const URI_TOO_LONG: Status = Status(414, "URI Too Long");
    /// 415 Unsupported Media Type
    ///
    /// 不支持的媒体类型：请求的格式不受请求页面的支持。
    ///
    /// 参见[[RFC7231, Section 6.5.13](https://tools.ietf.org/html/rfc7231#section-6.5.13)]
    pub const UNSUPPORTED_MEDIA_TYPE: Status = Status(415, "Unsupported Media Type");
    /// 416 Range Not Satisfiable
    ///
    /// 请求范围不符合要求：如果页面无法提供请求的范围，则服务器会返回此状态代码。
    ///
    /// 参见[[RFC7233, Section 4.4](https://tools.ietf.org/html/rfc7233#section-4.4)]
    pub const RANGE_NOT_SATISFIABLE: Status = Status(416, "Range Not Satisfiable");
    /// 417 Expectation Failed
    ///
    /// 未满足期望值：服务器未满足"期望"请求标头字段的要求。
    ///
    /// 参见[[RFC7231, Section 6.5.14](https://tools.ietf.org/html/rfc7231#section-6.5.14)]
    pub const EXPECTATION_FAILED: Status = Status(417, "Expectation Failed");
    /// 418 I'm a teapot
    ///
    /// 参见[curiously not registered by IANA but [RFC2324](https://tools.ietf.org/html/rfc2324)]
    pub const IM_A_TEAPOT: Status = Status(418, "I'm a teapot");
    /// 421 Misdirected Request
    ///
    /// 参见[RFC7540, Section 9.1.2](http://tools.ietf.org/html/rfc7540#section-9.1.2)
    pub const MISDIRECTED_REQUEST: Status = Status(421, "Misdirected Request");
    /// 422 Unprocessable Entity
    ///
    /// 参见[[RFC4918](https://tools.ietf.org/html/rfc4918)]
    pub const UNPROCESSABLE_ENTITY: Status = Status(422, "Unprocessable Entity");
    /// 423 Locked
    ///
    /// 参见[[RFC4918](https://tools.ietf.org/html/rfc4918)]
    pub const LOCKED: Status = Status(423, "Locked");
    /// 424 Failed Dependency
    ///
    /// 参见[[RFC4918](https://tools.ietf.org/html/rfc4918)]
    pub const FAILED_DEPENDENCY: Status = Status(424, "Failed Dependency");
    /// 426 Upgrade Required
    ///
    /// 参见[[RFC7231, Section 6.5.15](https://tools.ietf.org/html/rfc7231#section-6.5.15)]
    pub const UPGRADE_REQUIRED: Status = Status(426, "Upgrade Required");
    /// 428 Precondition Required
    ///
    /// 参见[[RFC6585](https://tools.ietf.org/html/rfc6585)]
    pub const PRECONDITION_REQUIRED: Status = Status(428, "Precondition Required");
    /// 429 Too Many Requests
    ///
    /// 参见[[RFC6585](https://tools.ietf.org/html/rfc6585)]
    pub const TOO_MANY_REQUESTS: Status = Status(429, "Too Many Requests");
    /// 431 Request Header Fields Too Large
    ///
    /// 参见[[RFC6585](https://tools.ietf.org/html/rfc6585)]
    pub const REQUEST_HEADER_FIELDS_TOO_LARGE: Status = Status(431, "Request Header Fields Too Large");
    /// 451 Unavailable For Legal Reasons
    ///
    /// 参见[[RFC7725](http://tools.ietf.org/html/rfc7725)]
    pub const UNAVAILABLE_FOR_LEGAL_REASONS: Status = Status(451, "Unavailable For Legal Reasons");
    // http状态返回代码 5xx（服务器错误）
    // 这些状态代码表示服务器在尝试处理请求时发生内部错误。 这些错误可能是服务器本身的错误，而不是请求出错。
    /// 500 Internal Server Error
    ///
    /// 服务器内部错误：服务器遇到错误，无法完成请求。
    ///
    /// 参见[[RFC7231, Section 6.6.1](https://tools.ietf.org/html/rfc7231#section-6.6.1)]
    pub const INTERNAL_SERVER_ERROR: Status = Status(500, "Internal Server Error");
    /// 501 Not Implemented
    ///
    /// 尚未实施：服务器不具备完成请求的功能。 例如，服务器无法识别请求方法时可能会返回此代码。
    ///
    /// 参见[[RFC7231, Section 6.6.2](https://tools.ietf.org/html/rfc7231#section-6.6.2)]
    pub const NOT_IMPLEMENTED: Status = Status(501, "Not Implemented");
    /// 502 Bad Gateway
    ///
    /// 错误网关：服务器作为网关或代理，从上游服务器收到无效响应。
    ///
    /// 参见[[RFC7231, Section 6.6.3](https://tools.ietf.org/html/rfc7231#section-6.6.3)]
    pub const BAD_GATEWAY: Status = Status(502, "Bad Gateway");
    /// 503 Service Unavailable
    ///
    /// 服务不可用：服务器目前无法使用（由于超载或停机维护）。 通常，这只是暂时状态。
    ///
    /// 参见[[RFC7231, Section 6.6.4](https://tools.ietf.org/html/rfc7231#section-6.6.4)]
    pub const SERVICE_UNAVAILABLE: Status = Status(503, "Service Unavailable");
    /// 504 Gateway Timeout
    ///
    /// 网关超时：服务器作为网关或代理，但是没有及时从上游服务器收到请求。
    ///
    /// 参见[[RFC7231, Section 6.6.5](https://tools.ietf.org/html/rfc7231#section-6.6.5)]
    pub const GATEWAY_TIMEOUT: Status = Status(504, "Gateway Timeout");
    /// 505 HTTP Version Not Supported
    ///
    /// HTTP 版本不受支持：服务器不支持请求中所用的 HTTP 协议版本。
    ///
    /// 参见[[RFC7231, Section 6.6.6](https://tools.ietf.org/html/rfc7231#section-6.6.6)]
    pub const HTTP_VERSION_NOT_SUPPORTED: Status = Status(505, "HTTP Version Not Supported");
    /// 506 Variant Also Negotiates
    ///
    /// 参见[[RFC2295](https://tools.ietf.org/html/rfc2295)]
    pub const VARIANT_ALSO_NEGOTIATES: Status = Status(506, "Variant Also Negotiates");
    /// 507 Insufficient Storage
    ///
    /// 参见[[RFC4918](https://tools.ietf.org/html/rfc4918)]
    pub const INSUFFICIENT_STORAGE: Status = Status(507, "Insufficient Storage");
    /// 508 Loop Detected
    ///
    /// 参见[[RFC5842](https://tools.ietf.org/html/rfc5842)]
    pub const LOOP_DETECTED: Status = Status(508, "Loop Detected");

    /// 510 Not Extended
    ///
    /// 参见[[RFC2774](https://tools.ietf.org/html/rfc2774)]
    pub const NOT_EXTENDED: Status = Status(510, "Not Extended");
    /// 511 Network Authentication Required
    ///
    /// 参见[[RFC6585](https://tools.ietf.org/html/rfc6585)]
    pub const NETWORK_AUTHENTICATION_REQUIRED: Status = Status(511, "Network Authentication Required");

    pub fn from_code(code: u16) -> StarryResult<Status> {
        match code {
            100 => Ok(Status::CONTINUE),
            101 => Ok(Status::SWITCHING_PROTOCOLS),
            102 => Ok(Status::PROCESSING),
            200 => Ok(Status::OK),
            201 => Ok(Status::CREATED),
            202 => Ok(Status::ACCEPTED),
            203 => Ok(Status::NON_AUTHORITATIVE_INFORMATION),
            204 => Ok(Status::NO_CONTENT),
            205 => Ok(Status::RESET_CONTENT),
            206 => Ok(Status::PARTIAL_CONTENT),
            207 => Ok(Status::MULTI_STATUS),
            208 => Ok(Status::ALREADY_REPORTED),
            226 => Ok(Status::IM_USED),
            300 => Ok(Status::MULTIPLE_CHOICES),
            301 => Ok(Status::MOVED_PERMANENTLY),
            302 => Ok(Status::FOUND),
            303 => Ok(Status::SEE_OTHER),
            304 => Ok(Status::NOT_MODIFIED),
            305 => Ok(Status::USE_PROXY),
            307 => Ok(Status::TEMPORARY_REDIRECT),
            308 => Ok(Status::PERMANENT_REDIRECT),
            400 => Ok(Status::BAD_REQUEST),
            401 => Ok(Status::UNAUTHORIZED),
            402 => Ok(Status::PAYMENT_REQUIRED),
            403 => Ok(Status::FORBIDDEN),
            404 => Ok(Status::NOT_FOUND),
            405 => Ok(Status::METHOD_NOT_ALLOWED),
            406 => Ok(Status::NOT_ACCEPTABLE),
            407 => Ok(Status::PROXY_AUTHENTICATION_REQUIRED),
            408 => Ok(Status::REQUEST_TIMEOUT),
            409 => Ok(Status::CONFLICT),
            410 => Ok(Status::GONE),
            411 => Ok(Status::LENGTH_REQUIRED),
            412 => Ok(Status::PRECONDITION_FAILED),
            413 => Ok(Status::PAYLOAD_TOO_LARGE),
            414 => Ok(Status::URI_TOO_LONG),
            415 => Ok(Status::UNSUPPORTED_MEDIA_TYPE),
            416 => Ok(Status::RANGE_NOT_SATISFIABLE),
            417 => Ok(Status::EXPECTATION_FAILED),
            418 => Ok(Status::IM_A_TEAPOT),
            421 => Ok(Status::MISDIRECTED_REQUEST),
            422 => Ok(Status::UNPROCESSABLE_ENTITY),
            423 => Ok(Status::LOCKED),
            424 => Ok(Status::FAILED_DEPENDENCY),
            426 => Ok(Status::UPGRADE_REQUIRED),
            428 => Ok(Status::PRECONDITION_REQUIRED),
            429 => Ok(Status::TOO_MANY_REQUESTS),
            431 => Ok(Status::REQUEST_HEADER_FIELDS_TOO_LARGE),
            451 => Ok(Status::UNAVAILABLE_FOR_LEGAL_REASONS),
            500 => Ok(Status::INTERNAL_SERVER_ERROR),
            501 => Ok(Status::NOT_IMPLEMENTED),
            502 => Ok(Status::BAD_GATEWAY),
            503 => Ok(Status::SERVICE_UNAVAILABLE),
            504 => Ok(Status::GATEWAY_TIMEOUT),
            505 => Ok(Status::HTTP_VERSION_NOT_SUPPORTED),
            506 => Ok(Status::VARIANT_ALSO_NEGOTIATES),
            507 => Ok(Status::INSUFFICIENT_STORAGE),
            508 => Ok(Status::LOOP_DETECTED),
            510 => Ok(Status::NOT_EXTENDED),
            511 => Ok(Status::NETWORK_AUTHENTICATION_REQUIRED),
            _ => Err(Errs::string(format!("un support http status code {}!", code)))
        }
    }

    /// 用`&str`表示当前Status
    pub fn phrase(&self) -> &str {
        self.1
    }

    pub fn phrase_as_slice(&self) -> &[u8] {
        self.1.as_bytes()
    }

    pub fn code(&self) -> u16 {
        self.0
    }
}

impl<'a> PartialEq<&'a Status> for Status {
    fn eq(&self, other: &&'a Status) -> bool {
        self == *other
    }
}

impl<'a> PartialEq<Status> for &'a Status {
    fn eq(&self, other: &Status) -> bool {
        *self == other
    }
}

impl PartialEq<str> for Status {
    fn eq(&self, other: &str) -> bool {
        self.phrase() == other
    }
}

impl PartialEq<Status> for str {
    fn eq(&self, other: &Status) -> bool {
        self == other.phrase()
    }
}

impl<'a> PartialEq<&'a str> for Status {
    fn eq(&self, other: &&'a str) -> bool {
        self.phrase() == *other
    }
}

impl<'a> PartialEq<Status> for &'a str {
    fn eq(&self, other: &Status) -> bool {
        *self == other.phrase()
    }
}

impl<'a> PartialEq<u16> for Status {
    fn eq(&self, other: &u16) -> bool {
        self.0 == *other
    }
}

impl<'a> PartialEq<Status> for u16 {
    fn eq(&self, other: &Status) -> bool {
        *self == other.code()
    }
}

impl fmt::Debug for Status {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&*self.code().to_string().add(" ").to_string().add(self.phrase()))
    }
}

#[test]
fn status_to_str() {
    let s = Status::ALREADY_REPORTED;
    assert_eq!(s.code(), 208);
    assert_eq!(s.phrase(), "Already Reported");
}

#[test]
fn status_eq() {
    assert_eq!(Status::OK, Status::OK);
    assert_eq!(Status::OK, "OK");
    assert_eq!(&Status::OK, "OK");
    assert_eq!(Status::OK, 200);
    assert_eq!("OK", Status::OK);
    assert_eq!("OK", &Status::OK);
    assert_eq!(200, Status::OK);
}

#[test]
fn status_from() {
    assert!(Status::from_code(200).is_ok());
    assert!(Status::from_code(800).is_err());
    assert_eq!(Status::from_code(200).unwrap(), 200)
}
