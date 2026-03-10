use freven_guest_sdk::{log_debug, log_error, log_info, log_warn};

#[test]
fn log_macros_expand_without_panic() {
    // no runtime bridge installed — calls are fire-and-forget no-ops here
    log_debug!("debug message");
    log_info!("info message");
    log_warn!("warn message");
    log_error!("error message");
}

#[test]
fn log_macros_accept_format_args() {
    let x = 42u32;
    log_info!("value is {}", x);
    log_warn!("coords {:?}", (1, 2, 3));
}

#[test]
fn log_macros_accept_trailing_comma() {
    log_debug!("trailing comma",);
    log_info!("also fine {}", 1,);
}
