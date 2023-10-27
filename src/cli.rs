use clap::ArgMatches;
use serde_json::Value;
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
    #[allow(dead_code)]
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
    #[allow(dead_code)]
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
    #[allow(dead_code)]
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


pub struct JSONFileHandler {
    file_handler: FileHandler,
}

impl JSONFileHandler {
    #[allow(dead_code)]
    pub fn new(file_path: &str, next: Option<Box<dyn Handler>>) -> Self {
        JSONFileHandler { file_handler: FileHandler::new(file_path, next) }
    }

    fn find_key_recursive(json_value: &Value, key: &str) -> Option<String> {
        match json_value {
            Value::Object(map) => {
                if let Some(value) = map.get(key) {
                    match value {
                        serde_json::Value::String(value) => return Some(value.as_str().to_string()),
                        _ => return Some(value.to_string())
                        // serde_json::Value::Number(value) => return Some(value.to_string()),
                        // _ => {}
                    }
                }
                for (_, value) in map.iter() {
                    if let Some(found) = Self::find_key_recursive(value, key) {
                        return Some(found);
                    }
                }
            }
            Value::Array(arr) => {
                for value in arr.iter() {
                    if let Some(found) = Self::find_key_recursive(value, key) {
                        return Some(found);
                    }
                }
            }
            _ => {}
        }
        None
    }
}

impl Handler for JSONFileHandler {
    fn handle_request(&self, key: &str) -> Option<String> {
        if let Some(file_data) = self.file_handler.handle_request(key) {
            if let Ok(parsed_json) = serde_json::from_str::<Value>(&file_data) {
                if let Some(value) = Self::find_key_recursive(&parsed_json, key) {
                    return Some(value);
                }
            } else {
                if let Some(next_handler) = &self.file_handler.next {
                    return next_handler.handle_request(key);
                }
            }
        }
        None
    }
}


pub struct DefaultHandler {
    value: String,
}

impl DefaultHandler {
    #[allow(dead_code)]
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

    mod json_file_handler {
        use tempfile::NamedTempFile;
        use std::io::Write;

        use super::*;

        #[test]
        fn test_retrieves_set_value_number() {
            let mut temp_file = NamedTempFile::new().unwrap();
            writeln!(temp_file, r#"{{"test_key": 123}}"#).unwrap();

            let handler = JSONFileHandler::new(temp_file.path().to_str().unwrap(), None);
            let actual = handler.handle_request("test_key"); // key is not used in this handler
            assert_eq!(actual, Some("123".to_string()));
        }

        #[test]
        fn test_retrieves_set_value_string() {
            let mut temp_file = NamedTempFile::new().unwrap();
            writeln!(temp_file, r#"{{"test_key": "example"}}"#).unwrap();

            let handler = JSONFileHandler::new(temp_file.path().to_str().unwrap(), None);
            let actual = handler.handle_request("test_key"); // key is not used in this handler
            assert_eq!(actual, Some("example".to_string()));
        }

        #[test]
        fn test_retrieves_set_value_nested_object() {
            let mut temp_file = NamedTempFile::new().unwrap();
            writeln!(temp_file, r#"{{"test_obj": {{"test_key": "example"}} }}"#).unwrap();

            let handler = JSONFileHandler::new(temp_file.path().to_str().unwrap(), None);
            let actual = handler.handle_request("test_key"); // key is not used in this handler
            assert_eq!(actual, Some("example".to_string()));
        }

        #[test]
        fn test_retrieves_set_value_in_array() {
            let mut temp_file = NamedTempFile::new().unwrap();
            writeln!(temp_file, r#"[{{"test_key": "example"}}]"#).unwrap();

            let handler = JSONFileHandler::new(temp_file.path().to_str().unwrap(), None);
            let actual = handler.handle_request("test_key"); // key is not used in this handler
            assert_eq!(actual, Some("example".to_string()));
        }

        #[test]
        fn test_returns_none_for_nonexistent_file() {
            let handler = JSONFileHandler::new("", None);
            let result = handler.handle_request("example");
            assert_eq!(result, None);
        }

        #[test]
        fn test_next_handler_called() {
            let next_handler: Option<Box<dyn Handler>> = Some(Box::new(DefaultHandler::new("DEFAULT_VALUE")));
            let handler = JSONFileHandler::new("", next_handler);
            let actual = handler.handle_request("example");
            assert_eq!(actual, Some("DEFAULT_VALUE".to_string()));
        }
    }
}

