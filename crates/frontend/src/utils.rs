use crate::api::{Monitor, MonitorConfig};

pub fn fmt_date(iso: &str) -> String {
    let d = js_sys::Date::new(&wasm_bindgen::JsValue::from_str(iso));
    let month = match d.get_month() {
        0 => "Jan",
        1 => "Feb",
        2 => "Mar",
        3 => "Apr",
        4 => "May",
        5 => "Jun",
        6 => "Jul",
        7 => "Aug",
        8 => "Sep",
        9 => "Oct",
        10 => "Nov",
        _ => "Dec",
    };
    format!(
        "{} {}, {:02}:{:02}",
        month,
        d.get_date(),
        d.get_hours(),
        d.get_minutes()
    )
}

pub fn fmt_duration(start: &str, end: Option<&str>) -> String {
    let start_ms = js_sys::Date::new(&wasm_bindgen::JsValue::from_str(start)).get_time();
    let end_ms = end
        .map(|e| js_sys::Date::new(&wasm_bindgen::JsValue::from_str(e)).get_time())
        .unwrap_or_else(js_sys::Date::now);
    let s = ((end_ms - start_ms) / 1000.0) as i64;
    if s < 60 {
        return format!("{}s", s);
    }
    let m = s / 60;
    if m < 60 {
        return format!("{}m", m);
    }
    let h = m / 60;
    let rm = m % 60;
    if h < 24 {
        return if rm > 0 {
            format!("{}h {}m", h, rm)
        } else {
            format!("{}h", h)
        };
    }
    format!("{}d {}h", h / 24, h % 24)
}

pub fn fmt_time(iso: &str) -> String {
    let d = js_sys::Date::new(&wasm_bindgen::JsValue::from_str(iso));
    format!(
        "{:02}:{:02}:{:02}",
        d.get_hours(),
        d.get_minutes(),
        d.get_seconds()
    )
}

pub fn fmt_ms(ms: u64) -> String {
    if ms >= 1000 {
        format!("{:.1}s", ms as f64 / 1000.0)
    } else {
        format!("{}ms", ms)
    }
}

pub fn monitor_url(m: &Monitor) -> &str {
    match &m.config {
        MonitorConfig::Http { url, .. } => url.as_str(),
    }
}

pub fn uptime_class(pct: f64) -> &'static str {
    if pct >= 99.0 {
        "high"
    } else if pct >= 95.0 {
        "mid"
    } else {
        "low"
    }
}
