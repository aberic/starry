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

use crate::utils::errors::{Errs, StarryResult};
use crate::utils::Time;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Cookie {
    /// cookie的名称
    pub(crate) name: String,
    /// cookie的值
    value: String,
    /// 可以访问此cookie的页面路径
    path: Option<String>,
    /// 可以访问此cookie的域名
    ///
    /// domain属性可以使多个web服务器共享cookie。
    /// domain属性的默认值是创建cookie的网页所在服务器的主机名。
    /// 不能将一个cookie的域设置成服务器所在的域之外的域。
    ///
    /// 例如让位于a.example.com的服务器能够读取b.example.com设置的cookie值。
    /// 如果b.example.com的页面创建的cookie把它的path属性设置为"/"，把domain属性设置成".example.com"，
    /// 那么所有位于b.example.com的网页和所有位于a.example.com的网页，以及位于example.com域的其他服务器上的网
    /// 页都可以访问这个cookie。
    ///
    /// 非顶级域名，如二级域名或者三级域名，设置的cookie的domain只能为顶级域名或者二级域名或者三级域名本身，
    /// 不能设置其他二级域名的cookie，否则cookie无法生成。
    ///
    /// 顶级域名只能设置domain为顶级域名，不能设置为二级域名或者三级域名，否则cookie无法生成。
    ///
    /// 二级域名能读取设置了domain为顶级域名或者自身的cookie，不能读取其他二级域名domain的cookie。
    ///
    /// 所以要想cookie在多个二级域名中共享，需要设置domain为顶级域名，这样就可以在所有二级域名里面或者到这个cookie的值了。
    ///
    /// 顶级域名只能获取到domain设置为顶级域名的cookie，其他domain设置为二级域名的无法获取。
    domain: Option<String>,
    /// 此cookie超时时间
    ///
    /// 指定了cookie的生存期，默认情况下cookie是暂时存在的，他们存储的值只在浏览器会话期间存在，当用户退出浏览器
    /// 后这些值也会丢失，如果想让cookie存在一段时间，就要为expires属性设置为未来的一个用毫秒数表示的过期日期或
    /// 时间点，expires默认为设置的expires的当前时间。
    ///
    /// 现在已经被max-age属性所取代，max-age用秒来设置cookie的生存期。
    expires: Option<Time>,
    /// 设置cookie的生存期
    ///
    /// max-age属性为正数，则表示该cookie会在max-age秒之后自动失效。
    /// 浏览器会将max-age为正数的cookie持久化，即写到对应的cookie文件中。
    /// 无论客户关闭了浏览器还是电脑，只要还在max-age秒之前，登录网站时该cookie仍然有效。
    ///
    /// max-age为负数，则表示该cookie仅在本浏览器窗口以及本窗口打开的子窗口内有效，关闭窗口后该cookie即失效。
    /// max-age为负数的Cookie，为临时性cookie，不会被持久化，不会被写到cookie文件中。
    /// cookie信息保存在浏览器内存中，因此关闭浏览器该cookie就消失了。cookie默认的max-age值为-1。
    ///
    /// max-age为0，则表示删除该cookie。
    /// cookie机制没有提供删除cookie的方法，因此通过设置该cookie即时失效实现删除cookie的效果。
    /// 失效的Cookie会被浏览器从cookie文件或者内存中删除。
    ///
    /// 如果不设置expires或者max-age这个cookie默认是Session的，也就是关闭浏览器该cookie就消失了。
    max_age: Option<isize>,
    /// 设置是否只能通过https来传递此条cookie
    ///
    /// 它是一个布尔值，指定在网络上如何传输cookie，默认是不安全的，通过一个普通的http连接传输。
    secure: bool,
    /// 若此属性为true，则只有在http请求头中会带有此cookie的信息，而不能通过document.cookie来访问此cookie。
    ///
    /// 这意味着，浏览器脚本，比如javascript中，是不允许访问操作此cookie的。
    httponly: bool,
    /// SameSite属性用于限制第三方Cookie，从而降低安全风险，减少`CSRF`攻击
    same_site: Option<SameSite>,
}

impl Cookie {
    pub fn new(name: String, value: String) -> Cookie {
        Cookie {
            name,
            value,
            path: None,
            domain: None,
            expires: None,
            max_age: None,
            secure: false,
            httponly: false,
            same_site: None,
        }
    }

    pub fn create(name: String, value: String, path: Option<String>, domain: Option<String>,
                  expires: Option<Time>, max_age: Option<isize>, secure: bool,
                  httponly: bool) -> Cookie {
        Cookie {
            name,
            value,
            path,
            domain,
            expires,
            max_age,
            secure,
            httponly,
            same_site: None,
        }
    }

