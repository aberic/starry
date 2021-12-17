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

use std::ops::{Add, Sub};

use chrono::{Duration, Local, NaiveDateTime};
use chrono::format::{DelayedFormat, StrftimeItems};

use crate::utils::errors::{Errs, StarryResult};

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub struct Time {
    duration: Duration,
}

impl Time {
    pub fn now() -> Self {
        Time {
            duration: Duration::nanoseconds(Local::now().naive_local().timestamp_nanos()),
        }
    }

    pub fn from(duration: Duration) -> Self {
        Time { duration }
    }

    pub fn from_minutes(minutes: i64) -> Self {
        Time {
            duration: Duration::minutes(minutes),
        }
    }

    /// 秒计
    pub fn from_secs(secs: i64) -> Self {
        Time {
            duration: Duration::seconds(secs),
        }
    }

    /// 毫秒计 -> 10的负3次秒 -> 0.001秒
    pub fn from_milliseconds(milliseconds: i64) -> Self {
        Time {
            duration: Duration::milliseconds(milliseconds),
        }
    }

    /// 微秒计 -> 10的负6次秒 -> 0.000001秒
    pub fn from_microseconds(microseconds: i64) -> Self {
        Time {
            duration: Duration::microseconds(microseconds),
        }
    }

    /// 纳秒计 -> 10的负9次秒 -> 0.000000001秒
    pub fn from_nanoseconds(nanoseconds: i64) -> Self {
        Time {
            duration: Duration::nanoseconds(nanoseconds),
        }
    }

    /// 用指定的格式字符串解析字符串并返回一个新的Time。
    ///
    /// # 例子
    ///
    /// ~~~~
    /// use starry::utils::Time;
    ///
    /// let time = Time::parse_from_str("2021-12-08 10:37:54", "%Y-%m-%d %H:%M:%S").unwrap();
    /// assert_eq!("2021-12-08 10:37:54", time.format_string("%Y-%m-%d %H:%M:%S"));
    ///
    /// let time = Time::parse_from_str("2021-W49-3103754", "%G-W%V-%u%H%M%S").unwrap();
    /// assert_eq!("2021-12-08 10:37:54", time.format_string("%Y-%m-%d %H:%M:%S"));
    ///
    /// let time = Time::parse_from_str("2021-12-08T10:37:54", "%Y-%m-%dT%H:%M:%S").unwrap();
    /// assert_eq!("2021-12-08 10:37:54", time.format_string("%Y-%m-%d %H:%M:%S"));
    ///
    /// let time = Time::parse_from_str("Wed, 08 Dec 2021 10:37:54 GMT", "%a, %d %b %Y %H:%M:%S GMT").unwrap();
    /// assert_eq!("2021-12-08 10:37:54", time.format_string("%Y-%m-%d %H:%M:%S"));
    ///
    /// let time = Time::parse_from_str("08Dec2021AM103754", "%d%b%Y%p%I%M%S%.f").unwrap();
    /// assert_eq!("2021-12-08 10:37:54", time.format_string("%Y-%m-%d %H:%M:%S"));
    ///
    /// let time = Time::parse_from_str("2021-12-08 10:37:54", "%Y-%m-%d %H:%M:%S%.f").unwrap();
    /// assert_eq!("2021-12-08 10:37:54", time.format_string("%Y-%m-%d %H:%M:%S"));
    ///
    /// let time = Time::parse_from_str("21/12/08 10:37:54", "%y/%m/%d %H:%M:%S").unwrap();
    /// assert_eq!("2021-12-08 10:37:54", time.format_string("%Y-%m-%d %H:%M:%S"));
    /// ~~~~
    pub fn parse_from_str(s: &str, fmt: &str) -> StarryResult<Self> {
        match NaiveDateTime::parse_from_str(s, fmt) {
            Ok(src) => Ok(Time {
                duration: Duration::nanoseconds(src.timestamp_nanos()),
            }),
            Err(err) => Err(Errs::err(err))
        }
    }

