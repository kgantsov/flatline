use yew::prelude::*;

#[derive(Properties, PartialEq)]
pub struct StatsBarProps {
    pub total: AttrValue,
    pub up: AttrValue,
    pub down: AttrValue,
    pub incidents: AttrValue,
}

#[function_component(StatsBar)]
pub fn stats_bar(props: &StatsBarProps) -> Html {
    html! {
        <div class="stats-bar">
            <div class="stat-card">
                <span class="stat-label">{ "Total" }</span>
                <span class="stat-value">{ &props.total }</span>
                <span class="stat-sub">{ "monitors" }</span>
            </div>
            <div class="stat-card">
                <span class="stat-label">{ "Operational" }</span>
                <span class="stat-value up">{ &props.up }</span>
                <span class="stat-sub">{ "monitors up" }</span>
            </div>
            <div class="stat-card">
                <span class="stat-label">{ "Down" }</span>
                <span class="stat-value down">{ &props.down }</span>
                <span class="stat-sub">{ "monitors down" }</span>
            </div>
            <div class="stat-card">
                <span class="stat-label">{ "Incidents" }</span>
                <span class="stat-value" style="color:var(--text)">{ &props.incidents }</span>
                <span class="stat-sub">{ "active now" }</span>
            </div>
        </div>
    }
}
