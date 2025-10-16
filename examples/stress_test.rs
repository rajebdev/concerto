use concerto::{scheduled, SchedulerBuilder, Runnable};
use std::sync::atomic::{AtomicU64, AtomicBool, Ordering};
use std::sync::Arc;
use std::time::{Duration, Instant};
use chrono::Local;
use tokio::signal;

// Global counters for different task types
static FAST_COUNTER: AtomicU64 = AtomicU64::new(0);
static MEDIUM_COUNTER: AtomicU64 = AtomicU64::new(0);
static SLOW_COUNTER: AtomicU64 = AtomicU64::new(0);

/// Ultra-fast task (50ms interval)
#[scheduled(fixed_rate = 50)]
async fn ultra_fast_task() {
    FAST_COUNTER.fetch_add(1, Ordering::Relaxed);
}

/// Fast task (100ms interval)
#[scheduled(fixed_rate = 100)]
async fn fast_task() {
    FAST_COUNTER.fetch_add(1, Ordering::Relaxed);
}

/// Medium task with computation (500ms interval)
#[scheduled(fixed_rate = 500)]
async fn medium_task() {
    let _sum: u64 = (0..1000).map(|i| i * i).sum();
    MEDIUM_COUNTER.fetch_add(1, Ordering::Relaxed);
}

/// Slow task with async operations (2s interval)
#[scheduled(fixed_rate = "2s")]
async fn slow_task() {
    tokio::time::sleep(Duration::from_millis(50)).await;
    SLOW_COUNTER.fetch_add(1, Ordering::Relaxed);
}

/// Cron-based task (every second)
#[scheduled(cron = "* * * * * *")]
async fn cron_task() {
    SLOW_COUNTER.fetch_add(1, Ordering::Relaxed);
}

/// High-frequency runnable task
struct HighFrequencyTask {
    counter: Arc<AtomicU64>,
}

#[scheduled(fixed_rate = 75)]
impl Runnable for HighFrequencyTask {
    fn run(&self) {
        self.counter.fetch_add(1, Ordering::Relaxed);
    }
}

/// CPU-intensive task
struct CpuIntensiveTask {
    counter: Arc<AtomicU64>,
}

#[scheduled(fixed_rate = 250)]
impl Runnable for CpuIntensiveTask {
    fn run(&self) {
        // Simulate CPU-intensive work
        let mut result = 0u64;
        for i in 0u64..5000 {
            result = result.wrapping_add(i.wrapping_mul(i));
        }
        self.counter.fetch_add(1, Ordering::Relaxed);
        // Use result to prevent optimization
        std::hint::black_box(result);
    }
}

/// Memory-intensive task
struct MemoryIntensiveTask {
    counter: Arc<AtomicU64>,
    data: Vec<u64>,
}

#[scheduled(fixed_rate = 1000)]
impl Runnable for MemoryIntensiveTask {
    fn run(&self) {
        // Create and manipulate some data
        let mut temp_data = self.data.clone();
        temp_data.sort();
        temp_data.reverse();
        let _sum: u64 = temp_data.iter().sum();
        
        self.counter.fetch_add(1, Ordering::Relaxed);
    }
}

struct StressTestMonitor {
    start_time: Instant,
    running: Arc<AtomicBool>,
}

impl StressTestMonitor {
    fn new() -> Self {
        Self {
            start_time: Instant::now(),
            running: Arc::new(AtomicBool::new(true)),
        }
    }
    
