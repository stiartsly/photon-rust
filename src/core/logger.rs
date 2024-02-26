use log::{Level, Record, Metadata};

pub(crate) struct MyLogger;

impl log::Log for MyLogger {
    fn enabled(&self, metadata: &Metadata) -> bool {
        metadata.level() <= Level::Info
    }

    fn log(&self, record: &Record) {
        if self.enabled(record.metadata()) {
            println!("{} {}- {}", record.target(), record.level(), record.args());
        }
    }
    fn flush(&self) {
        unimplemented!()
    }
}
