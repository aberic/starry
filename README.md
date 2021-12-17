# starry
基于Rust的Http服务库，提供Server和Client使用

## 开发环境
* Rust 1.55.0+
* Darwin/amd64

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
    let router = server.router("/path/test");
    router.get("/test1/:a/:b", h11, vec![]);
}

fn h11(mut context: Context) {
    context.resp_status(Status::OK);
    context.resp_set_header_str("a", "b");
    context.resp_set_header_str("m", "n");
    context.resp_set_header_str("x", "y");
    context.resp_add_set_cookie(Cookie::new("hello".to_string(), "world".to_string()));
    context.resp_add_set_cookie(Cookie::new("starry".to_string(), "http".to_string()));
    context.resp_set_body("test http response body 是否有效！".to_string().into_bytes());
    println!("h11");
    println!("a = {}", context.get_field("a").unwrap());
    println!("b = {}", context.get_field("b").unwrap());
    context.response().unwrap();
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
更多详情参考：https://github.com/aberic/starry/blob/master/examples/server.rs

### 文档
暂无

### 说明
coding

<br><br>