    fn print_status(&self, 
                    high_freq_counter: &Arc<AtomicU64>,
                    cpu_counter: &Arc<AtomicU64>,
                    mem_counter: &Arc<AtomicU64>) {
        let elapsed = self.start_time.elapsed();
        let elapsed_secs = elapsed.as_secs_f64();
        
        let fast = FAST_COUNTER.load(Ordering::Relaxed);
        let medium = MEDIUM_COUNTER.load(Ordering::Relaxed);
        let slow = SLOW_COUNTER.load(Ordering::Relaxed);
        let high_freq = high_freq_counter.load(Ordering::Relaxed);
        let cpu = cpu_counter.load(Ordering::Relaxed);
        let mem = mem_counter.load(Ordering::Relaxed);
        
        let total = fast + medium + slow + high_freq + cpu + mem;
        let rate = total as f64 / elapsed_secs;
        
        println!("\n{}", "=".repeat(80));
        println!("⏱️  ELAPSED TIME: {:.2}s", elapsed_secs);
        println!("{}", "=".repeat(80));
        println!("📊 TASK EXECUTION COUNTS:");
        println!("   Fast tasks (50-100ms):      {:>8} ({:>6.1}/s)", fast, fast as f64 / elapsed_secs);
        println!("   Medium tasks (500ms):       {:>8} ({:>6.1}/s)", medium, medium as f64 / elapsed_secs);
        println!("   Slow tasks (2s + cron):     {:>8} ({:>6.1}/s)", slow, slow as f64 / elapsed_secs);
        println!("   High-freq runnables (75ms): {:>8} ({:>6.1}/s)", high_freq, high_freq as f64 / elapsed_secs);
        println!("   CPU-intensive (250ms):      {:>8} ({:>6.1}/s)", cpu, cpu as f64 / elapsed_secs);
        println!("   Memory-intensive (1s):      {:>8} ({:>6.1}/s)", mem, mem as f64 / elapsed_secs);
        println!("{}", "-".repeat(80));
        println!("   TOTAL EXECUTIONS:           {:>8} ({:>6.1}/s)", total, rate);
        println!("{}", "=".repeat(80));
    }
    
