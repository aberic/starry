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

//! 一个用于并行执行函数的线程池工具
//!  
//! 可以生成指定数量的工作线程，并在任何工作线程出现故障时将线程池补充至指定阈
//!
//! 线程池会维护一个线程全量数组，一个可用线程队列以及一个有界非阻塞任务队列
//!
//! 当有新任务进入时，会提取可用线程去执行，如果没有，则放入任务队列
//!
//! 当有线程执行完任务后，会提取待执行任务，如果没有，则放入线程队列
//!
//! # 案例
//!
//! ## 默认方法
//! ```
//! use starry::utils::concurrent::ThreadPool;
//!
//! let thread_pool = ThreadPool::default().unwrap();
//! thread_pool.execute(|| println!("1")).unwrap();
//! ```
//!
//! ## 自定义线程池方法
//! ```
//! use std::thread;
//! use starry::utils::concurrent::ThreadPool;
//!
//! let mut thread_pool_builder = ThreadPool::builder();
//! thread_pool_builder.pool_size(100);
//! thread_pool_builder.name_prefix("test");
//! thread_pool_builder.task_count(4000);
//! let thread_pool = thread_pool_builder.create().unwrap();
//! thread_pool.execute(|| println!("1")).unwrap();
//! ```
//!
//! 线程池可维护待处理任务数量根据实际情况设置，设定为满足极限情况可最大程度保证不用进行额外的失败处理，当任务队列
//! 放满后，会返回失败，在使用线程池执行任务时，应处理该失败信息

use std::fmt;
use std::sync::{Arc, RwLock};
use std::{cmp, thread};

use crossbeam::atomic::AtomicCell;
use crossbeam::channel::{Receiver, Sender};
use crossbeam::queue::ArrayQueue;
use crate::utils::concurrent::Thread;
use crate::utils::errors::{Errs, StarryResult};

/// 一个通用的线程池，用于调度轮询[`Task`]以完成任务
///
/// 线程池将任意数量的任务多路复用到固定数量的工作线程上
///
/// 这个类型是线程池本身的可克隆句柄。克隆它只会创建一个新的引用，而不是一个新的线程池
///
/// 使用`Builder`创建`ThreadPool`实例
pub struct ThreadPool {
    /// 线程池状态
    pool_state: Arc<PoolState>,
    /// 线程数组
    threads: Arc<RwLock<Vec<Worker>>>,
    /// 线程可用状态跨线程通信发送机制，当新建线程时，需要作为线程参数传入
    tx_notify: Sender<Notify>,
    /// 线程可用状态跨线程通信接收机制
    rx_notify: Receiver<Notify>,
    /// 空闲线程队列
    worker_idle_queue: Arc<ArrayQueue<Worker>>,
    /// 待执行任务队列
    task_queue: Arc<ArrayQueue<Task<'static>>>,
}

impl ThreadPool {
    /// 用默认值创建一个新的`ThreadPool`
    ///
    /// 使用`Builder`创建已配置的线程池
    ///
    /// 有关默认配置的详细信息，请参阅[`Builder`](Builder)中的方法文档
    ///
    /// # 例子
    ///
    /// 创建一个新的线程池：
    ///
    /// ```rust
    /// use starry::utils::concurrent::ThreadPool;
    ///
    /// let pool = ThreadPool::default().unwrap();
    /// ```
    pub fn default() -> StarryResult<Self> {
        Builder::new().create()
    }

    /// 创建一个能够并发执行`size`数量的任务的新线程池
    ///
    /// # panics
    ///
    /// 如果size == 0，则会出现panic
    ///
    /// # 案例
    ///
    /// 创建一个新的线程池，可以同时执行四个任务:
    ///
    /// ```rust
    /// use starry::utils::concurrent::ThreadPool;
    ///
    /// let pool = ThreadPool::new(4).unwrap();
    /// ```
    pub fn new(size: usize) -> StarryResult<Self> {
        Builder::new().pool_size(size).create()
    }

