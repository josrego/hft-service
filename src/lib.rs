use std::collections::VecDeque;
use std::sync::Arc;

use tokio::sync::RwLock;

pub struct TradingDataBuffer {
    values: VecDeque<f64>,
    capacity: usize,
    min: f64,
    max: f64,
    sum: f64,
    sum_squares: f64,
}

impl TradingDataBuffer {
    pub fn new(capacity: usize) -> Self {
        TradingDataBuffer {
            values: VecDeque::with_capacity(capacity),
            capacity,
            min: f64::MAX,
            max: f64::MIN,
            sum: 0.0,
            sum_squares: 0.0,
        }
    }

    pub fn add_batch(&mut self, new_values: &[f64]) {
        for &value in new_values {
            self.add(value);
        }
    }

    fn add(&mut self, value: f64) {
        if self.values.len() >= self.capacity {
            let old_value = self.values.pop_front().unwrap();
            self.sum -= old_value;
            self.sum_squares -= old_value * old_value;
            if old_value == self.min || old_value == self.max {
                self.recalculate_min_max();
            }
        }

        self.values.push_back(value);
        self.sum += value;
        self.sum_squares += value * value;
        self.min = self.min.min(value);
        self.max = self.max.max(value);
    }

    fn recalculate_min_max(&mut self) {
        let (min, max) = self.values.iter().fold((f64::MAX, f64::MIN), |(min, max), &v| {
            (min.min(v), max.max(v))
        });
        self.min = min;
        self.max = max;
    }

    pub fn get_stats(&self) -> StatsResponse {
        if self.values.is_empty() {
            return StatsResponse::default();
        }
        let avg = self.sum / self.values.len() as f64;
        let variance = (self.sum_squares / self.values.len() as f64) - (avg * avg);
        let last = *self.values.back().unwrap();
        StatsResponse {
            min: self.min,
            max: self.max,
            last,
            avg,
            var: variance,
        }
    }
}

#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub struct StatsResponse {
    pub min: f64,
    pub max: f64,
    pub last: f64,
    pub avg: f64,
    pub var: f64,
}

impl Default for StatsResponse {
    fn default() -> Self {
        StatsResponse {
            min: 0.0,
            max: 0.0,
            last: 0.0,
            avg: 0.0,
            var: 0.0,
        }
    }
}

pub struct TradingDataService {
    buffers: Arc<RwLock<std::collections::HashMap<String, Vec<TradingDataBuffer>>>>,
}

impl TradingDataService {
    pub fn new() -> Self {
        TradingDataService {
            buffers: Arc::new(RwLock::new(std::collections::HashMap::new())),
        }
    }

    pub async fn add_batch_values(&self, symbol: String, values: Vec<f64>) -> Result<(), String> {
        if values.len() > 10000 {
            return Err("Batch size exceeds maximum limit of 10000".to_string());
        }

        let mut buffers = self.buffers.write().await;
        let symbol_buffers = buffers.entry(symbol).or_insert_with(|| {
            (1..=8).map(|k| TradingDataBuffer::new(10usize.pow(k))).collect()
        });

        for buffer in symbol_buffers.iter_mut() {
            buffer.add_batch(&values);
        }

        Ok(())
    }

    pub async fn get_stats(&self, symbol: String, k: usize) -> Result<StatsResponse, String> {
        if k < 1 || k > 8 {
            return Err("Invalid k input. Only values 1-8 are accepted.".to_string());
        }

        let buffers = self.buffers.read().await;
        buffers.get(&symbol)
            .and_then(|b| b.get(k - 1))
            .map(|b| b.get_stats())
            .ok_or_else(|| "Symbol not found".to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const DELTA: f64 = 1e-6;

    fn assert_float_eq(a: f64, b: f64) {
        assert!((a - b).abs() < DELTA, "{} != {}", a, b);
    }

    #[test]
    fn test_add_batch_and_get_stats() {
        let mut buffer = TradingDataBuffer::new(5);
        buffer.add_batch(&[1.0, 2.0, 3.0]);

        let stats = buffer.get_stats();
        assert_float_eq(1.0, stats.min);
        assert_float_eq(3.0, stats.max);
        assert_float_eq(3.0, stats.last);
        assert_float_eq(2.0, stats.avg);
        assert_float_eq(0.6666667, stats.var);
    }

    #[test]
    fn test_buffer_overflow() {
        let mut buffer = TradingDataBuffer::new(5);
        buffer.add_batch(&[1.0, 2.0, 3.0, 4.0, 5.0, 6.0]);

        let stats = buffer.get_stats();
        assert_float_eq(2.0, stats.min);
        assert_float_eq(6.0, stats.max);
        assert_float_eq(6.0, stats.last);
        assert_float_eq(4.0, stats.avg);
    }

    #[test]
    fn test_empty_buffer() {
        let buffer = TradingDataBuffer::new(5);
        let stats = buffer.get_stats();
        assert_float_eq(0.0, stats.min);
        assert_float_eq(0.0, stats.max);
        assert_float_eq(0.0, stats.last);
        assert_float_eq(0.0, stats.avg);
        assert_float_eq(0.0, stats.var);
    }

    #[test]
    fn test_single_element() {
        let mut buffer = TradingDataBuffer::new(5);
        buffer.add_batch(&[5.0]);

        let stats = buffer.get_stats();
        assert_float_eq(5.0, stats.min);
        assert_float_eq(5.0, stats.max);
        assert_float_eq(5.0, stats.last);
        assert_float_eq(5.0, stats.avg);
        assert_float_eq(0.0, stats.var);
    }

    #[test]
    fn test_variance_calculation() {
        let mut buffer = TradingDataBuffer::new(5);
        buffer.add_batch(&[2.0, 4.0, 6.0]);

        let stats = buffer.get_stats();
        assert_float_eq(4.0, stats.avg);
        assert_float_eq(2.6666667, stats.var);
    }

    #[test]
    fn test_min_max_recalculation() {
        let mut buffer = TradingDataBuffer::new(5);
        buffer.add_batch(&[3.0, 1.0, 5.0, 2.0, 4.0]);
        buffer.add_batch(&[6.0, 3.0]);

        let stats = buffer.get_stats();
        assert_float_eq(2.0, stats.min);
        assert_float_eq(6.0, stats.max);
    }

    #[test]
    fn test_large_number_of_additions() {
        let mut buffer = TradingDataBuffer::new(1000);
        let large_array: Vec<f64> = (0..1000).map(|i| i as f64).collect();
        buffer.add_batch(&large_array);

        let stats = buffer.get_stats();
        assert_float_eq(0.0, stats.min);
        assert_float_eq(999.0, stats.max);
        assert_float_eq(999.0, stats.last);
        assert_float_eq(499.5, stats.avg);
    }

    #[test]
    fn test_multiple_add_batches() {
        let mut buffer = TradingDataBuffer::new(5);
        buffer.add_batch(&[1.0, 2.0]);
        buffer.add_batch(&[3.0, 4.0]);
        buffer.add_batch(&[5.0]);

        let stats = buffer.get_stats();
        assert_float_eq(1.0, stats.min);
        assert_float_eq(5.0, stats.max);
        assert_float_eq(5.0, stats.last);
        assert_float_eq(3.0, stats.avg);
    }
}
