use sysinfo::System;
use std::thread;
use std::process::Command;

pub fn run() -> i32 {
    // 1. Try running fastfetch
    let status_fast = Command::new("fastfetch").status();
    if status_fast.is_ok() && status_fast.unwrap().success() {
        return 0;
    }

    // 2. Try running neofetch as a secondary fallback
    let status_neo = Command::new("neofetch").status();
    if status_neo.is_ok() && status_neo.unwrap().success() {
        return 0;
    }

    // 3. Fallback: Print our beautiful progress-bar system dashboard
    let mut sys = System::new_all();
    sys.refresh_cpu_usage();
    sys.refresh_memory();
    thread::sleep(std::time::Duration::from_millis(100));
    sys.refresh_cpu_usage();

    let os_name = System::name().unwrap_or_else(|| "Unknown".to_string());
    let kernel_version = System::kernel_version().unwrap_or_else(|| "Unknown".to_string());
    let host_name = System::host_name().unwrap_or_else(|| "Unknown".to_string());
    let uptime_sec = System::uptime();
    
    let cpus = sys.cpus();
    let cpu_brand = if let Some(cpu) = cpus.first() {
        cpu.brand().trim().to_string()
    } else {
        "Unknown".to_string()
    };
    
    let cpu_load = if cpus.is_empty() {
        0.0
    } else {
        cpus.iter().map(|cpu| cpu.cpu_usage()).sum::<f32>() / cpus.len() as f32
    };

    let total_mem = sys.total_memory() as f64 / (1024.0 * 1024.0 * 1024.0);
    let used_mem = sys.used_memory() as f64 / (1024.0 * 1024.0 * 1024.0);
    let mem_percentage = if sys.total_memory() > 0 {
        (sys.used_memory() as f32 / sys.total_memory() as f32) * 100.0
    } else {
        0.0
    };

    let uptime_str = format_uptime(uptime_sec);

    let bar_width = 20;
    let cpu_bar = get_progress_bar(cpu_load, bar_width);
    let mem_bar = get_progress_bar(mem_percentage, bar_width);

    println!("  \x1b[1;36mHostname:\x1b[0m    {}", host_name);
    println!("  \x1b[1;36mOS:\x1b[0m          {}", os_name);
    println!("  \x1b[1;36mKernel:\x1b[0m      {}", kernel_version);
    println!("  \x1b[1;36mUptime:\x1b[0m      {}", uptime_str);
    println!("  \x1b[1;36mCPU model:\x1b[0m   {}", cpu_brand);
    println!("  \x1b[1;36mCPU cores:\x1b[0m   {}", cpus.len());
    println!(
        "  \x1b[1;36mCPU load:\x1b[0m    \x1b[1;32m[{}]\x1b[0m  {:.1}%",
        cpu_bar, cpu_load
    );
    println!(
        "  \x1b[1;36mMemory:\x1b[0m      \x1b[1;34m[{}]\x1b[0m  {:.1}%  ({:.2} GB / {:.2} GB)",
        mem_bar, mem_percentage, used_mem, total_mem
    );
    println!();

    0
}

fn get_progress_bar(percentage: f32, width: usize) -> String {
    let clamped = percentage.clamp(0.0, 100.0);
    let filled_length = ((clamped / 100.0) * width as f32).round() as usize;
    let mut bar = String::new();
    for _ in 0..filled_length {
        bar.push('█');
    }
    for _ in filled_length..width {
        bar.push('░');
    }
    bar
}

fn format_uptime(secs: u64) -> String {
    let days = secs / 86400;
    let hours = (secs % 86400) / 3600;
    let minutes = (secs % 3600) / 60;
    
    let mut parts = Vec::new();
    if days > 0 {
        parts.push(format!("{} day{}", days, if days == 1 { "" } else { "s" }));
    }
    if hours > 0 {
        parts.push(format!("{} hour{}", hours, if hours == 1 { "" } else { "s" }));
    }
    if minutes > 0 || (days == 0 && hours == 0) {
        parts.push(format!("{} minute{}", minutes, if minutes == 1 { "" } else { "s" }));
    }
    
    parts.join(", ")
}