    /// 使用指定的格式字符串格式化组合的日期和时间。
    /// 这将返回一个DelayedFormat，该格式仅在实际格式化时才转换为字符串。
    /// 可以使用to_string方法来获取String对象，或者直接将其输出!和其他格式化宏。(这样就避免了内存的冗余分配。)
    /// 错误格式的字符串不会立即发出错误。相反，转换或格式化DelayedFormat失败。因此，建议立即使用DelayedFormat
    ///
    /// # 例子
    ///
    /// ```
    /// // use starry::utils::Time;
    ///
    /// // let time = Time::parse_from_str("2021-12-08 10:37:54", "%Y-%m-%d %H:%M:%S").unwrap();
    /// // assert_eq!(time.format("%Y-%m-%d %H:%M:%S").to_string(), "2021-12-08 10:37:54");
    /// // assert_eq!(time.format("around %l %p on %b %-d").to_string(), "around 10 AM on Dec 8");
    /// // assert_eq!(time.format("%G-W%V-%u%H%M%S").to_string(), "2021-W49-3103754");
    /// // assert_eq!(time.format("%Y-%m-%dT%H:%M:%S%z").to_string(), "2021-12-08T10:37:54");
    /// // assert_eq!(time.format("%a, %d %b %Y %H:%M:%S GMT").to_string(), "Wed, 08 Dec 2021 10:37:54 GMT");
    /// // assert_eq!(time.format("%d%b%Y%p%I%M%S%.f").to_string(), "08Dec2021AM103754");
    /// // assert_eq!(time.format("%Y-%m-%d %H:%M:%S%.f").to_string(), "2021-12-08 10:37:54");
    /// // assert_eq!(time.format("%y/%m/%d %H:%M:%S").to_string(), "21/12/08 10:37:54");
    ///
    /// ```
    ///
    /// The resulting `DelayedFormat` can be formatted directly via the `Display` trait.
    ///
    /// ```
    /// use starry::utils::Time;
    /// let time = Time::parse_from_str("2015-09-05 23:56:04", "%Y-%m-%d %H:%M:%S").unwrap();;
    /// assert_eq!(format!("{}", time.format("%Y-%m-%d %H:%M:%S")), "2015-09-05 23:56:04");
    /// assert_eq!(format!("{}", time.format("around %l %p on %b %-d")), "around 11 PM on Sep 5");
    /// ```
    pub fn format<'a>(&self, fmt: &'a str) -> DelayedFormat<StrftimeItems<'a>> {
        let time_from_stamp = NaiveDateTime::from_timestamp(self.duration.num_seconds(), 0);
        time_from_stamp.format(fmt)
    }

    /// 格式化成指定字符串样式
    ///
    /// # 例子
    /// * `%Y-%m-%d %H:%M:%S` -> `2015-09-05 23:56:04`
    /// * `around %l %p on %b %-d` -> `around 11 PM on Sep 5`
    /// * `%G-W%V-%u%H%M%S` -> `2015-W06-1 000000`
    /// * `%Y-%m-%dT%H:%M:%S%z` -> `2014-5-7T12:34:56+09:30`
    /// * `%a, %d %b %Y %H:%M:%S GMT` -> `Fri, 09 Aug 2013 23:54:35 GMT`
    /// * `%d%b%Y%p%I%M%S%.f` -> `5sep2015pm012345.6789`
    /// * `%Y-%m-%d %H:%M:%S%.f` -> `2015-07-01 08:59:60.123`
    /// * `%y/%m/%d %H:%M:%S` -> `94/9/4 7:15:21`
    pub fn format_string(&self, fmt: &str) -> String {
        self.format(fmt).to_string()
    }

    /// 将指定时间格式化成指定字符串样式
    ///
    /// # Example
    /// * `%Y-%m-%d %H:%M:%S` -> `2015-09-05 23:56:04`
    /// * `around %l %p on %b %-d` -> `around 11 PM on Sep 5`
    /// * `%G-W%V-%u%H%M%S` -> `2015-W06-1 000000`
    /// * `%Y-%m-%dT%H:%M:%S%z` -> `2014-5-7T12:34:56+09:30`
    /// * `%a, %d %b %Y %H:%M:%S GMT` -> `Fri, 09 Aug 2013 23:54:35 GMT`
    pub fn format_data(time: Time, fmt: &str) -> String {
        time.format(fmt).to_string()
    }

    /// 输出该`%Y-%m-%d %H:%M:%S`格式字符串
    pub fn to_string(&self) -> String {
        self.format("%Y-%m-%d %H:%M:%S").to_string()
    }

    pub fn duration(&self) -> Duration {
        self.duration
    }

    /// 返回持续时间中整个星期的总数。
    pub fn num_weeks(&self) -> i64 {
        self.duration.num_weeks()
    }

    /// 返回持续时间中整个天的总数。
    pub fn num_days(&self) -> i64 {
        self.duration.num_days()
    }

    /// 返回持续时间中整个小时的总数。
    pub fn num_hours(&self) -> i64 {
        self.duration.num_hours()
    }

    /// 返回持续时间中整个分钟的总数。
    pub fn num_minutes(&self) -> i64 {
        self.duration.num_minutes()
    }

    /// 返回持续时间中整个秒的总数。
    pub fn num_seconds(&self) -> i64 {
        self.duration.num_seconds()
    }

    /// 返回持续时间中整个毫秒的总数。
    pub fn num_milliseconds(&self) -> i64 {
        self.duration.num_milliseconds()
    }

    /// 返回持续时间中整个微秒的总数，或者在溢出时为None(在任何方向上都超过2^63微秒)。
    pub fn num_microseconds(&self) -> Option<i64> {
        self.duration.num_microseconds()
    }

    /// 返回持续时间中整个纳秒的总数，或者在溢出时为None(在任何方向上都超过2^63微秒)。
    pub fn num_nanoseconds(&self) -> Option<i64> {
        self.duration.num_nanoseconds()
    }

    pub fn add(&mut self, duration: Duration) {
        self.duration = self.duration.add(duration)
    }

    pub fn add_nanoseconds(&mut self, src: i64) {
        self.duration = self.duration.add(Duration::nanoseconds(src))
    }

    pub fn add_microseconds(&mut self, src: i64) {
        self.duration = self.duration.add(Duration::microseconds(src))
    }

    pub fn add_milliseconds(&mut self, src: i64) {
        self.duration = self.duration.add(Duration::milliseconds(src))
    }

    pub fn add_seconds(&mut self, src: i64) {
        self.duration = self.duration.add(Duration::seconds(src))
    }

    pub fn add_minutes(&mut self, src: i64) {
        self.duration = self.duration.add(Duration::minutes(src))
    }

    pub fn add_hours(&mut self, src: i64) {
        self.duration = self.duration.add(Duration::hours(src))
    }

    pub fn add_days(&mut self, src: i64) {
        self.duration = self.duration.add(Duration::days(src))
    }

    pub fn add_weeks(&mut self, src: i64) {
        self.duration = self.duration.add(Duration::weeks(src))
    }

    pub fn sub(&mut self, duration: Duration) {
        self.duration = self.duration.sub(duration)
    }

    pub fn sub_nanoseconds(&mut self, src: i64) {
        self.duration = self.duration.sub(Duration::nanoseconds(src))
    }

    pub fn sub_microseconds(&mut self, src: i64) {
        self.duration = self.duration.sub(Duration::microseconds(src))
    }

    pub fn sub_milliseconds(&mut self, src: i64) {
        self.duration = self.duration.sub(Duration::milliseconds(src))
    }

    pub fn sub_seconds(&mut self, src: i64) {
        self.duration = self.duration.sub(Duration::seconds(src))
    }

    pub fn sub_minutes(&mut self, src: i64) {
        self.duration = self.duration.sub(Duration::minutes(src))
    }

    pub fn sub_hours(&mut self, src: i64) {
        self.duration = self.duration.sub(Duration::hours(src))
    }

    pub fn sub_days(&mut self, src: i64) {
        self.duration = self.duration.sub(Duration::days(src))
    }

    pub fn sub_weeks(&mut self, src: i64) {
        self.duration = self.duration.sub(Duration::weeks(src))
    }
}


