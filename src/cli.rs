use clap::ArgMatches;
use std::env;
use std::fs::File;
use std::io::Read;
use std::path::Path;
use std::path::PathBuf;

pub trait Handler {
    fn handle_request(&self, key: &str) -> Option<String>;
}

pub struct ArgHandler<'a> {
    args: &'a ArgMatches,
    next: Option<Box<dyn Handler>>,
}

impl<'a> ArgHandler<'a> {
    pub fn new(args: &'a ArgMatches, next: Option<Box<dyn Handler>>) -> Self {
        ArgHandler { args, next }
    }
}

impl<'a> Handler for ArgHandler<'a> {
    fn handle_request(&self, key: &str) -> Option<String> {
        if let Some(value) = self.args.get_one::<String>(key).map(String::from) {
            return Some(value)
        }
        if let Some(next_handler) = &self.next {
            return next_handler.handle_request(key);
        }
        None
    }
}

pub struct EnvHandler {
    next: Option<Box<dyn Handler>>,
}

impl EnvHandler {
    pub fn new(next: Option<Box<dyn Handler>>) -> Self {
        EnvHandler { next }
    }
}

impl Handler for EnvHandler {
    fn handle_request(&self, key: &str) -> Option<String> {
        if let Ok(value) = env::var(key) {
            return Some(value);
        }
        if let Some(next_handler) = &self.next {
            return next_handler.handle_request(key);
        }
        None
    }
}


pub struct FileHandler {
    file_path: PathBuf,
    next: Option<Box<dyn Handler>>,
}

impl FileHandler {
    pub fn new(file_path: &str, next: Option<Box<dyn Handler>>) -> Self {
        FileHandler { file_path: Path::new(file_path).into(), next }
    }
}

impl Handler for FileHandler {
    fn handle_request(&self, key: &str) -> Option<String> {
        if let Ok(mut file) = File::open(&self.file_path) {
            let mut content = String::new();
            if let Ok(_byte_count) = file.read_to_string(&mut content) {
                return Some(content);
            }
        }
        if let Some(next_handler) = &self.next {
            return next_handler.handle_request(key);
        }
        None
    }
}

pub struct DefaultHandler {
    value: String,
}

impl DefaultHandler {
    pub fn new(value: &str) -> Self {
        DefaultHandler { value: String::from(value) }
    }
}

impl Handler for DefaultHandler {
    fn handle_request(&self, _: &str) -> Option<String> {
        Some(self.value.clone())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    mod default_handler {
        use super::*;

        #[test]
        fn test_retrieves_set_value() {
            let handler = DefaultHandler::new("TEST_VAL");
            let actual = handler.handle_request("");
            assert_eq!(actual, Some("TEST_VAL".to_string()));
        }
    }

    mod env_handler {
        use super::*;

        #[test]
        fn test_retrieves_set_value() {
            env::set_var("TEST_KEY", "test_value");
            let handler = EnvHandler::new(None);
            let actual = handler.handle_request("TEST_KEY");
            assert_eq!(actual, Some("test_value".to_string()));
        }

        #[test]
        fn test_returns_none_for_unset_value() {
            env::remove_var("UNSET_KEY"); // Ensure the variable is not set
            let handler = EnvHandler::new(None);
            let actual = handler.handle_request("UNSET_KEY");
            assert_eq!(actual, None);
        }

        #[test]
        fn test_next_handler_called() {
            env::remove_var("UNSET_KEY"); // Ensure the variable is not set
            let next_handler: Option<Box<dyn Handler>> = Some(Box::new(DefaultHandler::new("DEFAULT_VALUE")));
            let handler = EnvHandler::new(next_handler);
            let actual = handler.handle_request("UNSET_KEY");
            assert_eq!(actual, Some("DEFAULT_VALUE".to_string()));
        }
    }

    mod arg_handler {
        use clap::Arg;

        use super::*;

        #[test]
        fn test_retrieves_set_value() {
            let args = clap::Command::new("test_app")
                .arg(Arg::new("example").long("example"))
                .get_matches_from(vec!["test_app", "--example", "test_value"]);

            let handler = ArgHandler::new(&args, None);
            let result = handler.handle_request("example");
            assert_eq!(result, Some("test_value".to_string()));
        }

        #[test]
        fn test_returns_none_for_unset_value() {
            let args = clap::Command::new("test_app")
                .arg(Arg::new("example").long("example"))
                .get_matches_from(vec!["test_app"]);

            let handler = ArgHandler::new(&args, None);
            let result = handler.handle_request("example");
            assert_eq!(result, None);
        }

        #[test]
        fn test_next_handler_called() {
            let args = clap::Command::new("test_app")
                .arg(Arg::new("example").long("example"))
                .get_matches_from(vec!["test_app"]);
            let next_handler: Option<Box<dyn Handler>> = Some(Box::new(DefaultHandler::new("DEFAULT_VALUE")));
            let handler = ArgHandler::new(&args, next_handler);
            let actual = handler.handle_request("example");
            assert_eq!(actual, Some("DEFAULT_VALUE".to_string()));
        }
    }

    mod file_handler {
        use tempfile::NamedTempFile;
        use std::io::Write;

        use super::*;

        #[test]
        fn test_retrieves_set_value() {
            let mut temp_file = NamedTempFile::new().unwrap();
            writeln!(temp_file, "test_content").unwrap();

            let handler = FileHandler::new(temp_file.path().to_str().unwrap(), None);
            let result = handler.handle_request(""); // key is not used in this handler
            assert_eq!(result, Some("test_content\n".to_string()));
        }

        #[test]
        fn test_returns_none_for_nonexistent_file() {
            let handler = FileHandler::new("", None);
            let result = handler.handle_request("example");
            assert_eq!(result, None);
        }

        #[test]
        fn test_next_handler_called() {
            let next_handler: Option<Box<dyn Handler>> = Some(Box::new(DefaultHandler::new("DEFAULT_VALUE")));
            let handler = FileHandler::new("", next_handler);
            let actual = handler.handle_request("example");
            assert_eq!(actual, Some("DEFAULT_VALUE".to_string()));
        }
    }
}

