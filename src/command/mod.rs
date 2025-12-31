use crate::command::placeholder::PlaceholderEnum;
use redis;
use std::{collections::HashMap, sync::{Arc, Mutex}};

mod distribution;
mod parser;
mod placeholder;

fn split_args(cmd: &String) -> Vec<&str> {
    let mut args = Vec::new();
    let bytes = cmd.as_bytes();
    let mut i = 0;
    
    while i < bytes.len() {
        // Skip leading whitespace characters
        while i < bytes.len() && (bytes[i] == b' ' || bytes[i] == b'\t') {
            i += 1;
        }
        
        if i >= bytes.len() {
            break;
        }
        
        let start = i;
        
        // Check if it's the special syntax starting with #"
        if i + 1 < bytes.len() && bytes[i] == b'#' && bytes[i + 1] == b'"' {
            i += 2; // Skip #"
            let content_start = i;
            
            // Find the ending "#
            while i + 1 < bytes.len() {
                if bytes[i] == b'"' && bytes[i + 1] == b'#' {
                    args.push(&cmd[content_start..i]);
                    i += 2; // Skip "#
                    break;
                }
                i += 1;
            }
        } else {
            // Regular argument: until encountering whitespace
            while i < bytes.len() && bytes[i] != b' ' && bytes[i] != b'\t' {
                i += 1;
            }
            args.push(&cmd[start..i]);
        }
    }
    
    args
}

#[derive(Clone, Debug)]
pub struct Command {
    str: String,
    argv: Vec<PlaceholderEnum>,
    #[allow(dead_code)]
    lock: Arc<Mutex<()>>,
}

impl Command {
    pub fn new(cmd: &str) -> Command {
        let prev_cmd = cmd;
        match parser::parse_all(cmd) {
            Ok((nm, args)) => {
                assert_eq!(nm, "");
                Command {
                    str: prev_cmd.to_string(),
                    argv: args,
                    lock: Arc::new(Mutex::new(())),
                }
            }
            Err(e) => {
                panic!("cmd parse error. cmd: {}, error: {:?}", cmd, e);
            }
        }
    }
    pub fn gen_cmd(&mut self) -> redis::Cmd {
        let mut cmd = redis::Cmd::new();
        let mut cmd_str = String::new();
        let mut arg_map: HashMap<String, Vec<String>> = HashMap::new();
        for ph in self.argv.iter_mut() {
            let args = ph.generate(&arg_map);
            if let Some(name) = args.name {
                arg_map.insert(name, args.value.clone());
            }
            for arg in args.value {
                cmd_str.push_str(&arg);
            }
        }
        for word in split_args(&cmd_str) {
            cmd.arg(word);
        }
        cmd
    }
    #[allow(dead_code)]
    pub fn gen_cmd_with_lock(&mut self) -> redis::Cmd {
        let _lock = self.lock.lock().unwrap();
        let mut cmd = redis::Cmd::new();
        let mut cmd_str = String::new();
        let mut arg_map = HashMap::new();
        for ph in self.argv.iter_mut() {
            let args = ph.generate(&arg_map);
            if let Some(name) = args.name {
                eprintln!("{} {:?}", name, args.value);
                arg_map.insert(name, args.value.clone());
            }
            for arg in args.value {
                cmd_str.push_str(&arg);
            }
        }
        for word in split_args(&cmd_str) {
            cmd.arg(word);
        }
        cmd
    }
    pub fn to_string(&self) -> String {
        self.str.clone()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_split_args_normal() {
        let cmd = "SET key value".to_string();
        let args = split_args(&cmd);
        assert_eq!(args, vec!["SET", "key", "value"]);
    }

    #[test]
    fn test_split_args_with_quoted() {
        let cmd = r##"SET #"key with spaces"# value"##.to_string();
        let args = split_args(&cmd);
        assert_eq!(args, vec!["SET", "key with spaces", "value"]);
    }

    #[test]
    fn test_split_args_multiple_quoted() {
        let cmd = r##"SET #"key with spaces"# #"value with spaces"#"##.to_string();
        let args = split_args(&cmd);
        assert_eq!(args, vec!["SET", "key with spaces", "value with spaces"]);
    }

    #[test]
    fn test_split_args_mixed_args() {
        let cmd = r##"SET key1 #"value with spaces"# key2 normal_value"##.to_string();
        let args = split_args(&cmd);
        assert_eq!(args, vec!["SET", "key1", "value with spaces", "key2", "normal_value"]);
    }

    #[test]
    fn test_split_args_empty_quoted_part() {
        let cmd = r##"CMD #""# normal"##.to_string();
        let args = split_args(&cmd);
        assert_eq!(args, vec!["CMD", "", "normal"]);
    }

    #[test]
    fn test_split_args_empty_quoted_part_with_quotes() {
        let cmd = r##"CMD key #"{"abc": "def"}"#"##.to_string();
        let args = split_args(&cmd);
        assert_eq!(args, vec!["CMD", "key", "{\"abc\": \"def\"}"]);
    }

    #[test]
    fn test_split_args_no_quotes() {
        let cmd = "GET key1 key2 key3".to_string();
        let args = split_args(&cmd);
        assert_eq!(args, vec!["GET", "key1", "key2", "key3"]);
    }
}