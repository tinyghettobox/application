#!/bin/sh
###
# Script will be called during systemd shutdown phase.
# It will set the POWER_OFF_PIN to output and pull this pin low, which will tell on-off-shim to fully cut the power.
# Additionally if set the set led will flash 4 times and the power off pin will be set to low.
###

LED_PIN=$(sqlite3 /srv/tinyghettobox/tinyghettobox.sqlite "select led_pin from system_config")
POWER_OFF_PIN=$(sqlite3 /srv/tinyghettobox/tinyghettobox.sqlite "select power_off_pin from system_config")

if [ "$1" = "poweroff" ]; then
    if [ "${LED_PIN}" != "" && "${LED_PIN}" != "0" ]; then
        /bin/echo ${LED_PIN} > /sys/class/gpio/export
        /bin/echo out > /sys/class/gpio/gpio${LED_PIN}/direction

        for iteration in 1 2 3 4; do
            /bin/echo 0 > /sys/class/gpio/gpio${LED_PIN}/value
            /bin/sleep 0.2
            /bin/echo 1 > /sys/class/gpio/gpio${LED_PIN}/value
            /bin/sleep 0.2
        done
    fi

    /bin/echo ${POWER_OFF_PIN} > /sys/class/gpio/export
    /bin/echo out > /sys/class/gpio/gpio${POWER_OFF_PIN}/direction
    /bin/echo 0 > /sys/class/gpio/gpio${POWER_OFF_PIN}/value
fi