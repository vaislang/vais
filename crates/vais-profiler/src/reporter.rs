use crate::ProfileSnapshot;
use serde::{Deserialize, Serialize};
use std::fmt;

pub struct TextReport {
    snapshot: ProfileSnapshot,
}

impl TextReport {
    pub fn new(snapshot: ProfileSnapshot) -> Self {
        Self { snapshot }
    }

    pub fn generate(&self) -> String {
        let mut output = String::new();

        output.push_str("=== Vais Performance Profile ===\n\n");

        if let Some(duration) = self.snapshot.duration {
            output.push_str(&format!("Duration: {:.3}s\n\n", duration.as_secs_f64()));
        }

        self.write_cpu_profile(&mut output);
        self.write_memory_profile(&mut output);
        self.write_call_graph(&mut output);

        output
    }

    fn write_cpu_profile(&self, output: &mut String) {
        output.push_str("--- CPU Profile ---\n");

        if self.snapshot.samples.is_empty() {
            output.push_str("No samples collected\n\n");
            return;
        }

        let total_samples: usize = self.snapshot.samples.iter().map(|(_, count)| count).sum();
        output.push_str(&format!("Total samples: {}\n\n", total_samples));

        output.push_str("Hot functions:\n");
        output.push_str(&format!(
            "{:<40} {:>12} {:>10}\n",
            "Function", "Samples", "Percentage"
        ));
        output.push_str(&"-".repeat(65));
        output.push('\n');

        for (name, count) in self.snapshot.samples.iter().take(20) {
            let percentage = (*count as f64 / total_samples as f64) * 100.0;
            output.push_str(&format!(
                "{:<40} {:>12} {:>9.2}%\n",
                name, count, percentage
            ));
        }
        output.push('\n');
    }

    fn write_memory_profile(&self, output: &mut String) {
        output.push_str("--- Memory Profile ---\n");

        let stats = &self.snapshot.memory_stats;

        output.push_str(&format!(
            "Total allocations:     {}\n",
            stats.total_allocations
        ));
        output.push_str(&format!(
            "Total deallocations:   {}\n",
            stats.total_deallocations
        ));
        output.push_str(&format!(
            "Total allocated:       {} bytes ({:.2} MB)\n",
            stats.total_allocated_bytes,
            stats.total_allocated_bytes as f64 / 1_048_576.0
        ));
        output.push_str(&format!(
            "Current allocated:     {} bytes ({:.2} MB)\n",
            stats.current_allocated_bytes,
            stats.current_allocated_bytes as f64 / 1_048_576.0
        ));
        output.push_str(&format!(
            "Peak allocated:        {} bytes ({:.2} MB)\n",
            stats.peak_allocated_bytes,
            stats.peak_allocated_bytes as f64 / 1_048_576.0
        ));

        if stats.total_allocations > 0 {
            let avg_size = stats.total_allocated_bytes / stats.total_allocations;
            output.push_str(&format!("Average allocation:    {} bytes\n", avg_size));
        }

        output.push('\n');
    }

    fn write_call_graph(&self, output: &mut String) {
        output.push_str("--- Call Graph (Top 20 edges) ---\n");

        if self.snapshot.call_graph.is_empty() {
            output.push_str("No call graph data\n\n");
            return;
        }

        output.push_str(&format!(
            "{:<30} -> {:<30} {:>10}\n",
            "Caller", "Callee", "Count"
        ));
        output.push_str(&"-".repeat(75));
        output.push('\n');

        let mut edges = self.snapshot.call_graph.clone();
        edges.sort_by(|a, b| b.2.cmp(&a.2));

        for (caller, callee, count) in edges.iter().take(20) {
            output.push_str(&format!("{:<30} -> {:<30} {:>10}\n", caller, callee, count));
        }
        output.push('\n');
    }
}

