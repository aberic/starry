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
use std::fmt::{Display, Formatter};

use crate::utils::errors::{Errs, StarryResult};
use crate::http::method::Inner::*;

/// Method 指定HTTP协议中定义的方法（如GET、POST、PUT等）。
/// 对于客户端请求，空字符串表示GET。
#[derive(Clone, PartialEq, Eq, Hash)]
pub struct Method(Inner);

#[derive(Clone, PartialEq, Eq, Hash)]
enum Inner {
    /// HTTP/1.1 第一版定义，参考[RFC 2068](https://datatracker.ietf.org/doc/html/rfc2068#section-9.2)
    /// HTTP/1.1 第二版更新，参考[RFC 2616](https://datatracker.ietf.org/doc/html/rfc2616#section-9.2)
    ///
    /// OPTIONS方法表示对Request-URI标识的请求/响应链上可用通信选项的信息的请求。该方法允许客户端确定与资源或服务
    /// 器功能相关的选项和/或需求，而无需暗示资源操作或启动资源检索。此方法的响应是不可缓存的。如果OPTIONS请求包含
    /// 实体体(由Content-Length或Transfer-Encoding表示)，则媒体类型必须由Content-Type字段表示。尽管该规范没
    /// 有定义此类主体的任何用途，但HTTP未来的扩展可能会使用OPTIONS主体在服务器上进行更详细的查询。不支持这种扩展
    /// 的服务器可能会丢弃请求体。
    ///
    /// 如果Request-URI是星号("\*")，则OPTIONS请求通常应用于服务器，而不是特定的资源。由于服务器的通信选项通常依
    /// 赖于资源，"\*"请求仅用于“ping”或“no-op”类型的方法；它只允许客户端测试服务器的功能。例如，这可以用来测试一
    /// 个代理是否符合HTTP/1.1(或是否不符合)。
    ///
    /// 如果request - uri不是星号，OPTIONS请求仅应用于与该资源通信时可用的选项。
    ///
    /// 一个200响应应该包括任何报头字段，这些字段表示服务器实现的可选特性并适用于该资源(例如，Allow)，可能包括本规
    /// 范未定义的扩展。响应体(如果有的话)还应该包括关于通信选项的信息。这种格式本规范没有定义body，但可能由HTTP的
    /// 未来扩展定义。内容协商可用于选择适当的响应格式。如果没有包含响应体，响应必须包含一个Content-Length字段，字
    /// 段值为“0”。
    ///
    /// Max-Forwards请求头字段可以用于针对请求链中的特定代理。当一个代理在一个绝对uri上接收到一个允许请求转发的
    /// OPTIONS请求时，代理必须检查Max-Forwards字段。如果Max-Forwards字段值为零("0")，代理必须不转发消息；相
    /// 反，代理应该用它自己的通信选项来响应。如果Max-Forwards字段值是一个大于零的整数，代理在转发请求时必须减少字
    /// 段值。如果请求中没有Max-Forwards字段，则转发的请求中必须不包含Max-Forwards字段。
    Options,
    /// HTTP/1.0 定义，参考[RFC 1945](https://datatracker.ietf.org/doc/html/rfc1945#section-8.1)
    /// HTTP/1.1 第二版更新，参考[RFC 2616](https://datatracker.ietf.org/doc/html/rfc2616#section-9.3)
    ///
    /// GET方法意味着检索由Request-URI标识的任何信息(以实体的形式)。如果Request-URI指的是一个数据生成过程，那么
    /// 它是生成的数据，应该作为响应中的实体返回，而不是该过程的源文本，除非该文本恰好是该过程的输出。
    ///
    /// 如果请求消息包含If-Modified-Since, If-Unmodified-Since, If-Match, If-None-Match, 或If-Range报头
    /// 字段，则GET方法的语义变为“条件GET”。条件GET方法请求只在条件报头字段所描述的情况下传输实体。有条件GET方法的
    /// 目的是通过允许在不需要多个请求或传输客户端已经持有的数据的情况下刷新缓存的实体来减少不必要的网络使用。
    ///
    /// 如果请求消息包含Range报头字段，那么GET方法的语义将变为“partial GET”。部分GET请求只传输实体的一部分，如
    /// [协议](https://datatracker.ietf.org/doc/html/rfc2616#section-14.35) 所述。部分GET方法的目的是通
    /// 过允许在不传输客户端已经持有的数据的情况下完成部分检索的实体来减少不必要的网络使用。
    ///
    /// 当且仅当GET请求的响应满足[协议](https://datatracker.ietf.org/doc/html/rfc2616#section-13) 中描述
    /// 的HTTP缓存要求时，该响应是可缓存的。
    ///
    /// 参阅[协议](https://datatracker.ietf.org/doc/html/rfc2616#section-15.1.3) 了解用于表单时的安全性
    /// 考虑。
    Get,
    /// HTTP/1.0 定义，参考[RFC 1945](https://datatracker.ietf.org/doc/html/rfc1945#section-8.3)
    /// HTTP/1.1 第二版更新，参考[RFC 2616](https://datatracker.ietf.org/doc/html/rfc2616#section-9.5)
    ///
    /// POST方法用于请求源服务器接受请求中包含的实体，作为Request-Line中的Request-URI所标识的资源的一个新的从属
    /// 实体。POST的设计允许使用统一的方法来覆盖以下函数:
    /// * 现有资源的注释;
    /// * 向公告板、新闻组、邮件列表或类似的文章组发布信息;
    /// * 向数据处理过程提供数据块，例如提交表单的结果;
    /// * 通过追加操作扩展数据库。
    ///
    /// POST方法执行的实际函数由服务器决定，通常依赖于Request-URI。发布的实体从属于该URI，就像文件从属于包含它的
    /// 目录、新闻文章从属于发布它的新闻组或记录从属于数据库一样。
    ///
    /// POST方法执行的操作可能不会产生可以用URI标识的资源。在这种情况下，200 (OK)或204 (No Content)是合适的响
    /// 应状态，这取决于响应是否包含描述结果的实体。
    ///
    /// 如果源服务器上已经创建了一个资源，响应应该是201 (created)，并且包含一个描述请求状态和引用新资源的实体，以
    /// 及一个Location报头(参见[协议](https://datatracker.ietf.org/doc/html/rfc2616#section-14.30) )。
    ///
    /// 此方法的响应是不可缓存的，除非响应包含适当的Cache-Control或Expires报头字段。然而，303响应可用于指导用户
    /// 代理检索可缓存资源。
    ///
    /// POST请求必须遵守[协议](https://datatracker.ietf.org/doc/html/rfc2616#section-8.2) 中所列的消息
    /// 传输要求。
    ///
    /// 参阅[协议](https://datatracker.ietf.org/doc/html/rfc2616#section-15.1.3) 了解安全性考虑。
    Post,
    /// HTTP/1.1 第一版定义，参考[RFC 2068](https://datatracker.ietf.org/doc/html/rfc2068#section-9.6)
    /// HTTP/1.1 第二版更新，参考[RFC 2616](https://datatracker.ietf.org/doc/html/rfc2616#section-9.6)
    ///
    /// PUT方法请求将所包含的实体存储在所提供的Request-URI下。如果Request-URI指向一个已经存在的资源，则应该将所
    /// 包含的实体视为原始服务器上的实体的修改版本。如果Request-URI不指向现有资源，并且请求用户代理能够将该URI定义
    /// 为新资源，则源服务器可以使用该URI创建资源。如果创建了一个新资源，源服务器必须通过201 (created)响应通知用户
    /// 代理。如果修改了现有的资源，则应该发送200 (OK)或204 (No Content)响应码来表示请求成功完成。如果不能使用
    /// Request-URI创建或修改资源，则应该给出一个反映问题本质的适当错误响应。实体的接收者绝对不能忽略任何它不理解或
    /// 实现的Content-*(例如Content-Range)报头，并且在这种情况下必须返回501(未实现)响应。
    ///
    /// 如果请求通过一个缓存，并且Request-URI标识了一个或多个当前缓存的实体，那么这些条目应该被视为过时的。此方法
    /// 的响应是不可缓存的。
    ///
    /// POST和PUT请求之间的根本区别反映在Request-URI的不同含义上。POST请求中的URI标识将处理所附实体的资源。该资
    /// 源可能是一个数据接受过程、到其他协议的网关或接受注释的独立实体。
    /// 相反，PUT请求中的URI标识与请求一起封装的实体——用户代理知道URI的目的是什么，并且服务器绝对不能试图将请求应用
    /// 到其他资源。如果服务器希望请求被应用到一个不同的URI，它必须发送一个301(永久移动)响应；然后，用户代理可以自
    /// 己决定是否重定向请求。
    ///
    /// 一个资源可以由许多不同的uri标识。例如，一篇文章可能有一个标识“当前版本”的URI，该URI与标识每个特定版本的URI
    /// 是分开的。在这种情况下，一个通用URI上的PUT请求可能会导致源服务器定义几个其他URI。
    ///
    /// HTTP/1.1没有定义PUT方法如何影响源服务器的状态。
    ///
    /// PUT请求必须遵守[协议](https://datatracker.ietf.org/doc/html/rfc2616#section-8.2) 中规定的消息传
    /// 输要求。
    ///
    /// 除非特别指定了实体头，PUT请求中的实体头应该应用于PUT创建或修改的资源。
    Put,
    /// HTTP/1.1 第一版定义，参考[RFC 2068](https://datatracker.ietf.org/doc/html/rfc2068#section-9.7)
    /// HTTP/1.1 第二版更新，参考[RFC 2616](https://datatracker.ietf.org/doc/html/rfc2616#section-9.7)
    ///
    /// DELETE方法请求源服务器删除Request-URI标识的资源。此方法可能被源服务器上的人为干预(或其他方法)覆盖。即使从
    /// 源服务器返回的状态码表明操作已经成功完成，客户端也不能保证操作已经执行。然而，服务器不应该表示成功，除非在给
    /// 出响应时，它打算删除资源或将其移动到不可访问的位置。
    ///
    /// 如果响应包含描述状态的实体，则响应应该是200 (OK)；如果操作还没有实施，则响应应该是202 (Accepted)；如果操
    /// 作已经实施，但响应不包含实体，则响应应该是204 (No Content)。
    ///
    /// 如果请求通过一个缓存，并且Request-URI标识了一个或多个当前缓存的实体，那么这些条目应该被视为过时的。此方法
    /// 的响应是不可缓存的。
    Delete,
    /// HTTP/1.0 定义，参考[RFC 1945](https://datatracker.ietf.org/doc/html/rfc1945#section-8.2)
    /// HTTP/1.1 第二版更新，参考[RFC 2616](https://datatracker.ietf.org/doc/html/rfc2616#section-9.4)
    ///
    /// HEAD方法与GET方法相同，不同之处是服务器MUST NOT在响应中返回消息体。响应HEAD请求的HTTP报头中包含的元信息
    /// 应该与响应GET请求时发送的信息相同。该方法可用于获取请求所隐含的实体的元信息，而无需传输实体本身。这种方法通
    /// 常用于测试超文本链接的有效性、可访问性和最近的修改。
    ///
    /// 对于HEAD请求的响应是可缓存的，因为响应中包含的信息可以用来更新该资源中先前缓存的实体。如果新的字段值表明缓存
    /// 的实体不同于当前实体(通过Content-Length、Content-MD5、ETag或Last-Modified的变化来表明)，那么缓存必须
    /// 将缓存条目视为过时的。
    Head,
    /// HTTP/1.1 第一版定义，参考[RFC 2068](https://datatracker.ietf.org/doc/html/rfc2068#section-9.8)
    /// HTTP/1.1 第二版更新，参考[RFC 2616](https://datatracker.ietf.org/doc/html/rfc2616#section-9.8)
    ///
    /// TRACE方法用于调用请求消息的远程应用程序层回环。请求的最终接收方应该将收到的消息作为200 (OK)响应的实体反射
    /// 回客户端。最终的接收方要么是源服务器，要么是第一个在请求中接收Max-Forwards值为zero(0)的代理或网关(见
    /// [协议](https://datatracker.ietf.org/doc/html/rfc2616#section-14.31) )。
    /// 一个TRACE请求一定不能包含一个实体。
    ///
    /// TRACE允许客户端查看在请求链的另一端接收到什么，并使用该数据进行测试或诊断信息。Via报头字段(
    /// [协议](https://datatracker.ietf.org/doc/html/rfc2616#section-14.45) )的值特别重要，因为它作为请
    /// 求链的跟踪。
    /// 使用Max-Forwards报头字段允许客户端限制请求链的长度，这对于测试无限循环中转发消息的代理链非常有用。
    ///
    /// 如果请求有效，响应应该在实体体中包含整个请求消息，其Content-Type为“message/http”。对此方法的响应一定不能
    /// 被缓存。
    Trace,
    /// HTTP/1.1 第二版定义，参考[RFC 2616](https://datatracker.ietf.org/doc/html/rfc2616#section-9.9)
    ///
    /// 该规范保留了方法名CONNECT，以便与可以动态切换为管道的代理(例如SSL【
    /// [44](https://datatracker.ietf.org/doc/html/rfc2616#ref-44) 】)一起使用。
    Connect,
    /// HTTP/1.1 第一版扩展方法定义，参考
    /// [RFC 2068](https://datatracker.ietf.org/doc/html/rfc2068#section-19.6.1.1)
    ///
    /// PATCH方法与PUT类似，不同之处在于实体中包含了一个列表，列出了Request-URI标识资源的原始版本与应用PATCH操作
    /// 后资源的期望内容之间的差异。差异列表的格式由实体的媒体类型(例如，“application/diff”)定义，并且必须包含足
    /// 够的信息，以允许服务器重新进行必要的更改，以将资源的原始版本转换为所需的版本。如果请求通过一个缓存，并且
    /// Request-URI标识了一个当前缓存的实体，那么该实体必须从缓存中删除。对该方法的响应是不可缓存的。确定如何放置打
    /// 过补丁的资源以及对其前身的处理的实际方法完全由源服务器定义。如果被修补的原始版本资源包含一个
    /// Content-Version报头字段，请求实体必须包含一个与原始Content-Version报头字段值相对应的Derived-From报头
    /// 字段。鼓励应用程序使用这些字段来构建版本关系和解决版本冲突。
    ///
    /// PATCH请求必须遵守[协议](https://datatracker.ietf.org/doc/html/rfc2068#section-8.2) 中规定的消息传输要求。
    Patch,
    /// HTTP/1.1 第一版扩展方法定义，参考[RFC 2068](https://datatracker.ietf.org/doc/html/rfc2068#section-19.6.1.2)
    ///
    /// LINK方法在Request-URI标识的现有资源和其他现有资源之间建立一个或多个LINK关系。LINK方法与其他允许在资源之
    /// 间建立链接的方法之间的区别在于，LINK方法不允许在请求中发送任何消息体，也不直接导致新资源的创建。如果请求通过
    /// 一个缓存，并且Request-URI标识了一个当前缓存的实体，那么该实体必须从缓存中删除。对该方法的响应是不可缓存的。
    /// 实现LINK的缓存应该使缓存的响应失效。
    Link,
    /// HTTP/1.1 第一版扩展方法定义，参考[RFC 2068](https://datatracker.ietf.org/doc/html/rfc2068#section-19.6.1.3)
    ///
    /// UNLINK方法从Request-URI标识的现有资源中删除一个或多个Link关系。这些关系可能已经通过LINK方法或任何其他支
    /// 持LINK头的方法建立。删除到资源的链接并不意味着该资源不再存在或将来的引用变得不可访问。如果请求通过一个缓存，
    /// 并且Request-URI标识了一个当前缓存的实体，那么该实体必须从缓存中删除。对该方法的响应是不可缓存的。实现
    /// UNLINK的缓存应该使缓存的响应失效。
    UnLink,
    /// HTTP/2 定义，参考[RFC 7540](https://datatracker.ietf.org/doc/html/rfc7540)
    Pri,
}