#[test]
fn from_test() {
    let time = Time::from_secs(1638907696);
    assert_eq!("2021-12-07 20:08:16", time.format_string("%Y-%m-%d %H:%M:%S"));
    assert_eq!("2021-12-07 20:08:16", time.to_string());
}

#[test]
fn add_test() {
    let mut time = Time::from_secs(1638907696);
    assert_eq!("2021-12-07 20:08:16", time.to_string());
    time.add(Duration::seconds(60));
    assert_eq!("2021-12-07 20:09:16", time.to_string());
    time.add(Duration::minutes(60));
    assert_eq!("2021-12-07 21:09:16", time.to_string());
    time.add(Duration::hours(24));
    assert_eq!("2021-12-08 21:09:16", time.to_string());
    time.add(Duration::days(10));
    assert_eq!("2021-12-18 21:09:16", time.to_string());
    time.add(Duration::weeks(3));
    assert_eq!("2022-01-08 21:09:16", time.to_string());
}

#[test]
fn add_self_test() {
    let mut time = Time::from_secs(1638907696);
    assert_eq!("2021-12-07 20:08:16", time.to_string());
    time.add_seconds(60);
    assert_eq!("2021-12-07 20:09:16", time.to_string());
    time.add_minutes(60);
    assert_eq!("2021-12-07 21:09:16", time.to_string());
    time.add_hours(24);
    assert_eq!("2021-12-08 21:09:16", time.to_string());
    time.add_days(10);
    assert_eq!("2021-12-18 21:09:16", time.to_string());
    time.add_weeks(3);
    assert_eq!("2022-01-08 21:09:16", time.to_string());
}

