# starry
基于Rust的Http服务库，提供Server和Client使用

## 开发环境
* Rust 1.55.0+
* Darwin/amd64

## 功能特性
### 服务端
* RESTFUL
* 过滤器
* 限流
* 熔断（计划）
* 代理（计划）
* TLS（计划）
### 客户端
* HTTP
* HTTPS（计划）

## 示例
### 使用HTTP Server
```rust
use starry::{Context, Cookie, HttpServer, Status};

fn main() {
    let server = HttpServer::new();
    router1(server.clone());
    router2(server.clone());
    let addr = "0.0.0.0:7878";
    server.listener(addr).unwrap()
}

fn router1(server: HttpServer) {
    let router = server.router_wf("/path/test", Extend::e1(vec![f11, f12]));
    // 过滤过程 f13 f11 f12
    router.get_wf("/test1/:a/:b", h11, Extend::e1(vec![f13]));
    // 过滤过程 f14 f11 f12
    router.get_wf("/test1/:a/c/:b", h12, Extend::e1(vec![f14]));
    router.get_wf("/test1/:a/c/d/:b", h13, Extend::e1(vec![f15]));
    router.get_wf("/test1/:a/c/d/e/:b", h13, Extend::e3(vec![f15], Limit::new(1000, 10, 300)));
    router.get("/test1/:a/c/d/e/f/:b", h13);
    router.get("/test1/a/c/d/:b", h14);
}

fn f11(context: &mut Context) {
    context.resp_status(Status::OK);
    println!("f11 path = {}", context.req_path());
}

fn f12(context: &mut Context) {
    println!("f12 path = {}", context.req_path());
}

fn f13(context: &mut Context) {
    println!("f13 path = {}", context.req_path());
}

fn f14(context: &mut Context) {
    println!("f14 path = {}", context.req_path());
}

fn f15(context: &mut Context) {
    println!("f15 path = {}", context.req_path());
    context.resp_status(Status::OK);
    context.response()
}

fn h11(mut context: Box<Context>) {
    context.resp_status(Status::OK);
    context.resp_set_header_str("a", "b");
    context.resp_set_header_str("m", "n");
    context.resp_set_header_str("x", "y");
    context.resp_add_cookie(Cookie::new("hello".to_string(), "world".to_string()));
    context.resp_add_cookie(Cookie::new("starry".to_string(), "http".to_string()));
    context.resp_body("test http response body 是否有效！".to_string().into_bytes());
    println!("h11");
    println!("a = {}", context.req_field("a").unwrap());
    println!("b = {}", context.req_field("b").unwrap());
    context.response();
}

fn h12(mut context: Box<Context>) {
    context.resp_status(Status::BAD_REQUEST);
    println!("h12");
    println!("a = {}", context.req_field("a").unwrap());
    println!("b = {}", context.req_field("b").unwrap());
    context.response();
}

fn h13(mut context: Box<Context>) {
    context.resp_status(Status::MULTI_STATUS);
    println!("h13");
    println!("a = {}", context.req_field("a").unwrap());
    println!("b = {}", context.req_field("b").unwrap());
    context.response();
}

// 该方法永远不会执行，被h13拦截了
fn h14(mut context: Box<Context>) {
    context.resp_status(Status::LENGTH_REQUIRED);
    println!("h14");
    println!("b = {}", context.req_field("b").unwrap());
    context.response();
}

fn router2(server: HttpServer) {
    let router = server.router("/path/test");
    router.post("/test1/:a/:b", h21, vec![]);
}

fn h21(mut context: Context) {
    context.resp_status(Status::BAD_REQUEST);
    println!("h12");
    println!("a = {}", context.get_field("a").unwrap());
    println!("b = {}", context.get_field("b").unwrap());
    println!("1 = {:#?}", context.get_form_value("1").unwrap());
    println!("4 = {:#?}", context.get_form_file("4").unwrap());
    println!("10 = {:#?}", context.get_form_file("10").unwrap());
    context.response().unwrap();
}
```
更多详情参考：https://github.com/aberic/starry/blob/master/examples/server_nor.rs

### 使用HTTP Client
```rust
use starry::HttpClient;

fn get1() {
    let mut resp = HttpClient::get("http://user:password@localhost:7878/path/test/test1/hello/world").unwrap();
    println!("resp = {:#?}", resp);
}
```
更多详情参考：https://github.com/aberic/starry/blob/master/examples/client_nor.rs

### 文档
暂无

### 说明
coding中

## License
[Apache License 2.0](https://www.apache.org/licenses/LICENSE-2.0.html)