impl Method {
    /// OPTIONS 参考[`Inner::Options`]
    pub const OPTIONS: Method = Method(Options);

    /// GET 参考[`Inner::Get`]
    pub const GET: Method = Method(Get);

    /// POST 参考[`Inner::Post`]
    pub const POST: Method = Method(Post);

    /// PUT 参考[`Inner::Put`]
    pub const PUT: Method = Method(Put);

    /// DELETE 参考[`Inner::Delete`]
    pub const DELETE: Method = Method(Delete);

    /// HEAD 参考[`Inner::Head`]
    pub const HEAD: Method = Method(Head);

    /// TRACE 参考[`Inner::Trace`]
    pub const TRACE: Method = Method(Trace);

    /// CONNECT 参考[`Inner::Connect`]
    pub const CONNECT: Method = Method(Connect);

    /// PATCH 参考[`Inner::Patch`]
    pub const PATCH: Method = Method(Patch);

    /// LINK 参考[`Inner::Link`]
    pub const LINK: Method = Method(Link);

    /// UNLINK 参考[`Inner::UnLink`]
    pub const UNLINK: Method = Method(UnLink);

    /// PRI 参考[`Inner::Pri`]
    pub const PRI: Method = Method(Pri);

    /// 通过已知字节数组获取HTTP方法
    pub fn from_bytes(src: &[u8]) -> StarryResult<Method> {
        match src.len() {
            0 => Ok(Method(Get)),
            3 => match src {
                b"GET" => Ok(Method(Get)),
                b"PUT" => Ok(Method(Put)),
                b"PRI" => Ok(Method(Pri)),
                _ => Err(Errs::string(format!("invalid method {}!", String::from_utf8_lossy(src)))),
            },
            4 => match src {
                b"POST" => Ok(Method(Post)),
                b"HEAD" => Ok(Method(Head)),
                b"LINK" => Ok(Method(Link)),
                _ => Err(Errs::string(format!("invalid method {}!", String::from_utf8_lossy(src)))),
            },
            5 => match src {
                b"PATCH" => Ok(Method(Patch)),
                b"TRACE" => Ok(Method(Trace)),
                _ => Err(Errs::string(format!("invalid method {}!", String::from_utf8_lossy(src)))),
            },
            6 => match src {
                b"DELETE" => Ok(Method(Delete)),
                b"UNLINK" => Ok(Method(UnLink)),
                _ => Err(Errs::string(format!("invalid method {}!", String::from_utf8_lossy(src)))),
            },
            7 => match src {
                b"OPTIONS" => Ok(Method(Options)),
                b"CONNECT" => Ok(Method(Connect)),
                _ => Err(Errs::string(format!("invalid method {}!", String::from_utf8_lossy(src)))),
            },
            _ => Err(Errs::string(format!("invalid method {}!", String::from_utf8_lossy(src)))),
        }
    }

