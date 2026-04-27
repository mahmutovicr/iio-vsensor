<div align="center">

# IIO Virtual Sensor Driver

</div>

## About

Linux IIO kernel driver for virtual temperature, gyroscope, accelerometer and voltage sensors using Rust CLI for monitoring

## Features

- **Temperature** — thermal sensor data via Linux IIO sysfs interface
- **Gyroscope** — 3-axis angular velocity sensor (X, Y, Z) in rad/s
- **Accelerometer** — 3-axis acceleration sensor (X, Y, Z) in m/s²
- **Voltage** — power rail voltage monitoring in V

## Build

```bash
cd driver && make
sudo modprobe industrialio
sudo insmod viio_sensor.ko
cd ../cli && cargo build --release
```

## Usage

```bash
sensor-ctl status
sensor-ctl read
sensor-ctl read --temp
sensor-ctl read --gyro
sensor-ctl read --accel
sensor-ctl read --voltage
sensor-ctl monitor
sensor-ctl monitor --interval 500
```

## Unload

```bash
sudo rmmod viio_sensor
```

## Tech Stack

![C](https://img.shields.io/badge/C-A8B9CC?style=for-the-badge&logo=c&logoColor=black)
![Rust](https://img.shields.io/badge/Rust-000000?style=for-the-badge&logo=rust&logoColor=white)
![Linux](https://img.shields.io/badge/Linux%20Kernel-IIO-FCC624?style=for-the-badge&logo=linux&logoColor=black)

## License

MIT License
