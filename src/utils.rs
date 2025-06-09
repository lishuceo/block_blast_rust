use macroquad::prelude::{screen_width, screen_height};

// 获取设备DPI缩放比例的辅助函数
pub fn get_dpi_scale() -> f32 {
    #[cfg(target_os = "ios")]
    {
        // iOS设备通常有更高的像素密度，使用更高的缩放比
        // iPhone的Retina显示屏通常是2x或3x缩放
        return 3.0;
    }
    
    #[cfg(target_os = "android")]
    {
        // Android设备根据屏幕密度调整
        // 通常为1.5x到4x之间
        return 2.0;
    }
    
    // 桌面平台，根据实际DPI动态计算
    // macroquad没有直接提供获取系统DPI的API，所以我们使用推断
    let (w, h) = (screen_width(), screen_height());
    if w > 2000.0 || h > 2000.0 {
        // 4K或高分辨率显示器
        2.0
    } else if w > 1200.0 || h > 1200.0 {
        // 高清显示器
        1.5
    } else {
        // 标准显示器
        1.0
    }
}

// 平滑过渡辅助函数
pub fn smooth_step(edge0: f32, edge1: f32, x: f32) -> f32 {
    let t = f32::max(0.0, f32::min(1.0, (x - edge0) / (edge1 - edge0)));
    t * t * (3.0 - 2.0 * t) // 平滑的三次函数
}

// --- 周排行榜相关的辅助函数 ---

const SECONDS_IN_DAY: f64 = 24.0 * 60.0 * 60.0;
const DAYS_IN_NON_LEAP_YEAR: f64 = 365.0;
// const DAYS_IN_LEAP_YEAR: f64 = 366.0;

fn is_leap_year(year: i32) -> bool {
    (year % 4 == 0 && year % 100 != 0) || (year % 400 == 0)
}

fn day_of_year(year: i32, month: u32, day: u32) -> u32 {
    let days_in_month: [u32; 12] = [
        31, if is_leap_year(year) { 29 } else { 28 }, 31, 30, 31, 30, 
        31, 31, 30, 31, 30, 31
    ];
    let mut doy = day;
    for m in 0..(month -1) as usize {
        doy += days_in_month[m];
    }
    doy
}

// 简化的从时间戳计算年份、月份、日的方法 (非常粗略，仅用于周计算的近似)
// miniquad::date::now() 返回的是毫秒级时间戳，需转换为秒
fn timestamp_to_approx_date(timestamp_ms: f64) -> (i32, u32, u32) {
    let total_seconds = timestamp_ms / 1000.0;
    let mut year = 1970;
    let mut seconds_remaining_in_year = total_seconds;

    loop {
        let days_in_current_year = if is_leap_year(year) { 366.0 } else { 365.0 };
        let seconds_in_current_year = days_in_current_year * SECONDS_IN_DAY;
        if seconds_remaining_in_year < seconds_in_current_year {
            break;
        }
        seconds_remaining_in_year -= seconds_in_current_year;
        year += 1;
    }

    let mut day_of_year_val = (seconds_remaining_in_year / SECONDS_IN_DAY).floor() as u32 + 1; // 1-indexed
    let mut month = 1;
    let days_in_month_arr = [
        31, if is_leap_year(year) { 29 } else { 28 }, 31, 30, 31, 30, 
        31, 31, 30, 31, 30, 31
    ];

    for days_in_current_month in days_in_month_arr.iter() {
        if day_of_year_val <= *days_in_current_month {
            break;
        }
        day_of_year_val -= *days_in_current_month;
        month += 1;
    }
    let day = day_of_year_val;
    (year, month, day)
}


// 计算当前年份和ISO周数 (简化版)
// 注意：这只是一个近似实现，没有完全遵循ISO 8601周日历标准。
// 真正的ISO周计算比较复杂，通常需要专门的日期库。
fn get_current_year_week_approx() -> (i32, u32) {
    let now_ms = miniquad::date::now();
    let (year, month, day) = timestamp_to_approx_date(now_ms);
    
    // 计算这是一年中的第几天 (1-indexed)
    let day_of_year_num = day_of_year(year, month, day);

    // 简单的周数计算：(一年中的第几天 + 6) / 7，向上取整。
    // 这与ISO周略有不同，但对于生成每周key是一个可用的方法。
    // 另一种更简单的方法是 (day_of_year - 1) / 7 + 1
    let week_number = (day_of_year_num as f64 / 7.0).ceil() as u32;
    // 确保周数在1-53之间 (一年最多有53周)
    (year, week_number.clamp(1, 53))
}

pub fn get_weekly_leaderboard_key() -> String {
    let (year, week) = get_current_year_week_approx();
    format!("player_score_{}_{:02}", year, week) // 例如 player_score_2024_30
}

pub fn get_current_week_display_text() -> String {
    let (year, week) = get_current_year_week_approx();
    format!("({}年 第{:02}周)", year, week)
}

// pub fn get_current_week_placeholder() -> String {
//     // 返回一个简单的文本占位符，表示"本周"
//     // 之后可以替换为更精确的日期范围计算
//     "(本周)".to_string()
// } 