    pub fn from_str(t: &str) -> StarryResult<Method> {
        Method::from_bytes(t.as_bytes())
    }

    /// 用`&str`表示当前HTTP的方法
    pub fn as_str(&self) -> &str {
        match self.0 {
            Options => "OPTIONS",
            Get => "GET",
            Post => "POST",
            Put => "PUT",
            Delete => "DELETE",
            Head => "HEAD",
            Trace => "TRACE",
            Connect => "CONNECT",
            Patch => "PATCH",
            Link => "LINK",
            UnLink => "UNLINK",
            Pri => "PRI",
        }
    }

    /// Safe Methods 常见方法属性之一
    ///
    /// * 一个方法是否被认为是“安全的”，这意味着请求本质上是只读的。
    ///
    /// 如果请求方法定义的语义本质上是只读的，则认为它们是“安全的”；也就是说，客户端不请求，也不期望源服务器上的任何
    /// 状态变化，因为对目标资源应用了安全方法。
    /// 同样，合理使用安全方法也不会对源服务器造成任何伤害、财产损失或异常负担。
    ///
    /// 安全方法的定义并不阻止实现包含潜在有害的行为、不是完全只读的行为或在调用安全方法时引起副作用的行为。
    /// 然而，重要的是，客户并没有要求额外的行为，因此不能对此负责。
    /// 例如，大多数服务器在每个响应完成后都会附加请求信息以访问日志文件，而不管使用什么方法，即使日志存储空间已满并
    /// 可能导致服务器崩溃，这也被认为是安全的。
    /// 同样，通过在Web上选择广告发起的安全请求通常会产生收取广告账户费用的副作用。
    ///
    /// 在这个规范定义的请求方法中，GET、HEAD、OPTIONS和TRACE方法被定义为安全的。
    ///
    /// 区分安全方法和不安全方法的目的是允许自动检索过程(spider)和缓存性能优化(预读)工作，而不用担心造成损害。
    /// 此外，它允许用户代理在处理可能不可信的内容时，对不安全方法的自动使用应用适当的约束。
    ///
    /// 当向用户呈现潜在的操作时，用户代理应该区分安全的和不安全的方法，这样用户可以在请求不安全的操作之前就知道它。
    ///
    /// 当资源被构造成有效请求URI中的参数具有选择操作的效果时，资源所有者有责任确保操作与请求方法语义一致。
    /// 例如，基于web的内容编辑软件通常在查询参数中使用操作，例如“page?do=delete”。
    /// 如果这种资源的目的是执行不安全的操作，那么当使用安全请求方法访问资源时，资源所有者必须禁用或不允许该操作。
    /// 如果不这样做，当自动化进程为了链接维护、预取、构建搜索索引等目的对每个URI引用执行GET时，将导致不可控的副作用。
    pub fn is_safe(&self) -> bool {
        match self.0 {
            Get | Head | Options | Trace => true,
            _ => false,
        }
    }

