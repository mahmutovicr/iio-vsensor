#include <linux/module.h>
#include <linux/kernel.h>
#include <linux/init.h>
#include <linux/iio/iio.h>
#include <linux/iio/sysfs.h>
#include <linux/platform_device.h>
#include <linux/random.h>

MODULE_LICENSE("GPL");
MODULE_AUTHOR("mahmutovicr");
MODULE_DESCRIPTION("Virtual Multi-Sensor IIO Driver");
MODULE_VERSION("1.0");

static struct platform_device *viio_pdev;

static const struct iio_chan_spec viio_channels[] = {
    {
        .type = IIO_TEMP,
        .info_mask_separate = BIT(IIO_CHAN_INFO_RAW) | BIT(IIO_CHAN_INFO_SCALE),
        .indexed = 1,
        .channel = 0,
    },
    {
        .type = IIO_ANGL_VEL,
        .modified = 1,
        .channel2 = IIO_MOD_X,
        .info_mask_separate = BIT(IIO_CHAN_INFO_RAW) | BIT(IIO_CHAN_INFO_SCALE),
    },
    {
        .type = IIO_ANGL_VEL,
        .modified = 1,
        .channel2 = IIO_MOD_Y,
        .info_mask_separate = BIT(IIO_CHAN_INFO_RAW) | BIT(IIO_CHAN_INFO_SCALE),
    },
    {
        .type = IIO_ANGL_VEL,
        .modified = 1,
        .channel2 = IIO_MOD_Z,
        .info_mask_separate = BIT(IIO_CHAN_INFO_RAW) | BIT(IIO_CHAN_INFO_SCALE),
    },
    {
        .type = IIO_ACCEL,
        .modified = 1,
        .channel2 = IIO_MOD_X,
        .info_mask_separate = BIT(IIO_CHAN_INFO_RAW) | BIT(IIO_CHAN_INFO_SCALE),
    },
    {
        .type = IIO_ACCEL,
        .modified = 1,
        .channel2 = IIO_MOD_Y,
        .info_mask_separate = BIT(IIO_CHAN_INFO_RAW) | BIT(IIO_CHAN_INFO_SCALE),
    },
    {
        .type = IIO_ACCEL,
        .modified = 1,
        .channel2 = IIO_MOD_Z,
        .info_mask_separate = BIT(IIO_CHAN_INFO_RAW) | BIT(IIO_CHAN_INFO_SCALE),
    },
    {
        .type = IIO_VOLTAGE,
        .info_mask_separate = BIT(IIO_CHAN_INFO_RAW) | BIT(IIO_CHAN_INFO_SCALE),
        .indexed = 1,
        .channel = 0,
    },
};

static int viio_read_raw(struct iio_dev *indio_dev,
                         struct iio_chan_spec const *chan,
                         int *val, int *val2, long mask)
{
    u32 random;
    get_random_bytes(&random, sizeof(random));

    if (mask == IIO_CHAN_INFO_SCALE) {
        switch (chan->type) {
        case IIO_TEMP:
            *val = 10;
            *val2 = 0;
            return IIO_VAL_INT;
        case IIO_ANGL_VEL:
            *val = 0;
            *val2 = 17453;
            return IIO_VAL_INT_PLUS_MICRO;
        case IIO_ACCEL:
            *val = 0;
            *val2 = 9806;
            return IIO_VAL_INT_PLUS_MICRO;
        case IIO_VOLTAGE:
            *val = 1;
            *val2 = 0;
            return IIO_VAL_INT;
        default:
            return -EINVAL;
        }
    }

    if (mask == IIO_CHAN_INFO_RAW) {
        switch (chan->type) {
        case IIO_TEMP:
            *val = 2500 + (int)(random % 500);
            return IIO_VAL_INT;
        case IIO_ANGL_VEL:
            *val = (int)(random % 2000) - 1000;
            return IIO_VAL_INT;
        case IIO_ACCEL:
            *val = (int)(random % 2000) - 1000;
            return IIO_VAL_INT;
        case IIO_VOLTAGE:
            *val = 3300 + (int)(random % 200);
            return IIO_VAL_INT;
        default:
            return -EINVAL;
        }
    }

    return -EINVAL;
}

static const struct iio_info viio_info = {
    .read_raw = viio_read_raw,
};

static int viio_probe(struct platform_device *pdev)
{
    struct iio_dev *indio_dev;

    indio_dev = devm_iio_device_alloc(&pdev->dev, 0);
    if (!indio_dev)
        return -ENOMEM;

    indio_dev->name = "viio-sensor";
    indio_dev->info = &viio_info;
    indio_dev->modes = INDIO_DIRECT_MODE;
    indio_dev->channels = viio_channels;
    indio_dev->num_channels = ARRAY_SIZE(viio_channels);

    platform_set_drvdata(pdev, indio_dev);

    return devm_iio_device_register(&pdev->dev, indio_dev);
}

static struct platform_driver viio_driver = {
    .probe = viio_probe,
    .driver = {
        .name = "viio-sensor",
    },
};

static int __init viio_init(void)
{
    int ret;

    ret = platform_driver_register(&viio_driver);
    if (ret)
        return ret;

    viio_pdev = platform_device_register_simple("viio-sensor", -1, NULL, 0);
    if (IS_ERR(viio_pdev)) {
        platform_driver_unregister(&viio_driver);
        return PTR_ERR(viio_pdev);
    }

    return 0;
}

static void __exit viio_exit(void)
{
    platform_device_unregister(viio_pdev);
    platform_driver_unregister(&viio_driver);
}

module_init(viio_init);
module_exit(viio_exit);