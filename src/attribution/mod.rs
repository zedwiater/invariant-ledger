//! Energy attribution to cgroups and processes.
//!
//! Maps hardware Joules to software workloads via cgroup v2 CPU accounting.
//! The key insight: energy ∝ CPU time × power. We measure total power via
//! telemetry, then apportion by cgroup CPU usage.

use anyhow::Result;
use std::collections::HashMap;
use std::fs;

/// Per-cgroup CPU stats from cgroup v2
#[derive(Debug, Clone, Default)]
pub struct CgroupStats {
    pub usage_usec: u64,
    pub user_usec: u64,
    pub system_usec: u64,
}

/// Read cgroup v2 stats for all top-level cgroups
pub fn read_cgroup_stats() -> Result<HashMap<String, CgroupStats>> {
    let mut out = HashMap::new();
    let base = "/sys/fs/cgroup";

    let entries = fs::read_dir(base)?;
    for entry in entries {
        let entry = entry?;
        let path = entry.path();
        if !path.is_dir() {
            continue;
        }
        let name = entry.file_name().to_string_lossy().to_string();
        let stat_path = path.join("cpu.stat");

        if let Ok(content) = fs::read_to_string(&stat_path) {
            let mut stats = CgroupStats::default();
            for line in content.lines() {
                let parts: Vec<_> = line.split_whitespace().collect();
                if parts.len() == 2 {
                    match parts[0] {
                        "usage_usec" => stats.usage_usec = parts[1].parse().unwrap_or(0),
                        "user_usec" => stats.user_usec = parts[1].parse().unwrap_or(0),
                        "system_usec" => stats.system_usec = parts[1].parse().unwrap_or(0),
                        _ => {}
                    }
                }
            }
            out.insert(name, stats);
        }
    }
    Ok(out)
}

/// Attribute Joules to cgroups proportionally to CPU delta.
/// Returns map of cgroup name -> joules.
pub fn attribute_joules(
    total_joules: f64,
    prev: &HashMap<String, CgroupStats>,
    curr: &HashMap<String, CgroupStats>,
) -> HashMap<String, f64> {
    // Compute per-cgroup CPU deltas
    let mut deltas: HashMap<String, u64> = HashMap::new();
    let mut total_delta: u64 = 0;

    for (name, curr_stats) in curr {
        if let Some(prev_stats) = prev.get(name) {
            let d = curr_stats.usage_usec.saturating_sub(prev_stats.usage_usec);
            if d > 0 {
                deltas.insert(name.clone(), d);
                total_delta += d;
            }
        }
    }

    // Apportion joules by CPU share
    let mut out = HashMap::new();
    if total_delta > 0 {
        for (name, d) in deltas {
            let share = d as f64 / total_delta as f64;
            out.insert(name, total_joules * share);
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn apportions_proportionally() {
        let mut prev = HashMap::new();
        let mut curr = HashMap::new();
        prev.insert("a".into(), CgroupStats { usage_usec: 0, ..Default::default() });
        prev.insert("b".into(), CgroupStats { usage_usec: 0, ..Default::default() });
        curr.insert("a".into(), CgroupStats { usage_usec: 300, ..Default::default() });
        curr.insert("b".into(), CgroupStats { usage_usec: 100, ..Default::default() });

        let result = attribute_joules(40.0, &prev, &curr);
        assert!((result["a"] - 30.0).abs() < 1e-9); // 75% of CPU
        assert!((result["b"] - 10.0).abs() < 1e-9); // 25% of CPU
    }
}
