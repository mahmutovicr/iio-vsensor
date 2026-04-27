obj-m += viio_sensor.o

KDIR := /home/r_m/WSL2-Kernel-6.6
PWD := $(shell pwd)

all:
	make -C $(KDIR) M=$(PWD) KCONFIG_CONFIG=$(KDIR)/Microsoft/config-wsl modules

clean:
	make -C $(KDIR) M=$(PWD) KCONFIG_CONFIG=$(KDIR)/Microsoft/config-wsl clean