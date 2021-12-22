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

use crate::{Context, Status};
pub use crate::server::limit::Limit;

/// 过滤器/拦截器处理
///
/// 过滤操作尽量不要对数据体里的信息进行校验之类的流程，最好是对path、header和cookie进行过滤
///
/// ctx 请求处理上下文结构
pub(crate) type Filter = fn(context: &mut Context);

/// 降级服务
///
/// 当服务handler出现不可逆转的错误时，通过该方案进行补偿
///
/// 最终会让用户体验到的是某些功能暂时不可用，但不会是不受控制的返回信息
pub(crate) type Downgrade = fn(context: &mut Context);

#[derive(Clone)]
pub struct Extend {
    /// 过滤器/拦截器集合
    pub(crate) filters: Vec<Filter>,
    /// 限流策略
    pub(crate) limit: Option<Limit>,
    /// 降级服务
    pub(crate) downgrade: Option<Downgrade>,
}

impl Extend {
    /// 扩展生成方法
    ///
    /// 只有过滤
    pub fn e1(filters: Vec<Filter>) -> Extend {
        Extend { filters, limit: None, downgrade: None }
    }

    /// 扩展生成方法
    ///
    /// 只有限流
    pub fn e2(limit: Limit) -> Extend {
        Extend { filters: vec![], limit: Some(limit), downgrade: None }
    }

    /// 扩展生成方法
    ///
    /// 有限流，有过滤
    pub fn e3(filters: Vec<Filter>, limit: Limit) -> Extend {
        Extend { filters, limit: Some(limit), downgrade: None }
    }

    /// 扩展生成方法
    ///
    /// 有过滤，有降级
    pub(crate) fn e4(filters: Vec<Filter>, downgrade: Option<Downgrade>) -> Extend {
        Extend { filters, limit: None, downgrade }
    }

    pub(crate) fn copy1(&self) -> Self {
        Extend { filters: self.filters.clone(), limit: self.limit.clone(), downgrade: self.downgrade.clone() }
    }

    pub(crate) fn copy2(&self, filters: Vec<Filter>, downgrade: Option<Downgrade>) -> Self {
        Extend { filters, limit: self.limit.clone(), downgrade }
    }

    /// 扩展执行
    ///
    /// 自我诊断，先执行限流，再执行过滤
    pub(crate) fn exec(&self, context: &mut Context) {
        if self.check_limit(context) {
            self.check_filter(context)
        }
    }

    /// 检查限流
    ///
    /// 返回是否允许继续执行
    fn check_limit(&self, context: &mut Context) -> bool {
        match self.limit.clone() { // 检查限流
            Some(src) => match src.recv() {
                Ok(_) => {
                    log::trace!("http server extend limit run receive success!");
                    true
                }
                Err(err) => { // 限流异常
                    context.resp_status(Status::FORBIDDEN);
                    context.response();
                    log::error!("http server extend limit run receive failed! {}", err);
                    false
                }
            }
            None => true
        }
    }

    /// 检查过滤
    fn check_filter(&self, context: &mut Context) {
        for filter in self.filters.clone() {
            // 如果执行返回，则后续遍历结束
            if context.executed {
                break;
            }
            filter(context);
        }
    }
}

impl Debug for Extend {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "filter count: {:#?}, \nlimit: {:#?}", self.filters.len(), self.limit)
    }
}

