use clap::{Parser, Subcommand};
use std::fs;
use std::path::Path;
use std::thread;
use std::time::Duration;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

const IIO_BASE: &str = "/sys/bus/iio/devices/iio:device0";

#[derive(Parser)]
#[command(name = "sensor-ctl")]
#[command(about = "Virtual IIO Multi-Sensor CLI — iio-vsensor driver")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    Monitor {
        #[arg(long, default_value_t = 1000)]
        interval: u64,
    },
    Read {
        #[arg(long)]
        temp: bool,
        #[arg(long)]
        gyro: bool,
        #[arg(long)]
        accel: bool,
        #[arg(long)]
        voltage: bool,
    },
    Status,
}

fn read_sysfs(file: &str) -> Option<i64> {
    let path = format!("{}/{}", IIO_BASE, file);
    fs::read_to_string(&path).ok()?.trim().parse::<i64>().ok()
}

fn read_temp() -> Option<f64> {
    let raw = read_sysfs("in_temp0_raw")? as f64;
    let scale = fs::read_to_string(format!("{}/in_temp0_scale", IIO_BASE))
        .ok()?.trim().parse::<f64>().ok()?;
    Some(raw * scale / 1000.0)
}

fn read_gyro() -> Option<(f64, f64, f64)> {
    let scale_str = fs::read_to_string(format!("{}/in_anglvel_x_scale", IIO_BASE)).ok()?;
    let scale: f64 = scale_str.trim().parse().ok()?;
    let x = read_sysfs("in_anglvel_x_raw")? as f64 * scale;
    let y = read_sysfs("in_anglvel_y_raw")? as f64 * scale;
    let z = read_sysfs("in_anglvel_z_raw")? as f64 * scale;
    Some((x, y, z))
}

fn read_accel() -> Option<(f64, f64, f64)> {
    let scale_str = fs::read_to_string(format!("{}/in_accel_x_scale", IIO_BASE)).ok()?;
    let scale: f64 = scale_str.trim().parse().ok()?;
    let x = read_sysfs("in_accel_x_raw")? as f64 * scale;
    let y = read_sysfs("in_accel_y_raw")? as f64 * scale;
    let z = read_sysfs("in_accel_z_raw")? as f64 * scale;
    Some((x, y, z))
}

fn read_voltage() -> Option<f64> {
    let raw = read_sysfs("in_voltage0_raw")? as f64;
    let scale = fs::read_to_string(format!("{}/in_voltage0_scale", IIO_BASE))
        .ok()?.trim().parse::<f64>().ok()?;
    Some(raw * scale / 1000.0)
}

fn driver_present() -> bool {
    Path::new(IIO_BASE).exists()
}

fn print_all() {
    match read_temp() {
        Some(t) => println!("  Temperature  : {:.2} °C", t),
        None => println!("  Temperature  : N/A"),
    }
    match read_gyro() {
        Some((x, y, z)) => println!("  Gyroscope    : X={:.4}  Y={:.4}  Z={:.4} rad/s", x, y, z),
        None => println!("  Gyroscope    : N/A"),
    }
    match read_accel() {
        Some((x, y, z)) => println!("  Accelerometer: X={:.4}  Y={:.4}  Z={:.4} m/s²", x, y, z),
        None => println!("  Accelerometer: N/A"),
    }
    match read_voltage() {
        Some(v) => println!("  Voltage      : {:.3} V", v),
        None => println!("  Voltage      : N/A"),
    }
}

fn main() {
    let cli = Cli::parse();
    match cli.command {
        Commands::Status => {
            if driver_present() {
                println!("[OK] iio-vsensor driver is active.");
                println!("     Path: {}", IIO_BASE);
            } else {
                eprintln!("[ERROR] Driver not loaded. Run: sudo insmod viio_sensor.ko");
            }
        }
        Commands::Read { temp, gyro, accel, voltage } => {
            if !driver_present() {
                eprintln!("[ERROR] Driver not loaded.");
                return;
            }
            let all = !temp && !gyro && !accel && !voltage;
            if temp || all {
                match read_temp() {
                    Some(t) => println!("Temperature  : {:.2} °C", t),
                    None => println!("Temperature  : read error"),
                }
            }
            if gyro || all {
                match read_gyro() {
                    Some((x, y, z)) => println!("Gyroscope    : X={:.4}  Y={:.4}  Z={:.4} rad/s", x, y, z),
                    None => println!("Gyroscope    : read error"),
                }
            }
            if accel || all {
                match read_accel() {
                    Some((x, y, z)) => println!("Accelerometer: X={:.4}  Y={:.4}  Z={:.4} m/s²", x, y, z),
                    None => println!("Accelerometer: read error"),
                }
            }
            if voltage || all {
                match read_voltage() {
                    Some(v) => println!("Voltage      : {:.3} V", v),
                    None => println!("Voltage      : read error"),
                }
            }
        }
        Commands::Monitor { interval } => {
            if !driver_present() {
                eprintln!("[ERROR] Driver not loaded.");
                return;
            }
            let running = Arc::new(AtomicBool::new(true));
            let r = running.clone();
            ctrlc::set_handler(move || {
                r.store(false, Ordering::SeqCst);
            }).expect("Cannot set CTRL+C handler");
            println!("Monitoring sensors — CTRL+C to stop\n");
            while running.load(Ordering::SeqCst) {
                println!("--- iio-vsensor ---");
                print_all();
                println!();
                thread::sleep(Duration::from_millis(interval));
            }
            println!("Monitoring stopped.");
        }
    }
}