    fn print_final_report(&self,
                          high_freq_counter: &Arc<AtomicU64>,
                          cpu_counter: &Arc<AtomicU64>,
                          mem_counter: &Arc<AtomicU64>) {
        println!("\n\n");
        println!("{}", "═".repeat(80));
        println!("🏁 FINAL STRESS TEST REPORT");
        println!("{}", "═".repeat(80));
        
        let elapsed = self.start_time.elapsed();
        let elapsed_secs = elapsed.as_secs_f64();
        
        let fast = FAST_COUNTER.load(Ordering::Relaxed);
        let medium = MEDIUM_COUNTER.load(Ordering::Relaxed);
        let slow = SLOW_COUNTER.load(Ordering::Relaxed);
        let high_freq = high_freq_counter.load(Ordering::Relaxed);
        let cpu = cpu_counter.load(Ordering::Relaxed);
        let mem = mem_counter.load(Ordering::Relaxed);
        
        let total = fast + medium + slow + high_freq + cpu + mem;
        
        println!("\n⏱️  DURATION: {:.2} seconds", elapsed_secs);
        println!("\n📊 EXECUTION SUMMARY:");
        println!("   ┌─────────────────────────────────┬──────────┬─────────────┐");
        println!("   │ Task Type                       │ Count    │ Rate (tasks/s) │");
        println!("   ├─────────────────────────────────┼──────────┼─────────────┤");
        println!("   │ Fast (50-100ms)                 │ {:>8} │ {:>11.2} │", fast, fast as f64 / elapsed_secs);
        println!("   │ Medium (500ms)                  │ {:>8} │ {:>11.2} │", medium, medium as f64 / elapsed_secs);
        println!("   │ Slow (2s + cron)                │ {:>8} │ {:>11.2} │", slow, slow as f64 / elapsed_secs);
        println!("   │ High-frequency runnables (75ms) │ {:>8} │ {:>11.2} │", high_freq, high_freq as f64 / elapsed_secs);
        println!("   │ CPU-intensive (250ms)           │ {:>8} │ {:>11.2} │", cpu, cpu as f64 / elapsed_secs);
        println!("   │ Memory-intensive (1s)           │ {:>8} │ {:>11.2} │", mem, mem as f64 / elapsed_secs);
        println!("   ├─────────────────────────────────┼──────────┼─────────────┤");
        println!("   │ TOTAL                           │ {:>8} │ {:>11.2} │", total, total as f64 / elapsed_secs);
        println!("   └─────────────────────────────────┴──────────┴─────────────┘");
        
        println!("\n✅ STRESS TEST RESULTS:");
        println!("   • Average execution rate: {:.2} tasks/second", total as f64 / elapsed_secs);
        println!("   • Peak throughput: High-frequency tasks executed {} times", high_freq);
        println!("   • CPU-intensive tasks completed: {}", cpu);
        println!("   • Memory-intensive tasks completed: {}", mem);
        
        if total > 1000 {
            println!("\n🎉 EXCELLENT: Library handled {} task executions successfully!", total);
        } else if total > 500 {
            println!("\n✅ GOOD: Library executed {} tasks under stress conditions", total);
        } else {
            println!("\n⚠️  MODERATE: Library executed {} tasks", total);
        }
        
        println!("\n{}", "═".repeat(80));
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("\n{}", "═".repeat(80));
    println!("🚀 CONCERTO LIBRARY - COMPREHENSIVE STRESS TEST");
    println!("{}", "═".repeat(80));
    println!("\n📝 Test Configuration:");
    println!("   • 2 ultra-fast scheduled functions (50-100ms)");
    println!("   • 1 medium scheduled function (500ms)");
    println!("   • 2 slow scheduled functions (2s + cron)");
    println!("   • 20 high-frequency runnable tasks (75ms)");
    println!("   • 10 CPU-intensive runnable tasks (250ms)");
    println!("   • 5 memory-intensive runnable tasks (1s)");
    println!("   • Total: 40 concurrent tasks");
    println!("\n⏱️  Duration: 60 seconds (or Ctrl+C to stop early)");
    println!("{}", "═".repeat(80));
    
    let monitor = StressTestMonitor::new();
    
    // Counters for runnable tasks
    let high_freq_counter = Arc::new(AtomicU64::new(0));
    let cpu_counter = Arc::new(AtomicU64::new(0));
    let mem_counter = Arc::new(AtomicU64::new(0));
    
    // Build scheduler with all tasks
    println!("\n🔧 Building scheduler...");
    let mut builder = SchedulerBuilder::new();
    
    // Register 20 high-frequency tasks
    for _ in 0..20 {
        builder = builder.register(HighFrequencyTask {
            counter: Arc::clone(&high_freq_counter),
        });
    }
    
    // Register 10 CPU-intensive tasks
    for _ in 0..10 {
        builder = builder.register(CpuIntensiveTask {
            counter: Arc::clone(&cpu_counter),
        });
    }
    
    // Register 5 memory-intensive tasks
    for _ in 0..5 {
        builder = builder.register(MemoryIntensiveTask {
            counter: Arc::clone(&mem_counter),
            data: (0..1000).collect(),
        });
    }
    
    let scheduler = builder.build();
    
    println!("✅ Scheduler built successfully");
    println!("🚀 Starting all tasks...\n");
    
    let _handle = scheduler.start().await?;
    
    // Monitor every 10 seconds
    let running = Arc::clone(&monitor.running);
    
    tokio::select! {
        _ = async {
            for _ in 0..6 {  // 6 iterations = 60 seconds
                tokio::time::sleep(Duration::from_secs(10)).await;
                let now = Local::now().format("%Y-%m-%d %H:%M:%S");
                println!("[{}]", now);
                monitor.print_status(&high_freq_counter, &cpu_counter, &mem_counter);
            }
        } => {},
        _ = signal::ctrl_c() => {
            println!("\n\n⚠️  Ctrl+C received, stopping stress test...");
            running.store(false, Ordering::Relaxed);
        }
    }
    
    // Final report
    monitor.print_final_report(&high_freq_counter, &cpu_counter, &mem_counter);
    
    println!("\n💾 Tip: Run with cargo run --release --example stress-test for better performance");
    println!("\n");
    
    Ok(())
}