    /// read_cookie 解析头header中的所有“Set-Cookie”值，并返回成功解析的Cookie。
    pub(crate) fn read_set_cookies(opt: Option<Vec<String>>) -> StarryResult<Vec<Cookie>> {
        let mut cookies = vec![];
        match opt {
            Some(src) => {
                for s in src {
                    let mut cookie = Cookie::default();
                    let res = s.trim().split(";");
                    for v in res {
                        let parts: Vec<&str> = v.trim().split("=").collect();
                        let parts_len = parts.len();
                        if parts_len > 2 {
                            return Err(Errs::string(format!("cookie's value invalid! can not support {}", v)));
                        }
                        let key = parts[0].to_string();
                        match key.to_lowercase().as_str() {
                            "path" => cookie.path = if parts_len == 2 {
                                Some(parts[1].to_string())
                            } else {
                                None
                            },
                            "domain" => cookie.domain = if parts_len == 2 {
                                Some(parts[1].to_string())
                            } else {
                                None
                            },
                            "expires" => cookie.expires = if parts_len == 2 {
                                Some(match Time::parse_from_str(parts[1], "%a, %d %b %Y %H:%M:%S GMT") {
                                    Ok(src) => src,
                                    Err(err) => return Err(Errs::strs("set-cookie's expires invalid!", err))
                                })
                            } else {
                                None
                            },
                            "max-age" => cookie.max_age = if parts_len == 2 {
                                Some(match parts[1].parse::<isize>() {
                                    Ok(src) => src,
                                    Err(err) => return Err(Errs::strs("set-cookie's max-age invalid!", err))
                                })
                            } else {
                                None
                            },
                            "secure" => cookie.secure = true,
                            "httponly" => cookie.httponly = true,
                            "samesite" => cookie.same_site = if parts_len == 2 {
                                match parts[1].to_lowercase().as_str() {
                                    "node" => Some(SameSite::None),
                                    "strict" => Some(SameSite::Strict),
                                    "lax" => Some(SameSite::Lax),
                                    _ => None
                                }
                            } else {
                                None
                            },
                            _ => if parts_len == 2 {
                                cookie.name = key;
                                cookie.value = parts[1].to_string();
                            } else {
                                continue
                            }
                        }
                    }
                    cookies.push(cookie)
                }
            }
            None => {}
        }
        Ok(cookies)
    }

    /// read_cookie 解析头header中的所有“Cookie”值，并返回成功解析的Cookie。
    /// 如果filter不为空，则只返回该名称的cookie
    pub(crate) fn read_cookies(opt: Option<Vec<String>>, filter: &str) -> StarryResult<Vec<Cookie>> {
        let mut cookies = vec![];
        match opt {
            Some(src) => {
                for s in src {
                    let res = s.trim().split(";");
                    for v in res {
                        let kv: Vec<&str> = v.trim().split("=").collect();
                        if kv.len() != 2 {
                            return Err(Errs::string(format!("cookie's value invalid! can not support {}", v)));
                        }
                        if filter.is_empty() {
                            cookies.push(Cookie::new(kv[0].to_string(), kv[1].to_string()))
                        } else {
                            let key = kv[0].to_string();
                            if key.eq(filter) {
                                cookies.push(Cookie::new(key, kv[1].to_string()))
                            }
                        }
                    }
                }
            }
            None => {}
        }
        Ok(cookies)
    }

    fn expires_string(&self) -> String {
        let mut now = Time::now();
        now.add(self.expires.unwrap().duration());
        now.format_string("%a, %d %b %Y %H:%M:%S GMT")
    }
}

impl Default for Cookie {
    fn default() -> Self {
        Cookie {
            name: "".to_string(),
            value: "".to_string(),
            path: None,
            domain: None,
            expires: None,
            max_age: None,
            secure: false,
            httponly: false,
            same_site: None,
        }
    }
}

impl<'a> PartialEq<&'a Cookie> for Cookie {
    fn eq(&self, other: &&'a Cookie) -> bool {
        self == *other
    }
}

impl<'a> PartialEq<Cookie> for &'a Cookie {
    fn eq(&self, other: &Cookie) -> bool {
        *self == other
    }
}

