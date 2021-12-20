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

use std::thread;
use std::thread::JoinHandle;
use std::time::Duration;

use crate::utils::errors::{Errs, StarryResult};

pub struct Thread;

impl Thread {
    pub fn spawn<F, T>(f: F) -> StarryResult<JoinHandle<T>>
        where
            F: FnOnce() -> T,
            F: Send + 'static,
            T: Send + 'static,
    {
        let thread_builder = thread::Builder::new();
        match thread_builder.spawn(f) {
            Ok(res) => Ok(res),
            Err(err) => Err(Errs::strs("thread default error!", err)),
        }
    }

    pub fn spawn_on_name<F, T>(name: String, f: F) -> StarryResult<JoinHandle<T>>
        where
            F: FnOnce() -> T,
            F: Send + 'static,
            T: Send + 'static,
    {
        let mut thread_builder = thread::Builder::new();
        thread_builder = thread_builder.name(name);
        match thread_builder.spawn(f) {
            Ok(res) => Ok(res),
            Err(err) => Err(Errs::strs("thread new error!", err)),
        }
    }

    pub fn spawn_on_custom<F, T>(
        name: String,
        stack_size: usize,
        f: F,
    ) -> StarryResult<JoinHandle<T>>
        where
            F: FnOnce() -> T,
            F: Send + 'static,
            T: Send + 'static,
    {
        let mut thread_builder = thread::Builder::new();
        thread_builder = thread_builder.name(name);
        if stack_size > 0 {
            thread_builder = thread_builder.stack_size(stack_size);
        }
        match thread_builder.spawn(f) {
            Ok(res) => Ok(res),
            Err(err) => Err(Errs::strs("thread create error!", err)),
        }
    }

    pub fn sleep(duration: Duration) {
        thread::sleep(duration);
    }
}


#[cfg(test)]
mod thread_test {
    use std::thread;
    use std::time::Duration;

    use crate::utils::concurrent::Thread;

    fn spawn1() {
        println!("spawn 1");
    }

    // fn spawn2() {
    //     println!("spawn 2");
    // }
    //
    // fn spawn3() {
    //     println!("spawn 3");
    // }
    //
    // fn spawn_n(n: usize) {
    //     println!("spawn {}", n);
    // }

    #[test]
    fn test1() {
        Thread::spawn(spawn1).unwrap();
        Thread::sleep(Duration::from_secs(1));
    }

    #[test]
    fn test2() {
        let handle = Thread::spawn(|| {
            for i in 0..100 {
                println!("spawned handle thread print {}", i);
                thread::sleep(Duration::from_millis(10));
            }
        })
            .unwrap();
        for i in 0..30 {
            println!("main thread print {}", i);
            thread::sleep(Duration::from_millis(1));
        }
        handle.join().unwrap();
    }

    #[test]
    fn test3() {
        let handle1 = Thread::spawn(|| {
            for i in 0..10 {
                println!("spawned handle thread print none.{}", i);
                thread::sleep(Duration::from_millis(10));
            }
        })
            .unwrap();
        let handle2 = Thread::spawn_on_name("name".to_string(), || {
            for i in 0..10 {
                println!(
                    "spawned handle thread print {}.{}",
                    thread::current().name().unwrap(),
                    i
                );
                thread::sleep(Duration::from_millis(10));
            }
        })
            .unwrap();
        let handle3 = Thread::spawn_on_custom("stack".to_string(), 100, || {
            for i in 0..10 {
                println!(
                    "spawned handle thread print {}.{}",
                    thread::current().name().unwrap(),
                    i
                );
                thread::sleep(Duration::from_millis(10));
            }
        })
            .unwrap();
        handle1.join().unwrap();
        handle2.join().unwrap();
        handle3.join().unwrap();
    }
}