#[test]
fn sub_test() {
    let mut time = Time::from_secs(1641676156);
    assert_eq!("2022-01-08 21:09:16", time.to_string());
    time.sub(Duration::weeks(3));
    assert_eq!("2021-12-18 21:09:16", time.to_string());
    time.sub(Duration::days(10));
    assert_eq!("2021-12-08 21:09:16", time.to_string());
    time.sub(Duration::hours(24));
    assert_eq!("2021-12-07 21:09:16", time.to_string());
    time.sub(Duration::minutes(60));
    assert_eq!("2021-12-07 20:09:16", time.to_string());
    time.sub(Duration::seconds(60));
    assert_eq!("2021-12-07 20:08:16", time.to_string());
}

#[test]
fn sub_self_test() {
    let mut time = Time::from_secs(1641676156);
    assert_eq!("2022-01-08 21:09:16", time.to_string());
    time.sub_weeks(3);
    assert_eq!("2021-12-18 21:09:16", time.to_string());
    time.sub_days(10);
    assert_eq!("2021-12-08 21:09:16", time.to_string());
    time.sub_hours(24);
    assert_eq!("2021-12-07 21:09:16", time.to_string());
    time.sub_minutes(60);
    assert_eq!("2021-12-07 20:09:16", time.to_string());
    time.sub_seconds(60);
    assert_eq!("2021-12-07 20:08:16", time.to_string());
}

#[test]
fn format_test() {
    // 1638959874 1638959874647 1638959874647599 1638959874647599000
    // let tm = Time::from_secs(1638959874);
    // println!("s1 = {}", tm.format_string("%Y-%m-%d %H:%M:%S"));
    // let tm = Time::from_milliseconds(1638959874647);
    // println!("s2 = {}", tm.format_string("%Y-%m-%d %H:%M:%S%.3f"));
    // let tm = Time::from_microseconds(1638959874647599);
    // println!("s3 = {}", tm.format_string("%Y-%m-%d %H:%M:%S%.6f"));
    // let tm = Time::from_nanoseconds(1638959874647599000);
    // println!("s4 = {}", tm.format_string("%Y-%m-%d %H:%M:%S%.9f"));

    let time = Time::from_nanoseconds(1638959874647599000);
    assert_eq!("2021-12-08 10:37:54", time.format_string("%Y-%m-%d %H:%M:%S"));
    assert_eq!("around 10 AM on Dec 8", time.format_string("around %l %p on %b %-d"));
    assert_eq!("2021-W49-3103754", time.format_string("%G-W%V-%u%H%M%S"));
    assert_eq!("2021-12-08T10:37:54", time.format_string("%Y-%m-%dT%H:%M:%S"));
    assert_eq!("Wed, 08 Dec 2021 10:37:54 GMT", time.format_string("%a, %d %b %Y %H:%M:%S GMT"));
    assert_eq!("08Dec2021AM103754", time.format_string("%d%b%Y%p%I%M%S%.f"));
    assert_eq!("2021-12-08 10:37:54", time.format_string("%Y-%m-%d %H:%M:%S%.f"));
    assert_eq!("21/12/08 10:37:54", time.format_string("%y/%m/%d %H:%M:%S"));
}

#[test]
fn parse_from_str() {
    let time = Time::parse_from_str("2021-12-08 10:37:54", "%Y-%m-%d %H:%M:%S").unwrap();
    assert_eq!("2021-12-08 10:37:54", time.format_string("%Y-%m-%d %H:%M:%S"));

    let time = Time::parse_from_str("2021-W49-3103754", "%G-W%V-%u%H%M%S").unwrap();
    assert_eq!("2021-12-08 10:37:54", time.format_string("%Y-%m-%d %H:%M:%S"));

    let time = Time::parse_from_str("2021-12-08T10:37:54", "%Y-%m-%dT%H:%M:%S").unwrap();
    assert_eq!("2021-12-08 10:37:54", time.format_string("%Y-%m-%d %H:%M:%S"));

    let time = Time::parse_from_str("Wed, 08 Dec 2021 10:37:54 GMT", "%a, %d %b %Y %H:%M:%S GMT").unwrap();
    assert_eq!("2021-12-08 10:37:54", time.format_string("%Y-%m-%d %H:%M:%S"));

    let time = Time::parse_from_str("08Dec2021AM103754", "%d%b%Y%p%I%M%S%.f").unwrap();
    assert_eq!("2021-12-08 10:37:54", time.format_string("%Y-%m-%d %H:%M:%S"));

    let time = Time::parse_from_str("2021-12-08 10:37:54", "%Y-%m-%d %H:%M:%S%.f").unwrap();
    assert_eq!("2021-12-08 10:37:54", time.format_string("%Y-%m-%d %H:%M:%S"));

    let time = Time::parse_from_str("21/12/08 10:37:54", "%y/%m/%d %H:%M:%S").unwrap();
    assert_eq!("2021-12-08 10:37:54", time.format_string("%Y-%m-%d %H:%M:%S"));
}
