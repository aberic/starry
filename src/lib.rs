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

#[macro_use]
extern crate lazy_static;

pub use client::HttpClient;
pub use http::header;
pub use http::header::content_type::Inner;
pub use http::header::Header;
pub use http::method::Method;
pub use http::request::Request;
pub use http::requester::Requester;
pub use http::response::Response;
pub use http::status::Status;
pub use http::url::URL;
pub use http::values::MultipartValues;
pub use http::values::Values;
pub use http::version::Version;
pub use server::Context;
pub use server::Extend;
pub use server::HttpServer;
pub use server::limit::Limit;

mod server;
mod http;
pub mod utils;
mod client;