    /// 创建一个能够并发执行`size`数量的任务的新线程池，每个线程都有`name_prefix`作为名称前缀
    ///
    /// # panics
    ///
    /// 如果size == 0，则会出现panic
    ///
    /// # 案例
    ///
    /// 创建一个新的线程池，可以同时执行四个任务:
    ///
    /// ```rust
    /// use starry::utils::concurrent::ThreadPool;
    ///
    /// let pool = ThreadPool::new_with_name("test".to_string(), 4).unwrap();
    /// ```
    pub fn new_with_name(name_prefix: String, size: usize) -> StarryResult<Self> {
        Builder::new()
            .name_prefix(name_prefix)
            .pool_size(size)
            .create()
    }

    /// 创建一个能够并发执行`size`数量的任务的新线程池，每个线程都有`name_prefix`作为名称前缀，每个线程的堆栈大小
    /// 为`stack_size`字节，可维护待处理任务数量为`task_count`
    ///
    /// # panics
    ///
    /// 如果size == 0，则会出现panic
    ///
    /// # 案例
    ///
    /// 创建一个新的线程池，可以同时执行四个任务:
    ///
    /// ```rust
    /// use starry::utils::concurrent::ThreadPool;
    ///
    /// let pool = ThreadPool::new_custom("test".to_string(), 4, 8_000_000, 100).unwrap();
    /// ```
    pub fn new_custom(
        name_prefix: String,
        size: usize,
        stack_size: usize,
        task_count: usize,
    ) -> StarryResult<Self> {
        Builder::new()
            .name_prefix(name_prefix)
            .pool_size(size)
            .stack_size(stack_size)
            .task_count(task_count)
            .create()
    }

    /// 创建一个默认的线程池配置，可以继续对其进行定制
    ///
    /// 有关默认配置的详细信息，请参阅[`Builder`](Builder)中的方法文档
    pub fn builder() -> Builder {
        Builder::new()
    }

    /// 线程池初始化方法
    ///
    /// 该方法会创建一个通知检查线程，详见`notify_check`方法，便于监听任务队列和可用线程队列相关事件
    fn init(
        pool_state: Arc<PoolState>,
        threads: Arc<RwLock<Vec<Worker>>>,
        worker_idle_queue: Arc<ArrayQueue<Worker>>,
        task_count: usize,
        tx_notify: Sender<Notify>,
        rx_notify: Receiver<Notify>,
    ) -> StarryResult<Self> {
        let task_queue = Arc::new(ArrayQueue::new(task_count));
        let pool = ThreadPool {
            pool_state,
            threads,
            tx_notify,
            rx_notify,
            worker_idle_queue,
            task_queue,
        };
        let pool_c = pool.clone();
        match Thread::spawn_on_name(format!("{}pool", pool.pool_state.name_prefix), move || {
            pool_c.notify_check()
        }) {
            Err(err) => return Err(Errs::strs("thread pool init spawn", err)),
            _ => Ok(pool),
        }
    }

    /// 通知检查
    fn notify_check(&self) {
        loop {
            match self.rx_notify.recv() {
                Ok(res) => match res {
                    Notify::Fill(counter) => {
                        // 创建新线程补充
                        let mut tsw = self.threads.write().unwrap();
                        tsw.remove(counter);
                        let pool_state = self.pool_state.clone();
                        let worker =
                            Worker::new(counter, self.tx_notify.clone(), pool_state).unwrap();
                        self.insert(counter, worker);
                        self.pool_state.active_count.fetch_add(1);
                    }
                    Notify::Idle(counter) => {
                        let threads_r = self.threads.read().unwrap();
                        let thread = threads_r.get(counter).unwrap();
                        match self.task_queue.pop() {
                            Some(res) => thread.send(Message::Run(res)),
                            None => {
                                match self.worker_idle_queue.push(thread.clone()) {
                                    Err(_) => panic!("notify check error! the thread idle queue is full, the element can not push into!"),
                                    _ => {}
                                }
                            }
                        }
                    }
                    Notify::Close => break,
                },
                _ => {}
            }
        }
    }

