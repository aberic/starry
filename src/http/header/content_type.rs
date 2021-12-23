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

#[derive(Clone, PartialEq, Eq)]
pub enum Inner {
    // 常见的媒体格式类型如下
    /// text/html： HTML格式
    TextHtml,
    /// text/plain：纯文本格式
    TextPlain,
    /// text/xml： XML格式
    TextXml,
    /// image/gif：gif图片格式
    ImageGif,
    /// image/jpeg：jpg图片格式
    ImageJpeg,
    /// image/png：png图片格式
    ImagePng,
    // 以application开头的媒体格式类型：
    /// application/xhtml+xml：XHTML格式
    ApplicationXHtmlXml,
    /// application/xml： XML数据格式
    ApplicationXml,
    /// application/atom+xml：Atom XML聚合格式
    ApplicationAtomXml,
    /// application/json： JSON数据格式
    ApplicationJson,
    /// application/pdf：pdf格式
    ApplicationPdf,
    /// application/msword：Word文档格式
    ApplicationMsWord,
    /// application/octet-stream： 二进制流数据（如常见的文件下载）
    ApplicationOctetStream,
    /// application/x-www-form-urlencoded： \<form encType=””\>中默认的encType，
    /// form表单数据被编码为key/value格式发送到服务器（表单默认的提交数据的格式）
    ApplicationXWWWFormUrlEncoded,
    // 另外一种常见的媒体格式是上传文件之时使用的：
    /// multipart/form-data： 需要在表单中进行文件上传时，就需要使用该格式
    MultipartFormData(String),
    /// 自定义类型，一般是自行补充未定义类型
    CustomType(String),
}

impl Inner {
    /// 用`&str`表示当前Type
    pub fn as_str(&self) -> &str {
        match self {
            Inner::TextHtml => "text/html",
            Inner::TextPlain => "text/plain",
            Inner::TextXml => "text/xml",
            Inner::ImageGif => "image/gif",
            Inner::ImageJpeg => "image/jpeg",
            Inner::ImagePng => "image/png",
            Inner::ApplicationXHtmlXml => "application/xhtml+xml",
            Inner::ApplicationXml => "application/xml",
            Inner::ApplicationAtomXml => "application/atom+xml",
            Inner::ApplicationJson => "application/json",
            Inner::ApplicationPdf => "application/pdf",
            Inner::ApplicationMsWord => "application/msword",
            Inner::ApplicationOctetStream => "application/octet-stream",
            Inner::ApplicationXWWWFormUrlEncoded => "application/x-www-form-urlencoded",
            Inner::MultipartFormData(_) => "multipart/form-data",
            Inner::CustomType(src) => src.as_str()
        }
    }
}

/// ContentType 当消息中包含实体体时，该实体体的数据类型通过报头字段Content-Type和Content-Encoding确定。
/// 这些定义了一个两层的有序编码模型:
/// entity-body:= Content-Encoding(Content-Type(data))
///
/// Content-Type指定基础数据的媒体类型。
/// 内容编码可以用来表示应用于数据的任何附加内容编码，通常是为了数据压缩的目的，这些内容编码是被请求资源的一个属性。
/// 没有默认编码。
///
/// 任何包含实体体的HTTP/1.1消息都应该包含一个Content-Type报头字段，该字段定义了实体体的媒体类型。
/// 当且仅当媒体类型不是由content-type字段给出时，接收方可以通过检查其内容和/或用于标识资源的URI的名称扩展名来猜测
/// 媒体类型。如果媒体类型仍然未知，接收方应该将其视为类型“application/octet-stream”。
///
/// 目前ContentType仅对常用text、image、application及form类型做了枚举
#[derive(Clone, PartialEq, Eq)]
pub struct ContentType(Inner);

impl ContentType {
    /// text/html： HTML格式
    pub const TEXT_HTML: ContentType = ContentType(Inner::TextHtml);
    /// text/plain：纯文本格式
    pub const TEXT_PLAIN: ContentType = ContentType(Inner::TextPlain);
    /// text/xml： XML格式
    pub const TEXT_XML: ContentType = ContentType(Inner::TextXml);
    /// image/gif：gif图片格式
    pub const IMAGE_GIF: ContentType = ContentType(Inner::ImageGif);
    /// image/jpeg：jpg图片格式
    pub const IMAGE_JPEG: ContentType = ContentType(Inner::ImageJpeg);
    /// image/png：png图片格式
    pub const IMAGE_PNG: ContentType = ContentType(Inner::ImagePng);
    /// application/xhtml+xml：XHTML格式
    pub const APPLICATION_XHTML_XML: ContentType = ContentType(Inner::ApplicationXHtmlXml);
    /// application/xml： XML数据格式
    pub const APPLICATION_XML: ContentType = ContentType(Inner::ApplicationXml);
    /// application/atom+xml：Atom XML聚合格式
    pub const APPLICATION_ATOM_XML: ContentType = ContentType(Inner::ApplicationAtomXml);
    /// application/json： JSON数据格式
    pub const APPLICATION_JSON: ContentType = ContentType(Inner::ApplicationJson);
    /// application/pdf：pdf格式
    pub const APPLICATION_PDF: ContentType = ContentType(Inner::ApplicationPdf);
    /// application/msword：Word文档格式
    pub const APPLICATION_MS_WORD: ContentType = ContentType(Inner::ApplicationMsWord);
    /// application/octet-stream： 二进制流数据（如常见的文件下载）
    pub const APPLICATION_OCTET_STREAM: ContentType = ContentType(Inner::ApplicationOctetStream);
    /// application/x-www-form-urlencoded： \<form encType=””\>中默认的encType，
    /// form表单数据被编码为key/value格式发送到服务器（表单默认的提交数据的格式）
    pub const APPLICATION_X_WWW_FORM_URL_ENCODED: ContentType = ContentType(Inner::ApplicationXWWWFormUrlEncoded);
    // /// multipart/form-data： 需要在表单中进行文件上传时，就需要使用该格式
    // pub const MULTIPART_FORM_DATA: ContentType = ContentType(Inner::MultipartFormData);

