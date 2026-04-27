use clap::{Parser, Subcommand};
use std::fs;
use std::path::{Path, PathBuf};
use std::thread;
use std::time::Duration;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

const IIO_DEVICES_PATH: &str = "/sys/bus/iio/devices";
const DRIVER_NAME: &str = "viio-sensor";

#[derive(Parser)]
#[command(name = "sensor-ctl")]
#[command(about = "Virtual IIO Multi-Sensor CLI — iio-vsensor driver")]
struct Cli {
    #[arg(long)]
    device: Option<String>,

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

fn find_iio_device() -> Option<PathBuf> {
    let entries = fs::read_dir(IIO_DEVICES_PATH).ok()?;
    for entry in entries.flatten() {
        let path = entry.path();
        if let Ok(name) = fs::read_to_string(path.join("name")) {
            if name.trim() == DRIVER_NAME {
                return Some(path);
            }
        }
    }
    None
}

fn resolve_base(device_override: Option<String>) -> Option<PathBuf> {
    match device_override {
        Some(path) => Some(PathBuf::from(path)),
        None => find_iio_device(),
    }
}

fn read_sysfs(base: &Path, file: &str) -> Option<i64> {
    fs::read_to_string(base.join(file)).ok()?.trim().parse::<i64>().ok()
}

fn read_temp(base: &Path) -> Option<f64> {
    let raw = read_sysfs(base, "in_temp0_raw")? as f64;
    let scale = fs::read_to_string(base.join("in_temp0_scale"))
        .ok()?.trim().parse::<f64>().ok()?;
    Some(raw * scale / 1000.0)
}

fn read_gyro(base: &Path) -> Option<(f64, f64, f64)> {
    let scale: f64 = fs::read_to_string(base.join("in_anglvel_x_scale"))
        .ok()?.trim().parse().ok()?;
    let x = read_sysfs(base, "in_anglvel_x_raw")? as f64 * scale;
    let y = read_sysfs(base, "in_anglvel_y_raw")? as f64 * scale;
    let z = read_sysfs(base, "in_anglvel_z_raw")? as f64 * scale;
    Some((x, y, z))
}

fn read_accel(base: &Path) -> Option<(f64, f64, f64)> {
    let scale: f64 = fs::read_to_string(base.join("in_accel_x_scale"))
        .ok()?.trim().parse().ok()?;
    let x = read_sysfs(base, "in_accel_x_raw")? as f64 * scale;
    let y = read_sysfs(base, "in_accel_y_raw")? as f64 * scale;
    let z = read_sysfs(base, "in_accel_z_raw")? as f64 * scale;
    Some((x, y, z))
}

fn read_voltage(base: &Path) -> Option<f64> {
    let raw = read_sysfs(base, "in_voltage0_raw")? as f64;
    let scale = fs::read_to_string(base.join("in_voltage0_scale"))
        .ok()?.trim().parse::<f64>().ok()?;
    Some(raw * scale / 1000.0)
}

fn print_all(base: &Path) {
    match read_temp(base) {
        Some(t) => println!("  Temperature  : {:.2} °C", t),
        None    => println!("  Temperature  : N/A"),
    }
    match read_gyro(base) {
        Some((x, y, z)) => println!("  Gyroscope    : X={:.4}  Y={:.4}  Z={:.4} rad/s", x, y, z),
        None             => println!("  Gyroscope    : N/A"),
    }
    match read_accel(base) {
        Some((x, y, z)) => println!("  Accelerometer: X={:.4}  Y={:.4}  Z={:.4} m/s²", x, y, z),
        None             => println!("  Accelerometer: N/A"),
    }
    match read_voltage(base) {
        Some(v) => println!("  Voltage      : {:.3} V", v),
        None    => println!("  Voltage      : N/A"),
    }
}

fn main() {
    let cli = Cli::parse();

    match cli.command {
        Commands::Status => {
            match resolve_base(cli.device) {
                Some(base) => {
                    println!("[OK] iio-vsensor driver is active.");
                    println!("     Path: {}", base.display());
                }
                None => {
                    eprintln!("[ERROR] Driver not found. Run: sudo insmod viio_sensor.ko");
                }
            }
        }

        Commands::Read { temp, gyro, accel, voltage } => {
            let base = match resolve_base(cli.device) {
                Some(b) => b,
                None => {
                    eprintln!("[ERROR] Driver not found.");
                    return;
                }
            };
            let all = !temp && !gyro && !accel && !voltage;
            if temp || all {
                match read_temp(&base) {
                    Some(t) => println!("Temperature  : {:.2} °C", t),
                    None    => println!("Temperature  : read error"),
                }
            }
            if gyro || all {
                match read_gyro(&base) {
                    Some((x, y, z)) => println!("Gyroscope    : X={:.4}  Y={:.4}  Z={:.4} rad/s", x, y, z),
                    None             => println!("Gyroscope    : read error"),
                }
            }
            if accel || all {
                match read_accel(&base) {
                    Some((x, y, z)) => println!("Accelerometer: X={:.4}  Y={:.4}  Z={:.4} m/s²", x, y, z),
                    None             => println!("Accelerometer: read error"),
                }
            }
            if voltage || all {
                match read_voltage(&base) {
                    Some(v) => println!("Voltage      : {:.3} V", v),
                    None    => println!("Voltage      : read error"),
                }
            }
        }

        Commands::Monitor { interval } => {
            let base = match resolve_base(cli.device) {
                Some(b) => b,
                None => {
                    eprintln!("[ERROR] Driver not found.");
                    return;
                }
            };
            let running = Arc::new(AtomicBool::new(true));
            let r = running.clone();
            ctrlc::set_handler(move || {
                r.store(false, Ordering::SeqCst);
            }).expect("Cannot set CTRL+C handler");

            println!("Monitoring sensors — CTRL+C to stop\n");
            while running.load(Ordering::SeqCst) {
                println!("--- iio-vsensor ---");
                print_all(&base);
                println!();
                thread::sleep(Duration::from_millis(interval));
            }
            println!("Monitoring stopped.");
        }
    }
}