    /// 在线程池指定下标插入新线程
    fn insert(&self, index: usize, thread: Worker) {
        self.threads.write().unwrap().insert(index, thread)
    }

    /// 在线程池中的线程上执行给定函数
    ///
    /// 当线程池任务队列已满，新任务无法放入，则返回该错误信息
    ///
    /// # 案例
    ///
    /// 在一个可以并发运行两个任务的线程池上执行四个任务
    ///
    /// ```
    /// use starry::utils::concurrent::ThreadPool;
    ///
    /// let pool = ThreadPool::new(2).unwrap();
    /// pool.execute(|| println!("hello"));
    /// pool.execute(|| println!("world"));
    /// pool.execute(|| println!("foo"));
    /// pool.execute(|| println!("bar"));
    /// ```
    pub fn execute<F>(&self, f: F) -> StarryResult<()>
    where
        F: FnOnce() + Send + 'static,
    {
        let task = Box::new(f);
        match self.worker_idle_queue.pop() {
            Some(res) => {
                res.send(Message::Run(task));
                Ok(())
            }
            None => match self.task_queue.push(task) {
                Err(_) => {
                    return Err(Errs::str(
                        "the task queue is full, the element can not push into!",
                    ))
                }
                _ => Ok(()),
            },
        }
    }

    /// 计划的线程池中工作线程的数量
    pub fn size(&self) -> usize {
        self.pool_state.size
    }
}

impl Clone for ThreadPool {
    fn clone(&self) -> Self {
        self.pool_state.count.fetch_add(1);
        Self {
            pool_state: self.pool_state.clone(),
            threads: self.threads.clone(),
            tx_notify: self.tx_notify.clone(),
            rx_notify: self.rx_notify.clone(),
            worker_idle_queue: self.worker_idle_queue.clone(),
            task_queue: self.task_queue.clone(),
        }
    }
}

impl Drop for ThreadPool {
    fn drop(&mut self) {
        if self.pool_state.count.fetch_sub(1) == 0 {
            let threads_r = self.threads.write().unwrap();
            for thread in threads_r.iter() {
                thread.send(Message::Close);
            }
            self.tx_notify.send(Notify::Close).unwrap();
        }
    }
}

impl fmt::Debug for ThreadPool {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_struct("ThreadPool")
            .field("pool_state", &self.pool_state)
            .field("threads", &self.threads)
            .field("tx_notify", &self.tx_notify)
            .field("rx_notify", &self.rx_notify)
            .field("thread_idle_queue", &self.worker_idle_queue)
            .field("task_queue", &self.task_queue)
            .finish()
    }
}

/// [`ThreadPool`]工厂，可以用来配置[`ThreadPool`]的属性。
///
/// 目前至少有四种配置选项可用:
///
/// * `pool_size` 构建的[`ThreadPool`]在任何给定时刻将活跃的最大线程数
/// * `name_prefix` 由构建的[`ThreadPool`]生成的每个线程名前缀，如果前缀是`my-pool-`，那么池中的线程将获得类似`my-pool-1`这样的名称
/// * `stack_size` 由构建的[`ThreadPool`]生成的每个线程的堆栈大小(以字节为单位)
/// * `task_count` 由构建的[`ThreadPool`]生成的可维护待处理任务数量，超过该数值的任务会被放弃，并返回调用者失败，默认100
///
/// ## 案例
///
/// 建立一个[`ThreadPool`]，最多同时使用16个线程，每个线程有8mb的堆栈大小，每个线程名称前缀为`test`，任务队列上限4000:
///
/// ```
/// use std::thread;
/// use starry::utils::concurrent::ThreadPool;
///
/// let mut thread_pool_builder = ThreadPool::builder();
/// thread_pool_builder.pool_size(16);
/// thread_pool_builder.stack_size(8_000_000);
/// thread_pool_builder.name_prefix("test");
/// thread_pool_builder.task_count(4000);
/// let thread_pool = thread_pool_builder.create().unwrap();
/// thread_pool.execute(|| println!("1")).unwrap();
/// ```
pub struct Builder {
    /// 线程池中工作线程的数量
    pool_size: usize,
    /// 线程池中线程的堆栈大小
    stack_size: usize,
    /// 线程池的线程名前缀，如果前缀是`my-pool-`，那么池中的线程将获得类似`my-pool-1`这样的名称
    name_prefix: String,
    /// 线程池可维护待处理任务数量，超过该数值的任务会被放弃，并返回调用者失败，默认100
    task_count: usize,
}

