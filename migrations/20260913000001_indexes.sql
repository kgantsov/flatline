-- monitor_checks: covers list_for_monitor (WHERE monitor_id ORDER BY checked_at DESC)
--                 and latency_percentiles (WHERE monitor_id AND checked_at > ?)
--                 and the sweeper delete (WHERE checked_at < ?)
CREATE INDEX idx_monitor_checks_monitor_checked
    ON monitor_checks(monitor_id, checked_at DESC);

CREATE INDEX idx_monitor_checks_checked_at
    ON monitor_checks(checked_at);

-- incidents: covers list_for_monitor (WHERE monitor_id ORDER BY started_at DESC)
--            and uptime_percentage (WHERE monitor_id AND started_at range)
CREATE INDEX idx_incidents_monitor_started
    ON incidents(monitor_id, started_at DESC);

-- incidents: covers get_open_for_monitor (WHERE monitor_id AND resolved_at IS NULL)
CREATE INDEX idx_incidents_monitor_resolved
    ON incidents(monitor_id, resolved_at);