    /// multipart/form-data： 需要在表单中进行文件上传时，就需要使用该格式
    pub fn multipart_form_data(src: String) -> ContentType {
        ContentType(Inner::MultipartFormData(src))
    }

    /// multipart/form-data： 需要在表单中进行文件上传时，就需要使用该格式
    pub fn multipart_form_data_str(src: &str) -> ContentType {
        ContentType(Inner::MultipartFormData(src.to_string()))
    }

    pub fn custom(src: String) -> ContentType {
        ContentType(Inner::CustomType(src))
    }

    pub fn custom_str(src: &str) -> ContentType {
        ContentType(Inner::CustomType(src.to_string()))
    }

    pub fn default() -> ContentType {
        ContentType::APPLICATION_OCTET_STREAM
    }

    pub fn from_str(src: &str) -> ContentType {
        match src {
            "text/html" => ContentType::TEXT_HTML,
            "text/plain" => ContentType::TEXT_PLAIN,
            "text/xml" => ContentType::TEXT_XML,
            "image/gif" => ContentType::IMAGE_GIF,
            "image/jpeg" => ContentType::IMAGE_JPEG,
            "image/png" => ContentType::IMAGE_PNG,
            "application/xhtml+xml" => ContentType::APPLICATION_XHTML_XML,
            "application/xml" => ContentType::APPLICATION_XML,
            "application/atom+xml" => ContentType::APPLICATION_ATOM_XML,
            "application/json" => ContentType::APPLICATION_JSON,
            "application/pdf" => ContentType::APPLICATION_PDF,
            "application/msword" => ContentType::APPLICATION_MS_WORD,
            "application/x-www-form-urlencoded" => ContentType::APPLICATION_X_WWW_FORM_URL_ENCODED,
            _ => if src.len() > 30 && src[..30].eq("multipart/form-data; boundary=") {
                // multipart/form-data; boundary=--------------------------317111499693539547849948
                // multipart/form-data： 需要在表单中进行文件上传时，就需要使用该格式
                ContentType(Inner::MultipartFormData(src[30..].to_string()))
            } else {
                ContentType::default()
            },
        }
    }

    pub fn inner(&self) -> Inner {
        self.0.clone()
    }

    /// 用`&str`表示当前ContentType
    pub fn as_str(&self) -> &str {
        self.0.as_str()
    }
}

impl AsRef<str> for ContentType {
    fn as_ref(&self) -> &str {
        self.as_str()
    }
}

impl<'a> PartialEq<&'a ContentType> for ContentType {
    fn eq(&self, other: &&'a ContentType) -> bool {
        self == *other
    }
}

impl<'a> PartialEq<ContentType> for &'a ContentType {
    fn eq(&self, other: &ContentType) -> bool {
        *self == other
    }
}

impl PartialEq<str> for ContentType {
    fn eq(&self, other: &str) -> bool {
        self.as_ref() == other
    }
}

impl PartialEq<ContentType> for str {
    fn eq(&self, other: &ContentType) -> bool {
        self == other.as_ref()
    }
}

impl<'a> PartialEq<&'a str> for ContentType {
    fn eq(&self, other: &&'a str) -> bool {
        self.as_ref() == *other
    }
}

impl<'a> PartialEq<ContentType> for &'a str {
    fn eq(&self, other: &ContentType) -> bool {
        *self == other.as_ref()
    }
}

impl fmt::Debug for ContentType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_ref())
    }
}

#[cfg(test)]
mod content_type_test {
    use crate::header::ContentType;

    #[test]
    fn test1() {
        let ct1 = ContentType::TEXT_HTML;
        let ct2 = ContentType::custom_str("abc/xyz");

        assert_eq!("text/html", ct1.as_str());
        assert_eq!("abc/xyz", ct2.as_str());
    }
}