impl ToString for Cookie {
    fn to_string(&self) -> String {
        let mut cookie_string;
        if self.name.is_empty() {
            cookie_string = "".to_string();
        } else {
            if self.value.is_empty() {
                cookie_string = "".to_string();
            } else {
                cookie_string = format!("{}={}", self.name, self.value);
            }
        }
        match self.path.clone() {
            Some(path) => cookie_string = format!("{}; Path={}", cookie_string, path),
            None => {}
        }
        match self.domain.clone() {
            Some(domain) => cookie_string = format!("{}; Domain={}", cookie_string, domain),
            None => {}
        }
        match self.expires {
            Some(_) => cookie_string = format!("{}; Expires={}", cookie_string, self.expires_string()),
            None => {}
        }
        if self.httponly {
            cookie_string = format!("{}; HttpOnly", cookie_string)
        }
        if self.secure {
            cookie_string = format!("{}; Secure", cookie_string)
        }
        match self.max_age {
            Some(max_age) => cookie_string = format!("{}; Max-Age={}", cookie_string, max_age),
            None => {}
        }
        match self.same_site.clone() {
            Some(same_site) => match same_site {
                SameSite::None => cookie_string = format!("{}; SameSite=None", cookie_string),
                SameSite::Strict => cookie_string = format!("{}; SameSite=Strict", cookie_string),
                SameSite::Lax => cookie_string = format!("{}; SameSite=Lax", cookie_string),
            },
            None => {}
        }
        cookie_string
    }
}

pub struct CookieBuilder {
    cookie: Cookie,
}

impl CookieBuilder {
    pub fn new() -> Self {
        CookieBuilder { cookie: Default::default() }
    }

    pub fn name_value(&mut self, name: String, value: String) -> &mut CookieBuilder {
        self.cookie.name = name;
        self.cookie.value = value;
        self
    }

    pub fn path(&mut self, path: String) -> &mut CookieBuilder {
        self.cookie.path = Some(path);
        self
    }

    pub fn domain(&mut self, domain: String) -> &mut CookieBuilder {
        self.cookie.domain = Some(domain);
        self
    }

    pub fn expires(&mut self, expires: Time) -> &mut CookieBuilder {
        self.cookie.expires = Some(expires);
        self
    }

    pub fn max_age(&mut self, max_age: isize) -> &mut CookieBuilder {
        self.cookie.max_age = Some(max_age);
        self
    }

    pub fn secure(&mut self, secure: bool) -> &mut CookieBuilder {
        self.cookie.secure = secure;
        self
    }

    pub fn httponly(&mut self, httponly: bool) -> &mut CookieBuilder {
        self.cookie.httponly = httponly;
        self
    }

    pub fn same_site(&mut self, same_site: SameSite) -> &mut CookieBuilder {
        self.cookie.same_site = Some(same_site);
        self
    }

    pub fn build(&self) -> Cookie {
        self.cookie.clone()
    }
}