impl Builder {
    /// 创建默认线程池配置
    ///
    /// 有关默认值的详细信息，请参阅该类型的其他方法
    pub fn new() -> Self {
        Self {
            pool_size: cmp::max(1, num_cpus::get()),
            stack_size: 0,
            name_prefix: String::from("george-thread-pool-"),
            task_count: 100,
        }
    }

    /// 设置线程池的大小
    ///
    /// 线程池的大小是生成的工作线程的数量。默认情况下，等于CPU核数
    ///
    /// # Panics
    ///
    /// 如果`size == 0`，则会出现panic
    pub fn pool_size(&mut self, size: usize) -> &mut Self {
        assert!(size > 0);
        self.pool_size = size;
        self
    }

    /// 设置线程池中线程的堆栈大小，以字节为单位
    ///
    /// 默认情况下，工作线程使用`Rust`的标准堆栈大小
    pub fn stack_size(&mut self, stack_size: usize) -> &mut Self {
        self.stack_size = stack_size;
        self
    }

    /// 设置线程池的线程名前缀
    ///
    /// 线程名前缀用于生成线程名。例如，如果前缀是`my-pool-`，那么池中的线程将获得类似`my-pool-1`这样的名称
    ///
    /// 默认情况下，工作线程被分配`Rust`的标准线程名
    pub fn name_prefix<S: Into<String>>(&mut self, name_prefix: S) -> &mut Self {
        self.name_prefix = name_prefix.into();
        self
    }

    /// 设置线程池可维护待处理任务数量
    pub fn task_count(&mut self, task_count: usize) -> &mut Self {
        self.task_count = task_count;
        self
    }

    /// 通过已有的配置创建一个[`ThreadPool`](ThreadPool)
    pub fn create(&mut self) -> StarryResult<ThreadPool> {
        let (tx_notify, rx_notify) = crossbeam::channel::unbounded();
        let pool_state = Arc::new(PoolState::new(
            self.pool_size,
            self.stack_size,
            self.name_prefix.clone(),
            AtomicCell::new(1),
        ));
        let worker_idle_queue = Arc::new(ArrayQueue::new(self.pool_size));
        let workers = Arc::new(RwLock::new(vec![]));
        let workers_c = workers.clone();
        let mut workers_w = workers_c.write().unwrap();
        for counter in 0..self.pool_size {
            let state = pool_state.clone();
            let worker = Worker::new(counter, tx_notify.clone(), state.clone())?;
            workers_w.push(worker.clone());
            match worker_idle_queue.push(worker) {
                Err(_) => {
                    return Err(Errs::str(
                        "the thread idle queue is full, the element can not push into!",
                    ))
                }
                _ => {}
            }
        }
        ThreadPool::init(
            pool_state,
            workers,
            worker_idle_queue,
            self.task_count,
            tx_notify,
            rx_notify,
        )
    }
}

/// 线程池通知机制
enum Notify {
    /// 填充新线程到线程数组指定下标
    Fill(usize),
    /// 告知线程数组指定下标线程当前为闲置状态
    Idle(usize),
    Close,
}

