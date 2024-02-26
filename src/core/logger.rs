use log::{
    Level,
    Record,
    Metadata,
    LevelFilter
};

static MY_LOGGER: MyLogger = MyLogger;

struct MyLogger;

impl log::Log for MyLogger {
    fn enabled(&self, metadata: &Metadata) -> bool {
        metadata.level() <= Level::Info
    }

    fn log(&self, record: &Record) {
        if self.enabled(record.metadata()) {
            println!("{}[{}]-{}", record.target(), record.level(), record.args());
        }
    }
    fn flush(&self) {
        unimplemented!()
    }
}

pub(crate) fn setup_logger() {
    _ = log::set_logger(&MY_LOGGER);
    log::set_max_level(LevelFilter::Info);
}
