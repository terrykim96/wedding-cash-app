//! 실행 환경 정보: 현재 시각, PC 이름 (business-rules.md BR-14)

use chrono::{Local, Timelike};

use crate::model::Timestamp;

/// 로컬 시간대 오프셋을 포함한 현재 시각 (초 단위)
pub fn now() -> Timestamp {
    let t = Local::now().fixed_offset();
    t.with_nanosecond(0).unwrap_or(t)
}

/// OS 호스트 이름. 얻지 못하면 `unknown-pc`
pub fn pc_name() -> String {
    let name = gethostname::gethostname().to_string_lossy().trim().to_string();
    if name.is_empty() {
        "unknown-pc".to_string()
    } else {
        name
    }
}