    /// Idempotent Methods 常见方法属性之一
    ///
    /// * 一个方法是否被认为是“幂等的”，这意味着如果多次执行请求，结果是相同的。
    ///
    /// 如果使用该方法对服务器的多个相同请求的预期效果与单个请求的效果相同，则该请求方法被认为是“幂等的”。
    /// 在本规范定义的请求方法中，PUT、DELETE和安全请求方法是幂等的。
    ///
    /// 就像[`安全的`]定义一样，幂等性质只适用于用户所要求的东西；服务器可以自由地分别记录每个请求，保留修订控制历史，
    /// 或者为每个幂等请求实现其他非幂等副作用。
    ///
    /// 幂等方法的区别在于，如果在客户端能够读取服务器的响应之前发生通信故障，请求可以自动重复。
    /// 例如，如果客户机发送了一个PUT请求，而底层连接在接收到任何响应之前就关闭了，那么客户端可以建立一个新的连接并
    /// 重试幂等请求。
    /// 它知道重复请求将具有相同的预期效果，即使原始请求成功了，但是响应可能不同。
    ///
    /// [`安全的`]: Method::is_safe
    pub fn is_idempotent(&self) -> bool {
        match self.0 {
            Put | Delete => true,
            _ => false,
        }
    }

    /// Cacheable Methods 常见方法属性之一
    ///
    /// * 一个方法是否被认为是“可缓存的”，这意味着对它们的响应可以被存储以供将来重用。
    ///
    /// 请求方法可以被定义为“可缓存的”，具体要求见[`RFC7234`]。
    /// 通常，不依赖于当前或权威响应的安全方法被定义为可缓存的；
    /// 该规范将GET、HEAD和POST定义为可缓存的，尽管绝大多数缓存实现只支持GET和HEAD。
    ///
    /// [`RFC7234`]: https://datatracker.ietf.org/doc/html/rfc7234
    pub fn is_cacheable(&self) -> bool {
        match self.0 {
            Get | Head | Post => true,
            _ => false,
        }
    }
}

