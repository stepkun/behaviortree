use ariel_os::hal::peripherals;

#[cfg(context = "rp")]
ariel_os::hal::define_peripherals!(MotorPeripherals {
	motor_down: PIN_14,
	motor_up: PIN_15,
});

#[cfg(context = "rp")]
ariel_os::hal::define_peripherals!(UpperEndPeripherals {
	led_upper_end: PIN_13,
	btn_upper_end: PIN_16,
});

#[cfg(context = "rp")]
ariel_os::hal::define_peripherals!(LowerEndPeripherals {
	led_lower_end: PIN_12,
	btn_lower_end: PIN_17,
});

#[cfg(context = "rp")]
ariel_os::hal::define_peripherals!(PanelPeripherals {
	led_down: PIN_10,
	led_up: PIN_11,
	btn_up: PIN_18,
	btn_stop: PIN_19,
	btn_down: PIN_20,
});

#[cfg(context = "rp")]
ariel_os::hal::define_peripherals!(AddonPeripherals { io_preparation: PIN_9 });

#[cfg(context = "rp")]
ariel_os::hal::define_peripherals!(EmergencyPeripherals { btn_emergency: PIN_21 });
