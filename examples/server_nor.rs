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

// use std::io::{Read, Write};
// use std::net::{TcpListener, TcpStream};

use starry::{Context, Cookie, HttpServer, Status};

fn main() {
    let server = HttpServer::new();
    router1(server.clone());
    router2(server.clone());
    let addr = "0.0.0.0:7878";
    server.listener(addr).unwrap()
}

fn router1(server: HttpServer) {
    let router = server.router_wf("/path/test", vec![f11, f12]);
    router.get_wf("/test1/:a/:b", h11, vec![f13]);
    router.get("/test1/:a/c/:b", h12);
    router.get("/test1/:a/c/d/:b", h13);
    router.get("/test1/a/c/d/:b", h14);
}

fn f11(context: &Context) {
    println!("f11 path = {}", context.req_path());
}

fn f12(context: &Context) {
    println!("f12 path = {}", context.req_path());
}

fn f13(context: &Context) {
    println!("f13 path = {}", context.req_path());
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
    context.response().unwrap();
}

fn h12(mut context: Box<Context>) {
    context.resp_status(Status::BAD_REQUEST);
    println!("h12");
    println!("a = {}", context.req_field("a").unwrap());
    println!("b = {}", context.req_field("b").unwrap());
    context.response().unwrap();
}

fn h13(mut context: Box<Context>) {
    context.resp_status(Status::MULTI_STATUS);
    println!("h13");
    println!("a = {}", context.req_field("a").unwrap());
    println!("b = {}", context.req_field("b").unwrap());
    context.response().unwrap();
}

// 该方法永远不会执行，被h13拦截了
fn h14(mut context: Box<Context>) {
    context.resp_status(Status::LENGTH_REQUIRED);
    println!("h14");
    println!("b = {}", context.req_field("b").unwrap());
    context.response().unwrap();
}

fn router2(server: HttpServer) {
    let router = server.router("/path/test");
    router.post("/test1/:a/:b", h21);
    router.post("/test1/:a/c/:b", h22);
    router.post("/test1/:a/c/d/:b", h23);
    router.post("/test1/a/c/d/:b", h24);
}

fn h21(mut context: Box<Context>) {
    context.resp_status(Status::BAD_REQUEST);
    println!("h12");
    println!("a = {}", context.req_field("a").unwrap());
    println!("b = {}", context.req_field("b").unwrap());
    println!("1 = {:#?}", context.req_form("1").unwrap());
    println!("4 = {:#?}", context.req_form_file("4").unwrap());
    println!("10 = {:#?}", context.req_form_file("10").unwrap());
    context.response().unwrap();
}

fn h22(mut context: Box<Context>) {
    context.resp_status(Status::BAD_REQUEST);
    println!("h12");
    println!("a = {}", context.req_field("a").unwrap());
    println!("b = {}", context.req_field("b").unwrap());
    context.response().unwrap();
}

fn h23(mut context: Box<Context>) {
    context.resp_status(Status::MULTI_STATUS);
    println!("h13");
    println!("a = {}", context.req_field("a").unwrap());
    println!("b = {}", context.req_field("b").unwrap());
    context.response().unwrap();
}

// 该方法永远不会执行，被h23拦截了
fn h24(mut context: Box<Context>) {
    context.resp_status(Status::LENGTH_REQUIRED);
    println!("h14");
    println!("b = {}", context.req_field("b").unwrap());
    context.response().unwrap();
}
