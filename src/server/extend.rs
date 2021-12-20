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

use std::fmt::{Debug, Formatter};

use crate::Context;
pub use crate::server::limit::Limit;

/// 过滤器/拦截器处理
///
/// 过滤操作尽量不要对数据体里的信息进行校验之类的流程，最好是对path、header和cookie进行过滤
///
/// ctx 请求处理上下文结构
pub(crate) type Filter = fn(context: &mut Context);

#[derive(Clone)]
pub struct Extend {
    /// 过滤器/拦截器数组
    pub(crate) filters: Vec<Filter>,
    pub(crate) limit: Option<Limit>,
}

impl Extend {
    pub fn e1(filters: Vec<Filter>) -> Extend {
        Extend { filters, limit: None }
    }

    pub fn e2(limit: Limit) -> Extend {
        Extend { filters: vec![], limit: Some(limit) }
    }

    pub fn e3(filters: Vec<Filter>, limit: Limit) -> Extend {
        Extend { filters, limit: Some(limit) }
    }

    pub(crate) fn copy(&self) -> Self {
        Extend { filters: self.filters.clone(), limit: self.limit.clone() }
    }

    pub(crate) fn copy_with(&self, filters: Vec<Filter>) -> Self {
        Extend { filters, limit: self.limit.clone() }
    }

    /// 扩展执行
    ///
    /// 自我诊断，先执行限流，再执行过滤
    pub(crate) fn run(&self, context: &mut Context) {
        match self.limit.clone() {
            Some(src) => match src.recv() {
                Ok(_) => {
                    log::trace!("http server extend limit run receive success!");
                    for filter in self.filters.clone() {
                        filter(context);
                        // 如果执行返回，则后续遍历结束
                        if context.executed {
                            break
                        }
                    }
                }
                Err(err) => {
                    context.executed = true;
                    log::error!("http server extend limit run receive failed! {}", err)
                }
            }
            None => for filter in self.filters.clone() {
                filter(context);
                // 如果执行返回，则后续遍历结束
                if context.executed {
                    break
                }
            }
        }
    }
}

impl Debug for Extend {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "filter_len: {:#?}, \nlimit: {:#?}", self.filters.len(), self.limit)
    }
}

