use js_sys::Date;
use yew::prelude::*;

#[derive(Properties, PartialEq)]
pub struct IncidentSparklineProps {
    /// 90-element vec, index 0 = 89 days ago, index 89 = today.
    /// Each value is total downtime minutes for that calendar day.
    pub day_downtime: Vec<u32>,
}

fn day_color(minutes: u32) -> String {
    if minutes == u32::MAX { return "hsl(220,10%,30%)".into(); }
    if minutes == 0        { return "#22c55e".into(); }
    let t = (minutes.min(60) as f64) / 60.0;
    let h = 55.0 * (1.0 - t);
    let s = 92.0 - 8.0 * t;
    let l = 50.0 + 10.0 * t;
    format!("hsl({h:.0},{s:.0}%,{l:.0}%)")
}

fn downtime_label(mins: u32) -> String {
    if mins == u32::MAX { return "No data".into(); }
    if mins == 0        { return "No downtime".into(); }
    if mins >= 60       { return format!("{}h {}m downtime", mins / 60, mins % 60); }
    format!("{mins} min downtime")
}

#[function_component(IncidentSparkline)]
pub fn incident_sparkline(props: &IncidentSparklineProps) -> Html {
    let months = ["Jan","Feb","Mar","Apr","May","Jun",
                  "Jul","Aug","Sep","Oct","Nov","Dec"];
    let now_ms  = Date::now();
    let day_ms  = 86_400_000.0_f64;

    let data: &[u32] = if props.day_downtime.is_empty() {
        // Show grey placeholders while the fetch is in-flight.
        return html! {
            <div class="incident-sparkline-wrap">
                <div class="isl-loading">{ "Loading history…" }</div>
            </div>
        };
    } else {
        &props.day_downtime
    };

    let n = data.len();
    let bars: Html = data.iter().enumerate().map(|(i, &mins)| {
        let days_ago = n - 1 - i;
        let day_ms_val = now_ms - days_ago as f64 * day_ms;
        let d = Date::new(&wasm_bindgen::JsValue::from_f64(day_ms_val));
        let date_str = format!("{} {}", months[d.get_month() as usize], d.get_date());
        let color    = day_color(mins);
        let label    = downtime_label(mins);

        html! {
            <div class="bar-col sp-bar-col" key={i}>
                <div class="sp-day-bar" style={format!("background:{color}")}></div>
                <div class="bar-tooltip">
                    <span class="bar-tooltip-value">{ date_str }</span>
                    <span class="bar-tooltip-time">{ label }</span>
                </div>
            </div>
        }
    }).collect();

    html! {
        <div class="incident-sparkline-wrap">
            <div class="isl-header">
                <span class="isl-title">{ "90-day incident history" }</span>
                <div class="isl-legend">
                    <span class="isl-dot" style="background:#22c55e"></span>
                    <span class="isl-legend-label">{ "Operational" }</span>
                    <span class="isl-dot" style="background:hsl(38,90%,50%)"></span>
                    <span class="isl-legend-label">{ "Partial" }</span>
                    <span class="isl-dot" style="background:#ef4444"></span>
                    <span class="isl-legend-label">{ "Outage" }</span>
                </div>
            </div>
            <div class="sp-sparkline isl-bars">
                <div class="sp-bars">{ bars }</div>
            </div>
            <div class="isl-footer">
                <span>{ "90 days ago" }</span>
                <span>{ "Today" }</span>
            </div>
        </div>
    }
}