impl fmt::Display for TextReport {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.generate())
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProfileStats {
    pub duration_secs: f64,
    pub total_samples: usize,
    pub hot_functions: Vec<FunctionStats>,
    pub memory: MemoryStatsReport,
    pub call_graph_edges: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FunctionStats {
    pub name: String,
    pub samples: usize,
    pub percentage: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemoryStatsReport {
    pub total_allocations: usize,
    pub total_deallocations: usize,
    pub total_allocated_bytes: usize,
    pub current_allocated_bytes: usize,
    pub peak_allocated_bytes: usize,
    pub average_allocation_size: usize,
}

impl ProfileStats {
    pub fn from_snapshot(snapshot: &ProfileSnapshot) -> Self {
        let duration_secs = snapshot.duration.map(|d| d.as_secs_f64()).unwrap_or(0.0);

        let total_samples: usize = snapshot.samples.iter().map(|(_, count)| count).sum();

        let hot_functions = snapshot
            .samples
            .iter()
            .take(10)
            .map(|(name, samples)| {
                let percentage = if total_samples > 0 {
                    (*samples as f64 / total_samples as f64) * 100.0
                } else {
                    0.0
                };
                FunctionStats {
                    name: name.clone(),
                    samples: *samples,
                    percentage,
                }
            })
            .collect();

        let average_allocation_size = if snapshot.memory_stats.total_allocations > 0 {
            snapshot.memory_stats.total_allocated_bytes / snapshot.memory_stats.total_allocations
        } else {
            0
        };

        Self {
            duration_secs,
            total_samples,
            hot_functions,
            memory: MemoryStatsReport {
                total_allocations: snapshot.memory_stats.total_allocations,
                total_deallocations: snapshot.memory_stats.total_deallocations,
                total_allocated_bytes: snapshot.memory_stats.total_allocated_bytes,
                current_allocated_bytes: snapshot.memory_stats.current_allocated_bytes,
                peak_allocated_bytes: snapshot.memory_stats.peak_allocated_bytes,
                average_allocation_size,
            },
            call_graph_edges: snapshot.call_graph.len(),
        }
    }

    pub fn to_json(&self) -> Result<String, serde_json::Error> {
        serde_json::to_string_pretty(self)
    }
}

#[cfg(feature = "flamegraph")]
pub struct FlameGraphData {
    snapshot: ProfileSnapshot,
}

#[cfg(feature = "flamegraph")]
impl FlameGraphData {
    pub fn new(snapshot: ProfileSnapshot) -> Self {
        Self { snapshot }
    }

    pub fn generate_folded(&self) -> String {
        let mut lines = Vec::new();

        for (function, count) in &self.snapshot.samples {
            lines.push(format!("{} {}", function, count));
        }

        for (caller, callee, count) in &self.snapshot.call_graph {
            lines.push(format!("{};{} {}", caller, callee, count));
        }

        lines.join("\n")
    }

    pub fn write_svg<W: std::io::Write>(
        &self,
        writer: W,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let folded_data = self.generate_folded();
        let lines: Vec<_> = folded_data.lines().map(|s| s.to_string()).collect();

        let mut options = inferno::flamegraph::Options::default();
        inferno::flamegraph::from_lines(&mut options, lines.iter().map(|s| s.as_str()), writer)?;

        Ok(())
    }
}

pub struct CompactReport {
    snapshot: ProfileSnapshot,
}

impl CompactReport {
    pub fn new(snapshot: ProfileSnapshot) -> Self {
        Self { snapshot }
    }

    pub fn generate(&self) -> String {
        let mut output = String::new();

        if let Some(duration) = self.snapshot.duration {
            output.push_str(&format!("Duration: {:.3}s | ", duration.as_secs_f64()));
        }

        let total_samples: usize = self.snapshot.samples.iter().map(|(_, count)| count).sum();
        output.push_str(&format!("Samples: {} | ", total_samples));

        output.push_str(&format!(
            "Memory: peak {:.2}MB | ",
            self.snapshot.memory_stats.peak_allocated_bytes as f64 / 1_048_576.0
        ));

        output.push_str(&format!("Edges: {}", self.snapshot.call_graph.len()));

        if let Some((name, count)) = self.snapshot.samples.first() {
            let percentage = if total_samples > 0 {
                (*count as f64 / total_samples as f64) * 100.0
            } else {
                0.0
            };
            output.push_str(&format!("\nHottest: {} ({:.1}%)", name, percentage));
        }

        output
    }
}

impl fmt::Display for CompactReport {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.generate())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::MemoryStats;
    use std::time::Duration;

    fn create_test_snapshot() -> ProfileSnapshot {
        ProfileSnapshot {
            duration: Some(Duration::from_secs(5)),
            samples: vec![
                ("main".to_string(), 100),
                ("foo".to_string(), 50),
                ("bar".to_string(), 30),
            ],
            memory_stats: MemoryStats {
                total_allocations: 1000,
                total_deallocations: 900,
                total_allocated_bytes: 1_048_576,
                current_allocated_bytes: 104_857,
                peak_allocated_bytes: 524_288,
            },
            call_graph: vec![
                ("main".to_string(), "foo".to_string(), 50),
                ("main".to_string(), "bar".to_string(), 30),
                ("foo".to_string(), "baz".to_string(), 20),
            ],
        }
    }

    #[test]
    fn test_text_report() {
        let snapshot = create_test_snapshot();
        let report = TextReport::new(snapshot);
        let output = report.generate();

        assert!(output.contains("Duration: 5.000s"));
        assert!(output.contains("Total samples: 180"));
        assert!(output.contains("main"));
        assert!(output.contains("foo"));
        assert!(output.contains("bar"));
        assert!(output.contains("Total allocations:     1000"));
        assert!(output.contains("Peak allocated:"));
    }

    #[test]
    fn test_profile_stats() {
        let snapshot = create_test_snapshot();
        let stats = ProfileStats::from_snapshot(&snapshot);

        assert_eq!(stats.duration_secs, 5.0);
        assert_eq!(stats.total_samples, 180);
        assert_eq!(stats.hot_functions.len(), 3);
        assert_eq!(stats.hot_functions[0].name, "main");
        assert_eq!(stats.hot_functions[0].samples, 100);
        assert!((stats.hot_functions[0].percentage - 55.55).abs() < 0.1);
        assert_eq!(stats.memory.total_allocations, 1000);
        assert_eq!(stats.memory.average_allocation_size, 1048);
        assert_eq!(stats.call_graph_edges, 3);
    }

    #[test]
    fn test_profile_stats_json() {
        let snapshot = create_test_snapshot();
        let stats = ProfileStats::from_snapshot(&snapshot);
        let json = stats.to_json().unwrap();

        assert!(json.contains("duration_secs"));
        assert!(json.contains("hot_functions"));
        assert!(json.contains("memory"));
    }

    #[cfg(feature = "flamegraph")]
    #[test]
    fn test_flamegraph_data() {
        let snapshot = create_test_snapshot();
        let flamegraph = FlameGraphData::new(snapshot);
        let folded = flamegraph.generate_folded();

        assert!(folded.contains("main 100"));
        assert!(folded.contains("foo 50"));
        assert!(folded.contains("main;foo 50"));
    }

    #[test]
    fn test_compact_report() {
        let snapshot = create_test_snapshot();
        let report = CompactReport::new(snapshot);
        let output = report.generate();

        assert!(output.contains("Duration: 5.000s"));
        assert!(output.contains("Samples: 180"));
        assert!(output.contains("Memory: peak"));
        assert!(output.contains("Edges: 3"));
        assert!(output.contains("Hottest: main"));
    }

    #[test]
    fn test_text_report_empty() {
        let snapshot = ProfileSnapshot {
            duration: None,
            samples: vec![],
            memory_stats: MemoryStats::default(),
            call_graph: vec![],
        };
        let report = TextReport::new(snapshot);
        let output = report.generate();

        assert!(output.contains("No samples collected"));
        assert!(output.contains("No call graph data"));
    }
}
