use crate::api::{MonitorCheck, MonitorCheckStatus};
use crate::components::bar_tooltip;
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
            let is_up = c.status == MonitorCheckStatus::Up;
            let cls = if is_up { "sparkline-bar up" } else { "sparkline-bar down" };
            let h = if is_up {
                (100u64.saturating_sub(c.response_time_ms / 30)).clamp(20, 100)
            } else {
                15
            };
            let style = format!("height:{}%", h);
            html! {
                <div class="bar-col">
                    <div class={cls} style={style}></div>
                    { bar_tooltip(c) }
                </div>
            }
        })
        .collect();

    html! { <div class="sparkline">{ for bars }</div> }
}
