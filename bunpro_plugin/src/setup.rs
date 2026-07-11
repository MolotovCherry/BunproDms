pub fn setup() {
    crate::panic::set_hook();
    crate::log::QtLogger::init();
}