/// 待执行任务
enum Message {
    Run(Task<'static>),
    Close,
}

/// 线程池状态
struct PoolState {
    /// 计划的线程池中工作线程的数量
    size: usize,
    /// 线程池中线程的堆栈大小
    stack_size: usize,
    /// 线程池的线程名前缀，如果前缀是`my-pool-`，那么池中的线程将获得类似`my-pool-1`这样的名称
    name_prefix: String,
    /// 线程池副本数量
    count: AtomicCell<usize>,
    /// 活跃的工作线程数量
    active_count: AtomicCell<usize>,
    /// 异常的工作线程数量
    panic_count: AtomicCell<usize>,
}

impl PoolState {
    fn new(size: usize, stack_size: usize, name_prefix: String, count: AtomicCell<usize>) -> Self {
        Self {
            size,
            stack_size,
            name_prefix,
            count,
            active_count: AtomicCell::new(size),
            panic_count: AtomicCell::new(0),
        }
    }
}

impl fmt::Debug for PoolState {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_struct("PoolState")
            .field("size", &self.size)
            .field("stack_size", &self.stack_size)
            .field("name_prefix", &self.name_prefix)
            .field("count", &self.count)
            .field("active_count", &self.active_count)
            .field("panic_count", &self.panic_count)
            .finish()
    }
}

pub trait FnBox {
    fn run(self: Box<Self>);
}

impl<F: FnOnce()> FnBox for F {
    fn run(self: Box<F>) {
        (*self)()
    }
}

pub type Task<'a> = Box<dyn FnBox + Send + 'a>;

impl fmt::Debug for Task<'static> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_struct("Task").finish()
    }
}

/// 线程哨兵
///
/// 当线程出现异常退出时，由`Sentinel`负责传递新建信息
struct Sentinel {
    /// 线程位于线程数组下标
    counter: usize,
    /// 线程存活状态
    active: bool,
    /// 线程可用状态跨线程通信发送机制
    tx_notify: Sender<Notify>,
    /// 待执行任务跨线程通信接收机制
    rx_execute: Receiver<Message>,
    /// 线程池状态
    pool_state: Arc<PoolState>,
}

impl Sentinel {
    /// 新建线程哨兵
    /// * counter 线程位于线程数组下标
    /// * pool_state 线程池状态
    /// * tx_notify 线程可用状态跨线程通信发送机制
    /// * rx_execute 待执行任务跨线程通信接收机制
    fn new(
        counter: usize,
        pool_state: Arc<PoolState>,
        tx_notify: Sender<Notify>,
        rx_execute: Receiver<Message>,
    ) -> Sentinel {
        // println!(
        //     "sentinel new thread::current() name() = {}, id() = {:#?}",
        //     thread::current().name().unwrap(),
        //     thread::current().id(),
        // );
        Sentinel {
            counter,
            active: true,
            tx_notify,
            rx_execute,
            pool_state,
        }
    }

    /// 取消并销毁当前`Sentinel`
    fn cancel(mut self) {
        self.active = false;
    }
}

impl Drop for Sentinel {
    fn drop(&mut self) {
        if self.active {
            self.pool_state.active_count.fetch_sub(1);
            if thread::panicking() {
                self.pool_state.panic_count.fetch_add(1);
            }
            // 创建新线程补充
            self.tx_notify.send(Notify::Fill(self.counter)).unwrap()
        }
        // else {
        //     println!("count = {}", self.pool_state.count.take());
        //     println!("active_count = {}", self.pool_state.active_count.take());
        //     println!("size = {}", self.pool_state.size);
        //     println!("stack_size = {}", self.pool_state.stack_size);
        // }
    }
}

/// 经过封装的线程管理对象
struct Worker {
    /// 线程名称
    name: String,
    /// 待执行任务跨线程通信发送机制
    tx_execute: Sender<Message>,
}

