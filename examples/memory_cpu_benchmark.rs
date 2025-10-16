use concerto::{scheduled, SchedulerBuilder, Runnable};
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;
use std::time::{Duration, Instant};
use sysinfo::{System, Pid};
use chrono::Local;

static GLOBAL_COUNTER: AtomicU64 = AtomicU64::new(0);

/// Lightweight task for testing
#[scheduled(fixed_rate = 100)]
async fn fast_task() {
    GLOBAL_COUNTER.fetch_add(1, Ordering::Relaxed);
}

/// Moderate task
#[scheduled(fixed_rate = 500)]
async fn moderate_task() {
    let _result: u64 = (0..100).map(|i| i * i).sum();
    GLOBAL_COUNTER.fetch_add(1, Ordering::Relaxed);
}

/// Task with some I/O simulation
#[scheduled(fixed_rate = "1s")]
async fn io_task() {
    tokio::time::sleep(Duration::from_millis(10)).await;
    GLOBAL_COUNTER.fetch_add(1, Ordering::Relaxed);
}

/// Runnable implementation for benchmarking
struct BenchmarkTask {
    counter: Arc<AtomicU64>,
}

#[scheduled(fixed_rate = 200)]
impl Runnable for BenchmarkTask {
    fn run(&self) {
        // Simulate work
        let mut sum = 0u64;
        for i in 0..50 {
            sum = sum.wrapping_add(i);
        }
        self.counter.fetch_add(sum, Ordering::Relaxed);
    }
}

struct MemoryCpuMonitor {
    system: System,
    pid: Pid,
    start_time: Instant,
    samples: Vec<Sample>,
}

#[derive(Debug, Clone)]
struct Sample {
    timestamp: Duration,
    memory_kb: u64,
    cpu_percent: f32,
    task_count: u64,
}

impl MemoryCpuMonitor {
    fn new() -> Self {
        let mut system = System::new_all();
        system.refresh_all();
        
        let pid = sysinfo::get_current_pid().unwrap();
        
        Self {
            system,
            pid,
            start_time: Instant::now(),
            samples: Vec::new(),
        }
    }
    
    fn sample(&mut self, task_count: u64) {
        self.system.refresh_process(self.pid);
        
        if let Some(process) = self.system.process(self.pid) {
            let sample = Sample {
                timestamp: self.start_time.elapsed(),
                memory_kb: process.memory() / 1024,
                cpu_percent: process.cpu_usage(),
                task_count,
            };
            
            self.samples.push(sample);
        }
    }
    
    fn print_statistics(&self) {
        if self.samples.is_empty() {
            println!("No samples collected");
            return;
        }
        
        println!("\n{}", "=".repeat(80));
        println!("📊 MEMORY AND CPU STATISTICS");
        println!("{}", "=".repeat(80));
        
        // Memory statistics
        let mem_values: Vec<u64> = self.samples.iter().map(|s| s.memory_kb).collect();
        let min_mem = mem_values.iter().min().unwrap();
        let max_mem = mem_values.iter().max().unwrap();
        let avg_mem = mem_values.iter().sum::<u64>() / mem_values.len() as u64;
        
        println!("\n💾 MEMORY USAGE:");
        println!("   Minimum:  {:>10} KB ({:>8.2} MB)", min_mem, *min_mem as f64 / 1024.0);
        println!("   Maximum:  {:>10} KB ({:>8.2} MB)", max_mem, *max_mem as f64 / 1024.0);
        println!("   Average:  {:>10} KB ({:>8.2} MB)", avg_mem, avg_mem as f64 / 1024.0);
        println!("   Growth:   {:>10} KB ({:>8.2} MB)", 
                 max_mem - min_mem, 
                 (*max_mem - *min_mem) as f64 / 1024.0);
        
        // CPU statistics
        let cpu_values: Vec<f32> = self.samples.iter().map(|s| s.cpu_percent).collect();
        let min_cpu = cpu_values.iter().fold(f32::INFINITY, |a, &b| a.min(b));
        let max_cpu = cpu_values.iter().fold(f32::NEG_INFINITY, |a, &b| a.max(b));
        let avg_cpu = cpu_values.iter().sum::<f32>() / cpu_values.len() as f32;
        
        println!("\n⚡ CPU USAGE:");
        println!("   Minimum:  {:>10.2}%", min_cpu);
        println!("   Maximum:  {:>10.2}%", max_cpu);
        println!("   Average:  {:>10.2}%", avg_cpu);
        
        // Task execution statistics
        let first_count = self.samples.first().unwrap().task_count;
        let last_count = self.samples.last().unwrap().task_count;
        let total_executions = last_count - first_count;
        let duration = self.samples.last().unwrap().timestamp.as_secs_f64();
        
        println!("\n📈 TASK EXECUTION:");
        println!("   Total executions: {}", total_executions);
        println!("   Duration:         {:.2} seconds", duration);
        println!("   Rate:             {:.2} tasks/second", total_executions as f64 / duration);
        
        println!("\n📉 DETAILED TIMELINE:");
        println!("   {:>8} | {:>12} | {:>10} | {:>10}", "Time(s)", "Memory(KB)", "CPU(%)", "Tasks");
        println!("   {}", "-".repeat(50));
        
        for (i, sample) in self.samples.iter().enumerate() {
            if i % 5 == 0 || i == self.samples.len() - 1 {
                println!("   {:>8.1} | {:>12} | {:>10.2} | {:>10}", 
                         sample.timestamp.as_secs_f64(),
                         sample.memory_kb,
                         sample.cpu_percent,
                         sample.task_count);
            }
        }
        
        println!("\n{}", "=".repeat(80));
    }
    
