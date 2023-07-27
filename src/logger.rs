use crate::error::ErrorReport;
use flexi_logger::{
    Age, Cleanup, Criterion, Duplicate, FileSpec, Logger, LoggerHandle, Naming, WriteMode,
};

pub fn init_logger(directory_path: &String) -> Result<LoggerHandle, ErrorReport> {
    let file_specs = FileSpec::default()
        .directory(directory_path)
        .basename("log")
        .suffix("log");

    let result = Logger::try_with_str("info")?
        .log_to_file(file_specs)
        .duplicate_to_stderr(Duplicate::Warn)
        .write_mode(WriteMode::Async)
        .rotate(
            Criterion::Age(Age::Day),
            Naming::Timestamps,
            Cleanup::KeepLogFiles(7),
        )
        .start()?;
    Ok(result)
}

#[cfg(test)]
mod tests {
    use super::init_logger;
    use std::{fs, io::BufRead, thread, time};

    #[test]
    fn logging_async() {
        let dir_path = env!("TEMP_DIR_PATH");
        let _logger = init_logger(&String::from(dir_path)).unwrap();

        //log::set_logger(logger);
        log::info!("Testing info logging {}", 1);
        log::warn!("Testing warn logging {}", 2);
        log::error!("Testing error logging {}", 3);

        thread::sleep(time::Duration::from_secs(3)); //wait for the async logger to finish the job

        let file_path = fs::read_dir(&dir_path)
            .unwrap()
            .find(|entry| {
                let entry = entry.as_ref().unwrap();
                let meta = &entry.metadata().unwrap();
                meta.is_file() && entry.file_name().to_str().unwrap().ends_with("CURRENT.log")
            })
            .unwrap()
            .unwrap()
            .path();

        let text = fs::read(&file_path).unwrap();
        let line_count = text.lines().count();
        fs::remove_file(&file_path).unwrap();
        assert_eq!(line_count, 3, "Log file didn't contain all logs");
    }
}