impl Worker {
    /// 新建经过封装的线程管理对象
    /// * counter 线程位于线程数组下标
    /// * tx_notify 线程可用状态跨线程通信发送机制
    /// * pool_state 线程池状态
    fn new(
        counter: usize,
        tx_notify: Sender<Notify>,
        pool_state: Arc<PoolState>,
    ) -> StarryResult<Self> {
        let (tx_execute, rx_execute) = crossbeam::channel::unbounded();
        let pool_state_c = pool_state.clone();
        match Thread::spawn_on_custom(
            format!("{}{}", pool_state.name_prefix, counter),
            pool_state.stack_size,
            move || Worker::work(Sentinel::new(counter, pool_state_c, tx_notify, rx_execute)),
        ) {
            Err(err) => return Err(Errs::strs("thread builder spawn", err)),
            _ => {}
        }
        Ok(Self {
            name: pool_state.name_prefix.clone(),
            tx_execute,
        })
    }

    fn work(sentinel: Sentinel) {
        let rx = sentinel.rx_execute.clone();
        loop {
            match rx.recv() {
                Ok(res) => match res {
                    Message::Run(task) => {
                        task.run();
                        sentinel
                            .tx_notify
                            .send(Notify::Idle(sentinel.counter))
                            .unwrap()
                    }
                    Message::Close => break,
                },
                _ => {}
            }
        }
        sentinel.cancel()
    }

    fn send(&self, msg: Message) {
        self.tx_execute.send(msg).unwrap()
    }
}

impl Clone for Worker {
    fn clone(&self) -> Self {
        Self {
            name: self.name.clone(),
            tx_execute: self.tx_execute.clone(),
        }
    }
}

impl fmt::Debug for Worker {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_struct("Thread")
            .field("name", &self.name)
            .field("tx_execute", &self.tx_execute)
            .finish()
    }
}


#[cfg(test)]
mod thread_pool_test {
    use std::thread;
    use std::time::Duration;
    use crate::utils::concurrent::ThreadPool;

    fn spawn1() {
        println!("spawn 1");
    }

    fn spawn2() {
        println!("spawn 2");
    }

    fn spawn3() {
        println!("spawn 3");
    }

    fn spawn_n(n: usize) {
        println!("spawn {}", n);
    }

    #[test]
    fn test_1() {
        let thread_pool = ThreadPool::default().unwrap();
        thread_pool.execute(|| spawn1()).unwrap();
        thread_pool.execute(|| spawn2()).unwrap();
        thread::sleep(Duration::from_secs(1));
        thread_pool.execute(|| spawn3()).unwrap();
        thread::sleep(Duration::from_secs(1));
    }

    #[test]
    fn test_2() {
        let mut thread_pool_builder = ThreadPool::builder();
        thread_pool_builder.pool_size(100);
        thread_pool_builder.name_prefix("test");
        let thread_pool = thread_pool_builder.create().unwrap();
        thread_pool.execute(|| spawn1()).unwrap();
        thread_pool.execute(|| spawn2()).unwrap();
        thread::sleep(Duration::from_secs(1));
        thread_pool.execute(|| spawn3()).unwrap();
        thread::sleep(Duration::from_secs(1));
    }

    #[test]
    fn test_3() {
        let thread_pool = ThreadPool::default().unwrap();
        for n in 0..100 {
            thread_pool.execute(move || spawn_n(n)).expect("full!");
        }
        thread::sleep(Duration::from_secs(10));
    }

    #[test]
    fn test_4() {
        let mut thread_pool_builder = ThreadPool::builder();
        thread_pool_builder.task_count(4000);
        let thread_pool = thread_pool_builder.create().unwrap();
        for n in 0..1000 {
            thread_pool
                .execute(move || spawn_n(n))
                .expect(format!("err index n = {}", n).as_str());
        }
        thread::sleep(Duration::from_secs(1));
    }
}