impl AsRef<str> for Method {
    fn as_ref(&self) -> &str {
        self.as_str()
    }
}

impl<'a> PartialEq<&'a Method> for Method {
    fn eq(&self, other: &&'a Method) -> bool {
        self == *other
    }
}

impl<'a> PartialEq<Method> for &'a Method {
    fn eq(&self, other: &Method) -> bool {
        *self == other
    }
}

impl PartialEq<str> for Method {
    fn eq(&self, other: &str) -> bool {
        self.as_ref() == other
    }
}

impl PartialEq<Method> for str {
    fn eq(&self, other: &Method) -> bool {
        self == other.as_ref()
    }
}

impl<'a> PartialEq<&'a str> for Method {
    fn eq(&self, other: &&'a str) -> bool {
        self.as_ref() == *other
    }
}

impl<'a> PartialEq<Method> for &'a str {
    fn eq(&self, other: &Method) -> bool {
        *self == other.as_ref()
    }
}

impl fmt::Debug for Method {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_ref())
    }
}

impl Display for Method {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

#[cfg(test)]
mod method_test {
    use crate::Method;

    #[test]
    fn method_eq() {
        assert_eq!(Method::GET, Method::GET);
        assert_eq!(Method::GET, "GET");
        assert_eq!(&Method::GET, "GET");

        assert_eq!("GET", Method::GET);
        assert_eq!("GET", &Method::GET);

        assert_eq!(&Method::GET, Method::GET);
        assert_eq!(Method::GET, &Method::GET);
    }

