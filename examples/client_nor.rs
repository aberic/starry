/*
 * Copyright (c) $originalComment.match("Copyright \(c\) (\d+)", 1, "-")2021. Aberic - All Rights Reserved.
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

use std::io::{Read, Write};
use std::net::{TcpListener, TcpStream};

use log::LevelFilter;

use starry::utils::log::LogModule;

#[cfg(test)]
mod client_test {
    use starry::HttpClient;

    use crate::logs;

    #[test]
    fn get1() {
        logs();
        let mut resp = HttpClient::get("http://user:password@localhost:7878/path/test/test1/hello/world").unwrap();
        println!("resp = {:#?}", resp);
        println!("body = {}", String::from_utf8_lossy(resp.body().as_slice()).to_string());
    }

    #[test]
    fn get2() {
        logs();
        let resp = HttpClient::get("http://www.baidu.com").unwrap();
        println!("resp = {:#?}", resp);
    }
}

fn main() {
    let listener = TcpListener::bind("127.0.0.1:7878").unwrap();

    for stream in listener.incoming() {
        let stream = stream.unwrap();

        handle_connection(stream);
    }
}

fn handle_connection(mut stream: TcpStream) {
    let mut buffer = [0; 10240];

    stream.read(&mut buffer).unwrap();

    println!("Request: \n{}", String::from_utf8_lossy(&buffer[..]));

    let response = "HTTP/1.1 200 OK\r\n\r\n";

    stream.write(response.as_bytes()).unwrap();
    stream.flush().unwrap();
}


fn logs() {
    let module = LogModule {
        name: String::from("starry-http-client"),
        pkg: "".to_string(),
        level: LevelFilter::Trace,
        additive: true,
        dir: String::from("tmp"),
        file_max_size: 1024,
        file_max_count: 7,
    };
    module.config_log(vec![]);
}