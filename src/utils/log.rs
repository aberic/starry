/*
 * Copyright (c) 2020. Aberic - All Rights Reserved.
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

use std::sync::RwLock;

use log::LevelFilter;
use log4rs::append::console::ConsoleAppender;
use log4rs::append::rolling_file::policy::compound::roll::fixed_window::FixedWindowRoller;
use log4rs::append::rolling_file::policy::compound::trigger::size::SizeTrigger;
use log4rs::append::rolling_file::policy::compound::CompoundPolicy;
use log4rs::append::rolling_file::RollingFileAppender;
use log4rs::config::{Appender, Config, Logger, Root};
use log4rs::encode::json::JsonEncoder;
use log4rs::encode::pattern::PatternEncoder;
use log4rs::Handle;
use once_cell::sync::Lazy;
use std::fs;

#[derive(Debug, PartialEq, Clone)]
pub struct LogModule {
    /// 输出日志模块名称
    pub name: String,
    /// 输出日志所在包，如："app::requests"
    pub pkg: String,
    /// 输出日志级别
    pub level: LevelFilter,
    /// 是否在主日志文件中同步记录
    pub additive: bool,
    /// log_dir 日志文件目录
    pub dir: String,
    /// log_file_max_size 每个日志文件保存的最大尺寸 单位：M
    pub file_max_size: u64,
    /// file_max_count 文件最多保存多少个
    pub file_max_count: u32,
}

impl LogModule {
    fn name(&self) -> String {
        self.name.clone()
    }

    fn pkg(&self) -> String {
        self.pkg.clone()
    }

    fn level(&self) -> LevelFilter {
        self.level.clone()
    }

    fn dir(&self) -> String {
        self.dir.clone()
    }

    /// 获取日志空配置信息
    fn config_default(&self) -> Config {
        self.config(vec![])
    }

    /// 获取日志配置信息
    fn config(&self, modules: Vec<LogModule>) -> Config {
        let stdout_name = "stdout";
        let stdout = ConsoleAppender::builder()
            .encoder(Box::new(PatternEncoder::new(
                "{d(%+)(local)} {l} {t} {m}{n}",
            )))
            .build();

        let server = rolling_appender(
            self.dir(),
            self.name(),
            self.file_max_size,
            self.file_max_count,
        );

        let config = Config::builder()
            .appender(Appender::builder().build(stdout_name, Box::new(stdout)))
            .appender(Appender::builder().build(self.name(), Box::new(server)));
        let root = Root::builder().appender(stdout_name).appender(self.name());
        let mut appenders: Vec<Appender> = vec![];
        let mut loggers: Vec<Logger> = vec![];

        for mdl in modules {
            let dir: String;
            let file_max_size: u64;
            let file_max_count: u32;
            if mdl.dir().is_empty() {
                dir = self.dir();
            } else {
                dir = mdl.dir();
            }
            if mdl.file_max_size <= 0 {
                file_max_size = self.file_max_size;
            } else {
                file_max_size = mdl.file_max_size;
            }
            if mdl.file_max_count <= 0 {
                file_max_count = self.file_max_count;
            } else {
                file_max_count = mdl.file_max_count;
            }
            let rolling_module = rolling_appender(dir, mdl.name(), file_max_size, file_max_count);
            appenders.push(Appender::builder().build(mdl.name(), Box::new(rolling_module)));
            loggers.push(
                Logger::builder()
                    .appender(mdl.name())
                    .additive(mdl.additive)
                    .build(mdl.pkg(), mdl.level()),
            );
            root.clone().appender(mdl.name());
        }
        return config
            .appenders(appenders)
            .loggers(loggers)
            .build(root.build(self.level()))
            .expect("config build failed!");
    }

    /// 初始化日志
    ///
    /// service_name 日志所服务的服务名称
    ///
    /// log_dir 日志文件目录
    ///
    /// log_file_max_size 每个日志文件保存的最大尺寸 单位：M
    ///
    /// file_max_count 文件最多保存多少个
    ///
    /// log_level 日志级别(debug/info/warn/Error/panic/fatal)
    pub fn config_log(&self, modules: Vec<LogModule>) {
        GLOBAL_LOG
            .write()
            .expect("global log write failed!")
            .handle
            .set_config(self.config(modules))
    }
}

pub struct LogHandle {
    handle: Handle,
    // /// 是否生产环境，在生产环境下控制台不会输出任何日志
    // production: bool,
}

pub static GLOBAL_LOG: Lazy<RwLock<LogHandle>> = Lazy::new(|| {
    let module = LogModule {
        name: "log".to_string(),
        pkg: "".to_string(),
        level: LevelFilter::Off,
        additive: false,
        dir: "./wonder_log_test_rm/logs".to_string(),
        file_max_size: 0,
        file_max_count: 0,
    };
    let handle = LogHandle {
        handle: log4rs::init_config(module.config_default()).expect("log init config failed!"),
        // production: false,
    };
    fs::remove_dir_all("./wonder_log_test_rm").expect("fs remove dir failed!");
    RwLock::new(handle)
});

pub fn set_log_test() {
    let module = LogModule {
        name: "test".to_string(),
        pkg: "".to_string(),
        level: LevelFilter::Debug,
        additive: true,
        dir: "./test/logs".to_string(),
        file_max_size: 1024,
        file_max_count: 7,
    };
    GLOBAL_LOG
        .write()
        .expect("global log write failed!")
        .handle
        .set_config(module.config(vec![]))
}

fn rolling_appender(
    dir: String,
    module_name: String,
    file_max_size: u64,
    file_max_count: u32,
) -> RollingFileAppender {
    RollingFileAppender::builder()
        .append(true)
        .encoder(Box::new(JsonEncoder::default()))
        .build(
            format!("{}/{}{}", dir.clone(), module_name, ".log"),
            Box::new(CompoundPolicy::new(
                Box::new(SizeTrigger::new(file_max_size * 1024 * 1024)),
                Box::new(
                    FixedWindowRoller::builder()
                        .base(1)
                        .build(
                            &*format!("{}/{}{}", dir, module_name, "-log-{}.log"),
                            file_max_count,
                        )
                        .expect("fixed window roller build failed!"),
                ),
            )),
        )
        .expect("rolling file appender build failed!")
}


#[cfg(test)]
mod log_test {
    use log::LevelFilter;
    use crate::utils::log::LogModule;

    #[test]
    fn logs() {
        let module = LogModule {
            name: String::from("set"),
            pkg: "".to_string(),
            level: LevelFilter::Debug,
            additive: true,
            dir: String::from("tmp"),
            file_max_size: 1024,
            file_max_count: 7,
        };
        module.config_log(vec![
            LogModule {
                name: "mod1".to_string(),
                pkg: "george-log::examples::log_test::log_test_mod1".to_string(),
                level: LevelFilter::Trace,
                additive: true,
                dir: String::from("tmp"),
                file_max_size: 1024,
                file_max_count: 7,
            },
            LogModule {
                name: "mod2".to_string(),
                pkg: "george-log::examples::log_test::log_test_mod2".to_string(),
                level: LevelFilter::Debug,
                additive: true,
                dir: String::from(""),
                file_max_size: 0,
                file_max_count: 0,
            },
        ]);
        log::debug!("Hello, world!");
        log::info!("Hello, world!");
        log::warn!("Hello, world!");
        log::error!("Hello, world!");

        logs_mod1();
        logs_mod2();
    }

    fn logs_mod2() {
        log::trace!("Hello, world! logs_mod");
        log::debug!("Hello, world! logs_mod");
        log::info!("Hello, world! logs_mod");
        log::warn!("Hello, world! logs_mod");
        log::error!("Hello, world! logs_mod");
    }

    fn logs_mod1() {
        log::trace!("Hello, world! logs_mod");
        log::debug!("Hello, world! logs_mod");
        log::info!("Hello, world! logs_mod");
        log::warn!("Hello, world! logs_mod");
        log::error!("Hello, world! logs_mod");
    }
}
