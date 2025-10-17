use std::collections::HashMap;
use std::time::{Duration, Instant};
use serde::{Deserialize, Serialize};

/// Performance profiling data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProfileData {
    pub function_name: String,
    pub call_count: u64,
    pub total_time: Duration,
    pub min_time: Duration,
    pub max_time: Duration,
    pub avg_time: Duration,
}

/// Performance profiler
pub struct Profiler {
    entries: HashMap<String, ProfileEntry>,
    active_timers: HashMap<String, Instant>,
}

#[derive(Debug, Clone)]
struct ProfileEntry {
    call_count: u64,
    total_time: Duration,
    min_time: Duration,
    max_time: Duration,
}

impl Profiler {
    pub fn new() -> Self {
        Self {
            entries: HashMap::new(),
            active_timers: HashMap::new(),
        }
    }

    /// Start timing a function
    pub fn start(&mut self, function_name: &str) {
        self.active_timers.insert(function_name.to_string(), Instant::now());
    }

    /// Stop timing a function and record the duration
    pub fn stop(&mut self, function_name: &str) {
        if let Some(start_time) = self.active_timers.remove(function_name) {
            let elapsed = start_time.elapsed();

            self.entries
                .entry(function_name.to_string())
                .and_modify(|entry| {
                    entry.call_count += 1;
                    entry.total_time += elapsed;
                    entry.min_time = entry.min_time.min(elapsed);
                    entry.max_time = entry.max_time.max(elapsed);
                })
                .or_insert(ProfileEntry {
                    call_count: 1,
                    total_time: elapsed,
                    min_time: elapsed,
                    max_time: elapsed,
                });
        }
    }

    /// Get profiling data for all functions
    pub fn get_data(&self) -> Vec<ProfileData> {
        self.entries
            .iter()
            .map(|(name, entry)| ProfileData {
                function_name: name.clone(),
                call_count: entry.call_count,
                total_time: entry.total_time,
                min_time: entry.min_time,
                max_time: entry.max_time,
                avg_time: entry.total_time / entry.call_count as u32,
            })
            .collect()
    }

    /// Print profiling summary
    pub fn print_summary(&self) {
        let mut data = self.get_data();
        data.sort_by(|a, b| b.total_time.cmp(&a.total_time));

        println!("\n📊 Performance Profile:");
        println!("{:<30} {:>10} {:>15} {:>15} {:>15} {:>15}",
            "Function", "Calls", "Total (ms)", "Avg (ms)", "Min (ms)", "Max (ms)");
        println!("{:-<100}", "");

        for entry in data {
            println!(
                "{:<30} {:>10} {:>15.2} {:>15.2} {:>15.2} {:>15.2}",
                entry.function_name,
                entry.call_count,
                entry.total_time.as_secs_f64() * 1000.0,
                entry.avg_time.as_secs_f64() * 1000.0,
                entry.min_time.as_secs_f64() * 1000.0,
                entry.max_time.as_secs_f64() * 1000.0,
            );
        }
    }

    /// Reset all profiling data
    pub fn reset(&mut self) {
        self.entries.clear();
        self.active_timers.clear();
    }
}

impl Default for Profiler {
    fn default() -> Self {
        Self::new()
    }
}

/// RAII-style profiling guard
pub struct ProfileGuard<'a> {
    profiler: &'a mut Profiler,
    function_name: String,
}

impl<'a> ProfileGuard<'a> {
    pub fn new(profiler: &'a mut Profiler, function_name: String) -> Self {
        profiler.start(&function_name);
        Self {
            profiler,
            function_name,
        }
    }
}

impl<'a> Drop for ProfileGuard<'a> {
    fn drop(&mut self) {
        self.profiler.stop(&self.function_name);
    }
}

/// Macro for easy profiling
#[macro_export]
macro_rules! profile {
    ($profiler:expr, $name:expr, $code:block) => {{
        let _guard = ProfileGuard::new($profiler, $name.to_string());
        $code
    }};
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::thread;

    #[test]
    fn test_profiler() {
        let mut profiler = Profiler::new();

        profiler.start("test_function");
        thread::sleep(Duration::from_millis(10));
        profiler.stop("test_function");

        profiler.start("test_function");
        thread::sleep(Duration::from_millis(20));
        profiler.stop("test_function");

        let data = profiler.get_data();
        assert_eq!(data.len(), 1);

        let entry = &data[0];
        assert_eq!(entry.function_name, "test_function");
        assert_eq!(entry.call_count, 2);
        assert!(entry.total_time >= Duration::from_millis(30));
    }

    #[test]
    fn test_profile_guard() {
        let mut profiler = Profiler::new();

        {
            let _guard = ProfileGuard::new(&mut profiler, "guarded_function".to_string());
            thread::sleep(Duration::from_millis(5));
        }

        let data = profiler.get_data();
        assert_eq!(data.len(), 1);
        assert_eq!(data[0].function_name, "guarded_function");
        assert_eq!(data[0].call_count, 1);
    }
}
