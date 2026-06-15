use crate::api::MonitorCheck;
use crate::utils::fmt_ms;
use yew::prelude::*;

#[derive(Properties, PartialEq)]
pub struct SparklineProps {
    pub checks: Vec<MonitorCheck>,
}

#[function_component(Sparkline)]
pub fn sparkline(props: &SparklineProps) -> Html {
    let bars: Vec<Html> = props
        .checks
        .iter()
        .rev()
        .take(30)
        .map(|c| {
            let is_up = c.status == "up";
            let cls = if is_up { "sparkline-bar up" } else { "sparkline-bar down" };
            let h = if is_up {
                (100u64.saturating_sub(c.response_time_ms / 30)).max(20).min(100)
            } else {
                15
            };
            let style = format!("height:{}%", h);
            let title = format!("{} — {}", c.status, fmt_ms(c.response_time_ms));
            html! { <div class={cls} style={style} title={title}></div> }
        })
        .collect();

    html! { <div class="sparkline">{ for bars }</div> }
}