/// SameSite属性用于限制第三方Cookie，从而降低安全风险，减少`CSRF`攻击
///
/// Cookie 往往用来存储用户的身份信息，恶意网站可以设法伪造带有正确 Cookie 的 HTTP 请求，这就是 CSRF 攻击。
/// 举例来说，用户登陆了银行网站your-bank.com，银行服务器发来了一个 Cookie。
///
/// ```norust
/// Set-Cookie:id=a3fWa;
/// ```
///
/// 用户后来又访问了恶意网站`example.com`，上面有一个表单。
///
/// ```html
/// <form action="your-bank.com/transfer" method="POST">
///   ...
/// </form>
/// ```
/// 用户一旦被诱骗发送这个表单，银行网站就会收到带有正确 Cookie 的请求。
/// 为了防止这种攻击，表单一般都带有一个随机 token，告诉服务器这是真实请求。
///
/// ```html
/// <form action="your-bank.com/transfer" method="POST">
///   <input type="hidden" name="token" value="dad3weg34">
///   ...
/// </form>
/// ```
///
/// 这种第三方网站引导发出的 Cookie，就称为第三方 Cookie。它除了用于 CSRF 攻击，还可以用于用户追踪。
///
/// Chrome 51 开始，浏览器的 Cookie 新增加了一个SameSite属性，用来防止 CSRF 攻击和用户追踪。
///
/// 设置了Strict或Lax以后，基本就杜绝了 CSRF 攻击。前提是用户浏览器支持 SameSite 属性。
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum SameSite {
    /// 完全禁止第三方 Cookie，跨站点时，任何情况下都不会发送 Cookie。
    /// 换言之，只有当前网页的 URL 与请求目标一致，才会带上 Cookie。
    /// 这个规则过于严格，可能造成非常不好的用户体验。
    /// 比如，当前网页有一个 GitHub 链接，用户点击跳转就不会带有 GitHub 的 Cookie，跳转过去总是未登陆状态。
    Strict,
    /// 规则稍稍放宽，大多数情况也是不发送第三方 Cookie，但是导航到目标网址的 Get 请求除外。
    /// 导航到目标网址的 GET 请求，只包括三种情况：链接，预加载请求，GET 表单。详见下表。
    /// ```lax
    /// ┌───────────┬───────────────────────────────────────┬─────────────┬─────────────┐
    /// │  req type │               example                 │    normal   │     lax     │
    /// ├───────────┼───────────────────────────────────────┼─────────────┼─────────────┤
    /// │    link   │          <a href="..."></a>           │ send Cookie │ send Cookie │
    /// ├───────────┼───────────────────────────────────────┼─────────────┼─────────────┤
    /// │ pre load  │   <link rel="prerender" href="..."/>  │ send Cookie │ send Cookie │
    /// ├───────────┼───────────────────────────────────────┼─────────────┼─────────────┤
    /// │ get form  │   <form method="Get" action="...">    │ send Cookie │ send Cookie │
    /// ├───────────┼───────────────────────────────────────┼─────────────┼─────────────┤
    /// │ post form │   <form method="POST" action="...">   │ send Cookie │   no send   │
    /// ├───────────┼───────────────────────────────────────┼─────────────┼─────────────┤
    /// │   iframe  │      <iframe src="..."></iframe>      │ send Cookie │   no send   │
    /// ├───────────┼───────────────────────────────────────┼─────────────┼─────────────┤
    /// │    ajax   │        $.get("...")$.get("...")       │ send Cookie │   no send   │
    /// ├───────────┼───────────────────────────────────────┼─────────────┼─────────────┤
    /// │   image   │              <img src="...">          │ send Cookie │   no send   │
    /// └───────────┴───────────────────────────────────────┴─────────────┴─────────────┘
    /// ```
    Lax,
    /// Chrome 计划将Lax变为默认设置。
    /// 这时，网站可以选择显式关闭SameSite属性，将其设为None。
    /// 不过，前提是必须同时设置Secure属性（Cookie 只能通过 HTTPS 协议发送），否则无效。
    ///
    /// 下面的设置无效：
    ///
    /// ```norust
    /// Set-Cookie: widget_session=abc123; SameSite=None
    /// ```
    ///
    /// 下面的设置有效：
    ///
    /// ```norust
    /// Set-Cookie: widget_session=abc123; SameSite=None; Secure
    /// ```
    None,
}

#[cfg(test)]
mod cookie_test {
    use crate::{Cookie, CookieBuilder, Response};
    use crate::utils::Time;

    #[test]
    fn cookie_builder() {
        let time = Time::now();
        assert_eq!(
            CookieBuilder::new().name_value("1".to_string(), "2".to_string()).secure(false).build(),
            Cookie::create("1".to_string(), "2".to_string(), None, None, None, None, false, false));
        assert_eq!(
            CookieBuilder::new().name_value("3".to_string(), "4".to_string()).secure(true).build(),
            Cookie::create("3".to_string(), "4".to_string(), None, None, None, None, true, false));
        assert_eq!(
            CookieBuilder::new().name_value("5".to_string(), "6".to_string()).expires(time).build(),
            Cookie::create("5".to_string(), "6".to_string(), None, None, Some(time), None, false, false));
        assert_eq!(
            CookieBuilder::new().name_value("1".to_string(), "2".to_string()).max_age(100).build(),
            Cookie::create("1".to_string(), "2".to_string(), None, None, None, Some(100), false, false));
        assert_eq!(
            CookieBuilder::new().name_value("1".to_string(), "2".to_string()).expires(time).max_age(100).build(),
            Cookie::create("1".to_string(), "2".to_string(), None, None, Some(time), Some(100), false, false));
    }

    #[test]
    fn read_set_cookies() {
        let mut resp = Response::success();
        resp.add_set_cookie(CookieBuilder::new().name_value("1".to_string(), "2".to_string()).secure(false).build());
        resp.add_set_cookie(CookieBuilder::new().name_value("3".to_string(), "4".to_string()).secure(true).build());
        resp.add_set_cookie(CookieBuilder::new().name_value("7".to_string(), "8".to_string()).max_age(100).build());
        let cks = resp.read_set_cookies().unwrap();
        for (i, ck) in cks.iter().enumerate() {
            match i {
                0 => assert_eq!(
                    ck,
                    Cookie::create("1".to_string(), "2".to_string(), None, None, None, None, false, false)),
                1 => assert_eq!(
                    ck,
                    Cookie::create("3".to_string(), "4".to_string(), None, None, None, None, true, false)),
                2 => assert_eq!(
                    ck,
                    Cookie::create("7".to_string(), "8".to_string(), None, None, None, Some(100), false, false)),
                _ => panic!("error!")
            }
        }
    }
}
