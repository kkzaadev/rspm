//! Process table rendering.

use rspm_core::types::{ProcessInfo, ProcessStatus};

/// Renders a process list as a compact table.
pub fn render_process_list(processes: &[ProcessInfo]) -> String {
    let mut rows = Vec::with_capacity(processes.len() + 1);
    rows.push(vec![
        "id".to_owned(),
        "name".to_owned(),
        "pid".to_owned(),
        "status".to_owned(),
        "restarts".to_owned(),
        "script".to_owned(),
    ]);

    for process in processes {
        rows.push(vec![
            process.pm_id.to_string(),
            process.name.clone(),
            process
                .pid
                .map(|pid| pid.to_string())
                .unwrap_or_else(|| "-".to_owned()),
            status_name(&process.status).to_owned(),
            process.restart_time.to_string(),
            process.script.display().to_string(),
        ]);
    }

    let widths = column_widths(&rows);
    rows.into_iter()
        .map(|row| {
            row.into_iter()
                .enumerate()
                .map(|(index, value)| format!("{value:<width$}", width = widths[index]))
                .collect::<Vec<_>>()
                .join("  ")
        })
        .collect::<Vec<_>>()
        .join("\n")
}

fn column_widths(rows: &[Vec<String>]) -> Vec<usize> {
    let columns = rows.first().map_or(0, Vec::len);
    let mut widths = vec![0; columns];
    for row in rows {
        for (index, value) in row.iter().enumerate() {
            widths[index] = widths[index].max(value.len());
        }
    }
    widths
}

fn status_name(status: &ProcessStatus) -> &'static str {
    match status {
        ProcessStatus::Online => "online",
        ProcessStatus::Stopping => "stopping",
        ProcessStatus::Stopped => "stopped",
        ProcessStatus::Errored => "errored",
        ProcessStatus::OneLaunchStatus => "one-launch-status",
        ProcessStatus::Launching => "launching",
        ProcessStatus::Waiting => "waiting restart",
    }
}
