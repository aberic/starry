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

use std::sync::{Arc, Mutex};
use std::time::Duration;

use crossbeam::channel::{Receiver, Sender};

use crate::utils::concurrent::Thread;
use crate::utils::errors::{Errs, StarryResult};
use crate::utils::Time;

#[derive(Clone, Debug)]
struct Channel {
    /// 线程可用状态跨线程通信发送机制，当新建线程时，需要作为线程参数传入
    tx: Sender<()>,
    /// 线程可用状态跨线程通信接收机制
    rx: Receiver<()>,
}

impl Channel {
    pub fn new(count: usize) -> Self {
        let (tx, rx) = crossbeam::channel::bounded(count);
        Channel { tx, rx }
    }

    fn send(&self) {
        match self.tx.send(()) {
            Ok(_) => {}
            Err(err) => log::error!("request limit fetch error! {}", err)
        }
    }

    fn recv(&self) -> StarryResult<()> {
        match self.rx.recv() {
            Ok(_) => Ok(()),
            Err(err) => Err(Errs::err(err))
        }
    }
}

/// 限流策略
#[derive(Clone, Debug)]
pub struct Limit {
    /// 请求限定的时间段/区间（毫秒）
    section: i64,
    /// 请求限定的时间段内允许的请求次数
    count: usize,
    /// 请求限定的时间段内允许的请求次数 - 1
    count_sub_1: usize,
    /// 请求允许的最小间隔时间（毫秒），0表示不限
    interval: i64,
    /// 限流通道
    channel: Arc<Channel>,
    /// 请求时间数组
    times: Arc<Mutex<Vec<i64>>>,
}

impl Limit {
    /// 新建限流策略
    ///
    /// * section 请求限定的时间段/区间（毫秒），小于等于0表示不限
    /// * count 请求限定的时间段内允许的请求次数
    /// * interval 请求允许的最小间隔时间（毫秒），小于等于0表示不限
    pub fn new(section: i64, count: usize, interval: i64) -> Self {
        let channel = Arc::new(Channel::new(count));
        let mut times = vec![];
        let un_interval =
            if interval <= 0 {
                0
            } else {
                interval as u64
            };
        for _ in 0..count {
            times.push(Time::now().num_milliseconds());
            Thread::sleep(Duration::from_millis(un_interval))
        }
        Limit {
            section,
            count,
            count_sub_1: count - 1,
            interval,
            channel,
            times: Arc::new(Mutex::new(times)),
        }
    }

    pub(crate) fn run(&self) {
        loop {
            let time_now = Time::now().num_milliseconds();
            // 如果当前时间与时间数组第一时间差大于限定时间段，并且当前时间与时间数组最后时间差大于最小请求间隔，则放行新的请求
            let mut times = self.times.lock().unwrap();
            if time_now - times[0] > self.section && time_now - times[self.count_sub_1] > self.interval {
                // 发送一个元素，放行本次请求
                self.channel.send();
                // 重置时间集合
                times.remove(0);
                times.push(time_now)
            }
        }
    }

    pub(crate) fn recv(&self) -> StarryResult<()> {
        self.channel.recv()
    }
}

#[cfg(test)]
mod limit_test {
    use crate::server::limit::{Channel, Limit};
    use crate::utils::concurrent::Thread;

    impl Channel {
        fn len(&self) -> (usize, usize) {
            (self.tx.len(), self.rx.len())
        }
    }

    impl Limit {
        fn len(&self) -> (usize, usize) {
            self.channel.len()
        }
    }

    #[test]
    fn loops() {
        let l = Limit::new(1000, 5, 100);
        let mut l_c = l.clone();
        Thread::spawn(move || l_c.run()).unwrap();
        test_loop(l);
    }

    fn test_loop(limit: Limit) {
        for i in 0..20 {
            let limit_c = limit.clone();
            Thread::spawn(move || {
                println!("被堵住了 {} channel tx len = {}, rx len = {}", i, limit_c.len().0, limit_c.len().1);
                match limit_c.recv() {
                    Ok(_) => println!("OK!"),
                    Err(err) => println!("err = {}", err)
                }
                println!("被放行了 {} channel tx len = {}, rx len = {}", i, limit_c.len().0, limit_c.len().1);
            }).unwrap().join().unwrap();
        }
    }
}
