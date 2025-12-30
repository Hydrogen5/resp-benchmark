use crate::command::distribution::DistributionEnum;
use rand::distributions::Alphanumeric;
use rand::prelude::*;
use std::cmp::min;
use std::collections::HashMap;
use std::process::exit;
use std::str::FromStr;

#[derive(Debug, Clone)]
pub struct PlaceholderResult {
    pub name: Option<String>,
    pub value: Vec<String>,
}

impl PlaceholderResult {
    pub fn new(name: Option<String>, value: String) -> Self {
        Self { name, value: vec![value] }
    }
    pub fn new_multi(name: Option<String>, value: Vec<String>) -> Self {
        Self { name, value }
    }
}

#[derive(Debug, Clone)]
pub enum PlaceholderEnum {
    String(PlaceholderString),
    Key(PlaceholderKey),
    Value(PlaceholderValue),
    Rand(PlaceholderRand),
    Range(PlaceholderRange),
    Reference(PlaceholderReference),
}

impl PlaceholderEnum {
    pub fn new_string(str: &str) -> Self {
        Self::String(PlaceholderString::new(str.to_string()))
    }
    pub fn new(str: &str) -> Self {
        let s = str.to_string();
        let words: Vec<&str> = s.split_whitespace().collect();
        if words.len() == 0 {
            eprint!("placeholder is empty");
            exit(1);
        }
        let ph = match words[0] {
            "key" => {
                if words.len() != 3 && words.len() != 5 {
                    eprint!("wrong number of arguments for key placeholder: {:?}", words);
                    exit(1);
                }
                let range = u64::from_str(words[2]).unwrap();
                let distribution = DistributionEnum::new(words[1], range);
                let alias = if words.len() == 5 && words[3] == "alias" { Some(words[4].to_string()) } else { None };
                PlaceholderEnum::Key(PlaceholderKey::new(distribution, alias))
            }
            "value" => {
                if words.len() != 2 && words.len() != 4 {
                    eprint!("wrong number of arguments for value placeholder: {:?}", words);
                    exit(1);
                }
                let size = u64::from_str(words[1]).unwrap();
                let alias = if words.len() == 4 && words[2] == "alias" { Some(words[3].to_string()) } else { None };
                PlaceholderEnum::Value(PlaceholderValue::new(size, alias))
            }
            "rand" => {
                if words.len() != 2 && words.len() != 4 {
                    eprint!("wrong number of arguments for rand placeholder: {:?}", words);
                    exit(1);
                }
                let alias = if words.len() == 4 && words[2] == "alias" { Some(words[3].to_string()) } else { None };
                PlaceholderEnum::Rand(PlaceholderRand::new(u64::from_str(words[1]).unwrap(), alias))
            }
            "range" => {
                if words.len() != 3 && words.len() != 5 {
                    eprint!("wrong number of arguments for range placeholder: {:?}", words);
                    exit(1);
                }
                let range = u64::from_str(words[1]).unwrap();
                let width = u64::from_str(words[2]).unwrap();
                let alias = if words.len() == 5 && words[3] == "alias" { Some(words[4].to_string()) } else { None };
                PlaceholderEnum::Range(PlaceholderRange::new(range, width, alias))
            }
            "reference" => {
                if words.len() != 2 {
                    eprint!("wrong number of arguments for reference placeholder: {:?}", words);
                    exit(1);
                }
                PlaceholderEnum::Reference(PlaceholderReference::new(words[1].to_string()))
            }
            name => {
                eprint!("Invalid placeholder: {}", name);
                exit(1);
            }
        };
        ph
    }
    pub fn generate(&mut self, aliases: &HashMap<String, Vec<String>>) -> PlaceholderResult {
        match self {
            Self::String(p) => p.generate(),
            Self::Key(p) => p.generate(),
            Self::Value(p) => p.generate(),
            Self::Rand(p) => p.generate(),
            Self::Range(p) => p.generate(),
            Self::Reference(p) => p.generate(aliases),
        }
    }
}

#[derive(Clone, Debug)]
pub struct PlaceholderString {
    value: String,
}

impl PlaceholderString {
    pub fn new(value: String) -> Self {
        Self { value }
    }
    fn generate(&mut self) -> PlaceholderResult {
        PlaceholderResult::new(None, self.value.clone())
    }
}

#[derive(Clone, Debug)]
pub struct PlaceholderKey {
    distribution: DistributionEnum,
    alias: Option<String>,
}

impl PlaceholderKey {
    fn new(distribution: DistributionEnum, alias: Option<String>) -> Self {
        Self { distribution, alias }
    }
    fn generate(&mut self) -> PlaceholderResult {
        PlaceholderResult::new(self.alias.clone(), format!("key_{:010}", self.distribution.sample(&mut rand::thread_rng())))
    }
}

#[derive(Clone, Debug)]
pub struct PlaceholderValue {
    size: usize,
    alias: Option<String>,
}

impl PlaceholderValue {
    pub fn new(size: u64, alias: Option<String>) -> Self {
        Self { size: size as usize, alias }
    }
    pub fn generate(&self) -> PlaceholderResult {
        let rng = rand::thread_rng();
        let chars: String = rng.sample_iter(Alphanumeric).take(self.size).map(char::from).collect();
        PlaceholderResult::new(self.alias.clone(), chars)
    }
}

#[derive(Clone, Debug)]
pub struct PlaceholderRand {
    distribution: DistributionEnum,
    alias: Option<String>,
}

impl PlaceholderRand {
    pub fn new(range: u64, alias: Option<String>) -> Self {
        Self {
            distribution: DistributionEnum::new("uniform", range),
            alias,
        }
    }
    fn generate(&mut self) -> PlaceholderResult {
        PlaceholderResult::new(self.alias.clone(), format!("{}", self.distribution.sample(&mut rand::thread_rng())))
    }
}

#[derive(Clone, Debug)]
pub struct PlaceholderRange {
    distribution: DistributionEnum,
    range: u64,
    width: u64,
    alias: Option<String>,
}

impl PlaceholderRange {
    pub fn new(range: u64, width: u64, alias: Option<String>) -> Self {
        Self {
            distribution: DistributionEnum::new("uniform", range),
            range,
            width,
            alias,
        }
    }
    fn generate(&mut self) -> PlaceholderResult {
        let left = self.distribution.sample(&mut rand::thread_rng());
        let right = min(left + self.width, self.range - 1);
        PlaceholderResult::new_multi(self.alias.clone(), vec![left.to_string(), right.to_string()])
    }
}

#[derive(Clone, Debug)]
pub struct PlaceholderReference {
    ref_name: String,
}

impl PlaceholderReference {
    pub fn new(ref_name: String) -> Self {
        Self { ref_name }
    }
    fn generate(&mut self, aliases: &HashMap<String, Vec<String>>) -> PlaceholderResult {
        let ref_values = aliases.get(&self.ref_name);
        if let Some(ref_values) = ref_values {
            PlaceholderResult::new_multi(None, ref_values.clone())
        } else {
            eprint!("There is no reference names {}", self.ref_name);
            exit(1);
        }
    }
}