    #[test]
    fn invalid_method() {
        assert!(Method::from_str("").is_ok());
        assert!(Method::from_bytes(b"").is_ok());
        assert!(Method::from_str("omg").is_err());
        assert!(Method::from_bytes(b"omg").is_err());
        assert!(Method::from_bytes(&[0xC0]).is_err()); // invalid utf-8
        assert!(Method::from_bytes(&[0x10]).is_err()); // invalid method characters
    }

    #[test]
    fn is_safe() {
        assert!(Method::HEAD.is_safe());
        assert!(Method::TRACE.is_safe());
        assert!(Method::OPTIONS.is_safe());
        assert!(Method::GET.is_safe());

        assert!(!Method::PUT.is_safe());
        assert!(!Method::DELETE.is_safe());
        assert!(!Method::POST.is_safe());
        assert!(!Method::CONNECT.is_safe());
        assert!(!Method::PATCH.is_safe());
    }

    #[test]
    fn is_idempotent() {
        assert!(Method::PUT.is_idempotent());
        assert!(Method::DELETE.is_idempotent());

        assert!(!Method::OPTIONS.is_idempotent());
        assert!(!Method::GET.is_idempotent());
        assert!(!Method::HEAD.is_idempotent());
        assert!(!Method::TRACE.is_idempotent());
        assert!(!Method::POST.is_idempotent());
        assert!(!Method::CONNECT.is_idempotent());
        assert!(!Method::PATCH.is_idempotent());
    }

    #[test]
    fn is_cacheable() {
        assert!(Method::GET.is_cacheable());
        assert!(Method::POST.is_cacheable());
        assert!(Method::HEAD.is_cacheable());

        assert!(!Method::PUT.is_cacheable());
        assert!(!Method::OPTIONS.is_cacheable());
        assert!(!Method::DELETE.is_cacheable());
        assert!(!Method::TRACE.is_cacheable());
        assert!(!Method::CONNECT.is_cacheable());
        assert!(!Method::PATCH.is_cacheable());
    }
}
