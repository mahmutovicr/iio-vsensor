savedcmd_/home/r_m/iio-vsensor/driver/viio_sensor.mod := printf '%s\n'   viio_sensor.o | awk '!x[$$0]++ { print("/home/r_m/iio-vsensor/driver/"$$0) }' > /home/r_m/iio-vsensor/driver/viio_sensor.mod
