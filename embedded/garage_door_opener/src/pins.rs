use ariel_os::hal::peripherals;

#[cfg(context = "rp")]
ariel_os::hal::define_peripherals!(Peripherals {
	led_down: PIN_10,
	led_up: PIN_11,
	led_lower_end: PIN_12,
	led_upper_end: PIN_13,
	led_motor_down: PIN_14,
	led_motor_up: PIN_15,
	btn_upper_end: PIN_16,
	btn_lower_end: PIN_17,
	btn_up: PIN_18,
	btn_stop: PIN_19,
	btn_down: PIN_20,
	btn_emergency: PIN_21,
	io_preparation: PIN_22,
});
