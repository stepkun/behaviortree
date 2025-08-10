#![no_main]
#![no_std]

//! Embedded version of [t01_buid_your_first_tree](examples/t01_build_your_first_tree.rs)

use ariel_os::debug::{ExitCode, exit, log::*};

#[ariel_os::task(autostart)]
async fn main() {
        info!("Hello world!\n");

        exit(ExitCode::SUCCESS);
}