    fn save_csv(&self, filename: &str) -> std::io::Result<()> {
        use std::io::Write;
        let mut file = std::fs::File::create(filename)?;
        
        writeln!(file, "timestamp_sec,memory_kb,memory_mb,cpu_percent,task_count")?;
        
        for sample in &self.samples {
            writeln!(file, "{:.3},{},{:.2},{:.2},{}", 
                     sample.timestamp.as_secs_f64(),
                     sample.memory_kb,
                     sample.memory_kb as f64 / 1024.0,
                     sample.cpu_percent,
                     sample.task_count)?;
        }
        
        Ok(())
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🚀 Concerto Library - Memory & CPU Benchmark");
    println!("============================================\n");
    
    let mut monitor = MemoryCpuMonitor::new();
    
    // Initial baseline
    println!("📝 Collecting baseline metrics...");
    monitor.sample(0);
    tokio::time::sleep(Duration::from_secs(1)).await;
    
    // Create scheduler with multiple tasks
    println!("🔧 Setting up scheduler with multiple tasks...");
    
    let task_counter = Arc::new(AtomicU64::new(0));
    let mut builder = SchedulerBuilder::new();
    
    // Register 10 runnable tasks
    for _ in 0..10 {
        builder = builder.register(BenchmarkTask {
            counter: Arc::clone(&task_counter),
        });
    }
    
    let scheduler = builder.build();
    
    println!("✅ Scheduler built with:");
    println!("   - 3 scheduled functions (fast_task, moderate_task, io_task)");
    println!("   - 10 runnable tasks");
    println!("\n⏱️  Starting benchmark for 30 seconds...\n");
    
    let _handle = scheduler.start().await?;
    
    // Monitor for 30 seconds
    let monitoring_duration = Duration::from_secs(30);
    let sample_interval = Duration::from_millis(500);
    let num_samples = monitoring_duration.as_millis() / sample_interval.as_millis();
    
    for i in 0..num_samples {
        tokio::time::sleep(sample_interval).await;
        
        let total_tasks = GLOBAL_COUNTER.load(Ordering::Relaxed) 
                        + task_counter.load(Ordering::Relaxed);
        
        monitor.sample(total_tasks);
        
        // Print progress
        if (i + 1) % 4 == 0 {
            let elapsed = (i + 1) * sample_interval.as_millis() / 1000;
            let now = Local::now().format("%H:%M:%S");
            println!("[{}] ⏱️  {} seconds elapsed - {} tasks executed", 
                     now, elapsed, total_tasks);
        }
    }
    
    // Final statistics
    let total_tasks = GLOBAL_COUNTER.load(Ordering::Relaxed) 
                    + task_counter.load(Ordering::Relaxed);
    
    println!("\n✅ Benchmark completed!");
    println!("   Total task executions: {}", total_tasks);
    
    // Print and save results
    monitor.print_statistics();
    
    let csv_filename = format!("benchmark_results_{}.csv", 
                               Local::now().format("%Y%m%d_%H%M%S"));
    
    match monitor.save_csv(&csv_filename) {
        Ok(_) => println!("\n💾 Results saved to: {}", csv_filename),
        Err(e) => println!("\n❌ Failed to save CSV: {}", e),
    }
    
    Ok(())
}
