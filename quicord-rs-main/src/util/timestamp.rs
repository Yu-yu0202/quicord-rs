/*
 * Copyright (c) 2026 Yu-yu0202
 *
 * This Source Code Form is subject to the terms of the
 * Mozilla Public License, v. 2.0. If a copy of the MPL
 * was not distributed with this file, You can obtain one at
 * https://mozilla.org/MPL/2.0/.
 */

use chrono::{DateTime, Local, Utc};
use twilight_model::util::Timestamp;

pub fn now_unix_secs() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_secs()
}

pub fn now_unix_millis() -> u128 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_millis()
}

pub fn now_unix_micros() -> u128 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_micros()
}

pub fn now_utc() -> DateTime<Utc> {
    Utc::now()
}

pub fn now_local() -> DateTime<Local> {
    Local::now()
}

pub fn now_timestamp() -> Timestamp {
    Timestamp::from_micros(now_unix_micros() as i64).unwrap()
}
