use std::fs;

use crate::module::config::BENCHMARK_EXT;

#[derive(Clone)]
pub struct BenchmarkStatistics {
    pub analysis_time: u128,
    pub text_export_time: u128,
    pub graphics_export_time: u128,
    pub total_time: u128,
    pub total_node_count: usize,
    pub total_edge_count: usize,
}

impl BenchmarkStatistics {
    pub fn new() -> Self {
        Self {
            analysis_time: 0,
            text_export_time: 0,
            graphics_export_time: 0,
            total_time: 0,
            total_node_count: 0,
            total_edge_count: 0,
        }
    }
}

pub fn write_benchmarks(benchmark_statistics: &BenchmarkStatistics) {
    let benchmark_string = format!(
        "Analysis Time: {} ms\nText Export Time: {} ms\nGraphics Export Time: {} ms\nTotal Time: {} ms\nTotal Node Count: {}\nTotal Edge Count: {}",
        benchmark_statistics.analysis_time,
        benchmark_statistics.text_export_time,
        benchmark_statistics.graphics_export_time,
        benchmark_statistics.total_time,
        benchmark_statistics.total_node_count,
        benchmark_statistics.total_edge_count
    );

    fs::write(
        format!("workspace/benchmarks{BENCHMARK_EXT}.txt"),
        benchmark_string,
    )
    .unwrap();
}
