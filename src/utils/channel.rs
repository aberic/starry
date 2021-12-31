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

use crossbeam::channel::{Receiver, Sender};

use crate::utils::errors::{Errs, StarryResult};

#[derive(Clone, Debug)]
pub(crate) struct Channel<T> {
    /// 线程可用状态跨线程通信发送机制，当新建线程时，需要作为线程参数传入
    tx: Sender<T>,
    /// 线程可用状态跨线程通信接收机制
    rx: Receiver<T>,
}

impl<T> Channel<T> {
    /// 新建指定容量通道
    pub(crate) fn bounded(count: usize) -> Self {
        let (tx, rx) = crossbeam::channel::bounded(count);
        Channel { tx, rx }
    }

    /// 新建不限容量通道
    pub(crate) fn unbounded() -> Self {
        let (tx, rx) = crossbeam::channel::unbounded();
        Channel { tx, rx }
    }

    /// 尝试在不阻塞的情况下将消息发送到信道。
    /// 此方法将立即向通道发送消息，或者在通道已满或断开连接时返回错误。返回的错误包含原始消息。
    /// 如果在零容量信道上调用，此方法将只在信道的另一边碰巧同时有接收操作时才发送消息。
    pub(crate) fn try_send(&self, t: T) -> StarryResult<()> {
        match self.tx.try_send(t) {
            Ok(_) => Ok(()),
            Err(err) => Err(Errs::string(err.to_string()))
        }
    }

    /// 阻塞当前线程，直到消息发送或通道断开。
    /// 如果通道已满且未断开，此调用将阻塞，直到发送操作可以继续。如果通道断开，这个调用将被唤醒并返回一个错误。
    /// 返回的错误包含原始消息。
    /// 如果在零容量信道上调用，此方法将等待接收操作出现在信道的另一边。
    pub(crate) fn send(&self, t: T) -> StarryResult<()> {
        match self.tx.send(t) {
            Ok(_) => Ok(()),
            Err(err) => Err(Errs::string(err.to_string()))
        }
    }

    /// 阻塞当前线程，直到收到消息或通道为空并断开连接。
    /// 如果通道为空且未断开连接，则此调用将阻塞，直到接收操作可以继续。
    /// 如果通道是空的并且断开连接，这个调用将被唤醒并返回一个错误。
    /// 如果在一个零容量的信道上被调用，这个方法将等待一个发送操作出现在信道的另一边。
    pub(crate) fn recv(&self) -> StarryResult<T> {
        match self.rx.recv() {
            Ok(src) => Ok(src),
            Err(err) => Err(Errs::err(err))
        }
    }

    /// 尝试不阻塞地从信道接收消息。
    /// 此方法将立即从通道接收消息，如果通道为空则返回一个错误。
    /// 如果在零容量信道上调用，此方法将仅在信道的另一边碰巧同时有发送操作时才接收消息。
    pub(crate) fn try_recv(&self) -> StarryResult<T> {
        match self.rx.try_recv() {
            Ok(src) => Ok(src),
            Err(err) => Err(Errs::err(err))
        }
    }
}

