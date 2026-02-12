use clap::Parser as ClapParser;
use colored_json::to_colored_json_auto;
use pest::Parser;
use pest_derive::Parser;
use regex::Regex;
use serde_json::{Map, Number, Value};

#[derive(Parser)]
#[grammar = "grammar.pest"]
struct SedParser;
// use std::fs;
// use std::io::Read;

/// Example: Options and flags
#[derive(ClapParser)]
struct Cli {
    /// Jed command (-c, --command)
    #[clap(short, long, action)]
    command: String,
    /// input_files, optional positional
    input_file: Option<String>,
}

#[derive(Clone)]
struct ArrayRange {
    begin: usize,
    end: usize,
}
#[derive(Clone)]
enum RangeType {
    Key(Regex),
    Array(ArrayRange),
}
enum JedCommand {
    Substitute(SubstituteParams),
    Print,
    Other(String),
}
struct SubstituteParams {
    pattern: Regex,
    replacement: String,
    flags: String,
}

fn main() {
    // Restore default SIGPIPE handling
    unsafe {
        libc::signal(libc::SIGPIPE, libc::SIG_DFL);
    }

    let cli = Cli::parse();
    let file_contents: String;

    match cli.input_file {
        Some(ref input_file) => {
            // println!("Input files: {:?}", input_file);
            file_contents = std::fs::read_to_string(&input_file)
                .expect("Something went wrong reading the file")
                .parse::<String>()
                .unwrap();
        }
        None => {
            std::process::exit(1);
        }
    }

    let input = &cli.command;
    let mut v: Value = serde_json::from_str(&file_contents).expect("pailla");
    let (stack, command) = parse_grammar(input);

    match command {
        JedCommand::Substitute(params) => {
            let pattern = params.pattern;
            let replacement = params.replacement;
            if stack.len() > 0 {
                v = substitute_values_on_specified_ranges(v, &stack, &pattern, &replacement);
            } else {
                v = substitute_values(v, &pattern, &replacement);
            }
        }
        JedCommand::Print => {
            v = print_on_specified_ranges(v, &stack);
        }
        JedCommand::Other(_) => {
            println!("Only substitute command is supported for now");
            std::process::exit(1);
        }
    }

    println!("{}", to_colored_json_auto(&v).unwrap());
}

fn parse_grammar(input: &String) -> (Vec<RangeType>, JedCommand) {
    let mut stack = Vec::new();
    let parsed = match SedParser::parse(Rule::substitute, &input) {
        Ok(parsed) => parsed,
        Err(e) => {
            println!("Failed to parse command: {}", e);
            std::process::exit(1);
        }
    };
    let mut pattern = Regex::new("").unwrap();
    let mut replacement = String::from("");
    let mut flags = String::from("");
    let mut sed_command = ' ';
    for pair in parsed.into_iter().next().unwrap().into_inner() {
        match pair.as_rule() {
            Rule::range_regex => {
                for inner_pair in pair.into_inner() {
                    match inner_pair.as_rule() {
                        Rule::key_range_regex => {
                            stack.push(RangeType::Key(
                                Regex::new(inner_pair.as_str().trim_matches('/')).unwrap(),
                            ));
                        }
                        Rule::array_range_regex => {
                            let mut begin = 0;
                            let mut end = 0;
                            for ip in inner_pair.into_inner() {
                                match ip.as_rule() {
                                    Rule::array_range_regex_begin => {
                                        begin = ip.as_str().parse::<usize>().unwrap();
                                    }
                                    Rule::array_range_regex_end => {
                                        end = ip.as_str().parse::<usize>().unwrap();
                                    }
                                    _ => (),
                                }
                            }

                            stack.push(RangeType::Array(ArrayRange { begin, end }));
                        }
                        _ => (),
                    }
                }
            }
            Rule::sed_command => {
                sed_command = pair.as_str().chars().next().unwrap();
            }
            Rule::pattern => pattern = Regex::new(pair.as_str()).unwrap(),
            Rule::replacement => replacement = pair.as_str().to_string(),
            Rule::flags => flags = pair.as_str().to_string(),
            _ => {}
        }
    }

    if sed_command == 's' {
        return (
            stack,
            JedCommand::Substitute(SubstituteParams {
                pattern,
                replacement,
                flags,
            }),
        );
    }
    if sed_command == 'p' {
        return (stack, JedCommand::Print);
    }
    return (stack, JedCommand::Other(String::from("temporary")));
}

/// Performs a substitution on the keys of the JSON recursively.
fn substitute_keys(v: Value, replace_regex: &String, replace_with: &String) -> Value {
    let re = Regex::new(&replace_regex).unwrap();
    let v = match v {
        Value::Object(old_map) => {
            let mut new_map: Map<String, Value> = Map::new();
            for (k, v) in old_map {
                let new_key = re.replace_all(&k, replace_with).into_owned();
                let new_v = substitute_keys(v, replace_regex, replace_with);
                new_map.insert(new_key, new_v);
            }
            Value::Object(new_map)
        }
        Value::String(v) => Value::String(v),
        Value::Array(v) => {
            let mut new_vec = Vec::new();
            for value in v {
                let new_v = substitute_keys(value, replace_regex, replace_with);
                new_vec.push(new_v);
            }
            Value::Array(new_vec)
        }
        Value::Null => Value::Null,
        Value::Bool(v) => Value::Bool(v),
        Value::Number(v) => Value::Number(v),
    };
    return v;
}
fn substitute_values(v: Value, search_regexp: &Regex, replace_with: &String) -> Value {
    let v = match v {
        Value::Object(old_map) => {
            let mut new_map: Map<String, Value> = Map::new();
            for (k, v) in old_map {
                let new_v = substitute_values(v, search_regexp, replace_with);
                new_map.insert(k, new_v);
            }
            Value::Object(new_map)
        }
        Value::String(v) => Value::String(search_regexp.replace_all(&v, replace_with).into_owned()),
        Value::Array(v) => {
            let mut new_vec = Vec::new();
            for value in v {
                let new_v = substitute_values(value, search_regexp, replace_with);
                new_vec.push(new_v);
            }
            Value::Array(new_vec)
        }
        Value::Null => {
            let old_null = "null".to_string();
            let old_null_replaced = search_regexp
                .replace_all(&old_null, replace_with)
                .into_owned();
            if &old_null == &old_null_replaced {
                return Value::Null;
            } else {
                match &old_null_replaced.parse::<i128>() {
                    Ok(int) => return Value::Number(Number::from_i128(*int).unwrap()),
                    Err(_) => (),
                }
                match &old_null_replaced.parse::<f64>() {
                    Ok(float) => return Value::Number(Number::from_f64(*float).unwrap()),
                    Err(_) => (),
                }
                match &old_null_replaced.parse::<bool>() {
                    Ok(new_bool) => return Value::Bool(*new_bool),
                    Err(_) => (),
                }
            }
            return Value::String(old_null_replaced);
        }
        Value::Bool(v) => {
            let old_bool = v.to_string();
            let old_bool_replaced = search_regexp
                .replace_all(&old_bool, replace_with)
                .into_owned();
            if &old_bool == &old_bool_replaced {
                return Value::Bool(v);
            } else {
                match &old_bool_replaced.parse::<bool>() {
                    Ok(new_bool) => return Value::Bool(*new_bool),
                    Err(_) => (),
                }
            }
            return Value::String(old_bool_replaced);
        }
        Value::Number(v) => {
            let old_number = v.to_string();
            let old_number_replaced = search_regexp
                .replace_all(&old_number, replace_with)
                .into_owned();
            if &old_number == &old_number_replaced {
                return Value::Number(v);
            } else {
                match &old_number_replaced.parse::<i128>() {
                    Ok(int) => return Value::Number(Number::from_i128(*int).unwrap()),
                    Err(_) => (),
                }
                match &old_number_replaced.parse::<f64>() {
                    Ok(float) => return Value::Number(Number::from_f64(*float).unwrap()),
                    Err(_) => (),
                }
            }
            return Value::String(old_number_replaced);
        }
    };
    return v;
}
fn filter_key(v: Value, stack: &Vec<String>) -> Value {
    let mut response = Value::Null;
    match &stack.len() {
        0 => (),
        1 => {
            response = match v {
                Value::Object(current) => {
                    let mut new_stack = stack.clone();
                    let re = Regex::new(&new_stack.remove(0)).unwrap();
                    let mut new_map: Map<String, Value> = Map::new();
                    let mut found_something = false;
                    for (k, v) in &current {
                        if re.find(&k).is_some() {
                            new_map.insert(k.clone(), v.clone());
                            found_something = true;
                        }
                    }
                    if found_something {
                        return serde_json::Value::Object(new_map);
                    } else {
                        return serde_json::Value::Null;
                    }
                }
                Value::String(_) => serde_json::Value::Null,
                Value::Array(_) => serde_json::Value::Null,
                Value::Null => serde_json::Value::Null,
                Value::Bool(_) => serde_json::Value::Null,
                Value::Number(_) => serde_json::Value::Null,
            };
        }
        _ => {
            response = match v {
                Value::Object(current) => {
                    let mut new_stack = stack.clone();
                    let re = Regex::new(&new_stack.remove(0)).unwrap();
                    let mut new_map: Map<String, Value> = Map::new();
                    let mut found_something = false;
                    for (k, v) in &current {
                        if re.find(&k).is_some() {
                            // response = serde_json::Value::Object(current.clone());
                            let new_v = filter_key(v.clone(), &new_stack);
                            if new_v != Value::Null {
                                new_map.insert(k.clone(), new_v);
                            }
                            found_something = true;
                        }
                    }
                    if found_something {
                        return serde_json::Value::Object(new_map);
                    } else {
                        return serde_json::Value::Null;
                    }
                }
                Value::String(_) => serde_json::Value::Null,
                Value::Array(_) => serde_json::Value::Null,
                Value::Null => serde_json::Value::Null,
                Value::Bool(_) => serde_json::Value::Null,
                Value::Number(_) => serde_json::Value::Null,
            };
        } // _ => (),
    };
    return response;
}

/// This function performs the substitution only in the values that match the filter "stack"
fn print_on_specified_ranges(v: Value, stack: &Vec<RangeType>) -> Value {
    let mut response = Value::Null;
    match &stack.len() {
        0 => (),
        1 => {
            response = match v {
                Value::Object(current) => {
                    let mut new_stack = stack.clone();
                    match new_stack.remove(0) {
                        RangeType::Key(re) => {
                            let mut new_map: Map<String, Value> = Map::new();
                            for (k, v) in &current {
                                if re.find(&k).is_some() {
                                    new_map.insert(k.clone(), v.clone());
                                }
                            }
                            return serde_json::Value::Object(new_map);
                        }
                        RangeType::Array(_) => return serde_json::Value::Null,
                    }
                }
                Value::String(_) => serde_json::Value::Null,
                Value::Array(v) => {
                    let mut new_stack = stack.clone();
                    match new_stack.remove(0) {
                        RangeType::Key(_) => return serde_json::Value::Null,
                        RangeType::Array(array_range) => {
                            let mut new_vec: Vec<Value> = Vec::new();
                            for (i, val) in v.iter().enumerate() {
                                if i >= array_range.begin && i <= array_range.end {
                                    new_vec.push(val.clone());
                                }
                            }
                            return serde_json::Value::Array(new_vec);
                        }
                    }
                }
                Value::Null => serde_json::Value::Null,
                Value::Bool(_) => serde_json::Value::Null,
                Value::Number(_) => serde_json::Value::Null,
            };
        }
        _ => {
            response = match v {
                Value::Object(current) => {
                    let mut new_stack = stack.clone();
                    match new_stack.remove(0) {
                        RangeType::Key(re) => {
                            let mut new_map: Map<String, Value> = Map::new();
                            for (k, v) in &current {
                                if re.find(&k).is_some() {
                                    let new_v = print_on_specified_ranges(v.clone(), &new_stack);
                                    new_map.insert(k.clone(), new_v);
                                }
                            }
                            return serde_json::Value::Object(new_map);
                        }
                        RangeType::Array(_) => {
                            return serde_json::Value::Null;
                        }
                    }
                }
                Value::String(_) => serde_json::Value::Null,
                Value::Array(_) => serde_json::Value::Null,
                Value::Null => serde_json::Value::Null,
                Value::Bool(_) => serde_json::Value::Null,
                Value::Number(_) => serde_json::Value::Null,
            };
        } // _ => (),
    };
    return response;
}
/// This function performs the substitution only in the values that match the filter "stack"
fn substitute_values_on_specified_ranges(
    v: Value,
    stack: &Vec<RangeType>,
    old_regexp: &Regex,
    replace_with: &String,
) -> Value {
    let mut response = Value::Null;
    match &stack.len() {
        0 => (),
        1 => {
            response = match v {
                Value::Object(current) => {
                    let mut new_stack = stack.clone();
                    match new_stack.remove(0) {
                        RangeType::Key(re) => {
                            let mut new_map: Map<String, Value> = Map::new();
                            for (k, v) in &current {
                                if re.find(&k).is_some() {
                                    new_map.insert(
                                        k.clone(),
                                        substitute_values(v.clone(), &old_regexp, &replace_with),
                                    );
                                } else {
                                    new_map.insert(k.clone(), v.clone());
                                }
                            }
                            return serde_json::Value::Object(new_map);
                        }
                        RangeType::Array(_) => return serde_json::Value::Null,
                    }
                }
                Value::String(_) => serde_json::Value::Null,
                Value::Array(v) => {
                    let mut new_stack = stack.clone();
                    match new_stack.remove(0) {
                        RangeType::Key(_) => return serde_json::Value::Null,
                        RangeType::Array(array_range) => {
                            let mut new_vec: Vec<Value> = Vec::new();
                            for (i, val) in v.iter().enumerate() {
                                if i >= array_range.begin && i <= array_range.end {
                                    new_vec.push(substitute_values(
                                        val.clone(),
                                        &old_regexp,
                                        &replace_with,
                                    ));
                                } else {
                                    new_vec.push(val.clone());
                                }
                            }
                            return serde_json::Value::Array(new_vec);
                        }
                    }
                }
                Value::Null => serde_json::Value::Null,
                Value::Bool(_) => serde_json::Value::Null,
                Value::Number(_) => serde_json::Value::Null,
            };
        }
        _ => {
            response = match v {
                Value::Object(current) => {
                    let mut new_stack = stack.clone();
                    match new_stack.remove(0) {
                        RangeType::Key(re) => {
                            let mut new_map: Map<String, Value> = Map::new();
                            for (k, v) in &current {
                                if re.find(&k).is_some() {
                                    // response = serde_json::Value::Object(current.clone());
                                    let new_v = substitute_values_on_specified_ranges(
                                        v.clone(),
                                        &new_stack,
                                        old_regexp,
                                        replace_with,
                                    );
                                    new_map.insert(k.clone(), new_v);
                                } else {
                                    new_map.insert(k.clone(), v.clone());
                                }
                            }
                            return serde_json::Value::Object(new_map);
                        }
                        RangeType::Array(_) => {
                            return serde_json::Value::Null;
                        }
                    }
                }
                Value::String(_) => serde_json::Value::Null,
                Value::Array(_) => serde_json::Value::Null,
                Value::Null => serde_json::Value::Null,
                Value::Bool(_) => serde_json::Value::Null,
                Value::Number(_) => serde_json::Value::Null,
            };
        } // _ => (),
    };
    return response;
}

mod tests {
    use super::*;
    #[test]
    fn test_substitute_keys_1() {
        let some_json = r#"
        {"sha": "0eb3da11ed489189963045a3d4eb21ba343736cb", "node_id": "C_kwDOAE3WVdoAKDBlYjNkYTExZWQ0ODkxODk5NjMwNDVhM2Q0ZWIyMWJhMzQzNzM2Y2I"}"#;
        let mut v: Value = serde_json::from_str(some_json).unwrap();
        v = substitute_keys(v, &String::from("sha"), &String::from("new_sha"));
        assert_eq!(v["new_sha"], "0eb3da11ed489189963045a3d4eb21ba343736cb");
    }

    #[test]
    fn test_substitute_keys_recursivity() {
        let some_json = r#"
        {
          "commit": {
            "author": {
              "name": "bigmoonbit"
            }
        }
        }"#;
        let mut v: Value = serde_json::from_str(some_json).unwrap();
        v = substitute_keys(v, &String::from("a"), &String::from("o"));
        assert_eq!(v["commit"]["outhor"]["nome"], "bigmoonbit");
    }
    #[test]
    fn test_substitute_keys_repeated_keys_keeps_last() {
        let some_json = r#"
        {
          "commit": {
            "author": {
              "name": "bigmoonbit",
              "nombre": "hola"
            }
        }
        }"#;
        let mut v: Value = serde_json::from_str(some_json).unwrap();
        v = substitute_keys(v, &String::from("nombre"), &String::from("name"));
        assert_eq!(v["commit"]["author"]["name"], "hola");
    }
    #[test]
    fn test_substitute_keys_recursivity_inside_lists() {
        let some_json = r#"
        {
          "commit": [
            { "author": "camilo" },
            { "author": "andres" }
            ]
        }"#;
        let mut v: Value = serde_json::from_str(some_json).unwrap();
        v = substitute_keys(v, &String::from("author"), &String::from("autor"));
        assert_eq!(v["commit"][0]["autor"], "camilo");
        assert_eq!(v["commit"][1]["autor"], "andres");
    }
    #[test]
    fn test_substitute_values() {
        let some_json = r#"
        {
          "commit": {
            "author": {
              "name": "bigmoonbit"
            }
        }
        }"#;
        let mut v: Value = serde_json::from_str(some_json).unwrap();
        v = substitute_values(v, &Regex::new("oo").unwrap(), &String::from("AAA"));
        assert_eq!(v["commit"]["author"]["name"], "bigmAAAnbit");
    }
    #[test]
    fn test_substitute_values_2() {
        let some_json = r#"
        {
          "commit": {
            "author": {
              "name": "bigmoonbit"
            }
        }
        }"#;
        let mut v: Value = serde_json::from_str(some_json).unwrap();
        v = substitute_values(v, &Regex::new("o").unwrap(), &String::from("A"));
        assert_eq!(v["commit"]["author"]["name"], "bigmAAnbit");
    }
    #[test]
    fn test_substitute_values_recursivity_inside_lists() {
        let some_json = r#"
        {
          "commit": [
            { "author": "camilo" },
            { "author": "andres" }
            ]
        }"#;
        let mut v: Value = serde_json::from_str(some_json).unwrap();
        v = substitute_values(v, &Regex::new("andres").unwrap(), &String::from("mata"));
        assert_eq!(v["commit"][1]["author"], "mata");
    }
    #[test]
    fn test_substitute_values_recursivity_with_list_in_the_root() {
        let some_json = r#"
        [
            { "author": "camilo" },
            { "author": "andres" }
        ]
        "#;
        let mut v: Value = serde_json::from_str(some_json).unwrap();
        v = substitute_values(v, &Regex::new("andres").unwrap(), &String::from("mata"));
        assert_eq!(v[1]["author"], "mata");
    }
    #[test]
    fn test_substitute_values_numbers_can_be_replaced() {
        let some_json = r#"
        {
          "commit": {
            "author": {
              "name": 5
            }
        }
        }"#;
        let mut v: Value = serde_json::from_str(some_json).unwrap();
        v = substitute_values(v, &Regex::new("5").unwrap(), &String::from("6"));
        assert_eq!(v["commit"]["author"]["name"], 6);
    }
    #[test]
    fn test_substitute_values_booleans_can_be_modified() {
        let some_json = r#"
        {
          "commit": {
            "author": {
              "name": true
            }
        }
        }"#;
        let mut v: Value = serde_json::from_str(some_json).unwrap();
        v = substitute_values(v, &Regex::new("true").unwrap(), &String::from("false"));
        assert_eq!(v["commit"]["author"]["name"], false);
    }
    #[test]
    fn test_substitute_values_random_bug() {
        let some_json = r#"
        {
        "sha": "03cb1e19da91f0df728914d4c8717f7490df04e4"
        }"#;
        let mut v: Value = serde_json::from_str(some_json).unwrap();
        v = substitute_values(v, &Regex::new(".+").unwrap(), &String::from("hola"));
        assert_eq!(v["sha"], "hola");
    }
    #[test]
    fn test_substitute_values_numbers_can_be_replaced_2() {
        let some_json = r#"
        {
        "sha": 0
        }"#;
        let mut v: Value = serde_json::from_str(some_json).unwrap();
        v = substitute_values(v, &Regex::new(".+").unwrap(), &String::from("hola"));
        assert_eq!(v["sha"], "hola");
    }
    #[test]
    fn test_substitute_values_nulls_can_be_replaced() {
        let some_json = r#"
        {
        "sha": null 
        }"#;
        let mut v: Value = serde_json::from_str(some_json).unwrap();
        v = substitute_values(v, &Regex::new(".+").unwrap(), &String::from("hola"));
        assert_eq!(v["sha"], "hola");
    }
    #[test]
    fn test_substitute_values_new_lines_can_be_replaced() {
        let some_json = r#"
        {
        "sha": "a\\nb"
        }"#;
        let mut v: Value = serde_json::from_str(some_json).unwrap();
        v = substitute_values(v, &Regex::new(".+").unwrap(), &String::from("hola"));
        assert_eq!(v["sha"], "hola");
    }
    #[test]
    fn test_filter_0() {
        let some_json = r#"
        {
            "name": "camilo"
        }"#;
        let mut v: Value = serde_json::from_str(some_json).unwrap();
        let stack = vec![String::from("nothing")];
        v = filter_key(v, &stack);
        assert_eq!(v["name"], Value::Null);
    }
    #[test]
    fn test_filter_1() {
        let some_json = r#"
        {
            "name": "camilo"
        }"#;
        let mut v: Value = serde_json::from_str(some_json).unwrap();
        let stack = vec![String::from("name")];
        v = filter_key(v, &stack);
        assert_eq!(v["name"], "camilo");
    }
    #[test]
    fn test_filter_2() {
        let some_json = r#"
        {
            "name": "camilo",
            "nombre": "andres"
        }"#;
        let mut v: Value = serde_json::from_str(some_json).unwrap();
        let stack = vec![String::from("name")];
        v = filter_key(v, &stack);
        assert_eq!(v["nombre"], Value::Null);
    }
    #[test]
    fn test_filter_3() {
        let some_json = r#"
        {
            "author": {
              "name": "bigmoonbit",
              "nombre": "hola"
            }
        }"#;
        let mut v: Value = serde_json::from_str(some_json).unwrap();
        let stack = vec![String::from("author"), String::from("name")];
        v = filter_key(v, &stack);
        println!("{}", serde_json::to_string_pretty(&v).unwrap());
        assert_eq!(v["author"]["name"], "bigmoonbit");
        assert_eq!(v["author"]["nombre"], Value::Null);
    }
    #[test]
    fn test_filter_4() {
        let some_json = r#"
        {
          "commit": {
            "author": {
              "name": "bigmoonbit",
              "nombre": "hola"
            }
        }
        }"#;
        let mut v: Value = serde_json::from_str(some_json).unwrap();
        let stack = vec![
            String::from("commit"),
            String::from("author"),
            String::from("name"),
        ];
        v = filter_key(v, &stack);
        println!("{}", serde_json::to_string_pretty(&v).unwrap());
        assert_eq!(v["commit"]["author"]["name"], "bigmoonbit");
        assert_eq!(v["commit"]["author"]["nombre"], Value::Null);
    }
    #[test]
    fn test_filter_5() {
        let some_json = r#"
        {
          "commit": {
            "author": {
              "name": "bigmoonbit",
              "nombre": "hola"
            }
        }
        }"#;
        let mut v: Value = serde_json::from_str(some_json).unwrap();
        let stack = vec![String::from("commit")];
        v = filter_key(v, &stack);
        println!("{}", serde_json::to_string_pretty(&v).unwrap());
        assert_eq!(v["commit"]["author"]["name"], "bigmoonbit");
        assert_eq!(v["commit"]["author"]["nombre"], "hola");
    }
    #[test]
    fn test_grammar_1() {
        // let input = String::from("/c/./d/ s/sha/new_sha/g");
        let input = String::from("s/sha/new_sha/g");
        let parsed = SedParser::parse(Rule::substitute, &input).expect("failed to parse");
        let mut _pattern: String;
        let mut _replacement: String;
        let mut _flags: String;
        for pair in parsed.into_iter().next().unwrap().into_inner() {
            match pair.as_rule() {
                Rule::pattern => _pattern = pair.as_str().to_string(),
                Rule::replacement => _replacement = pair.as_str().to_string(),
                Rule::flags => _flags = pair.as_str().to_string(),
                _ => {}
            }
        }
    }
    #[test]
    fn test_grammar_2() {
        let input = String::from("/c/s/sha/new_sha/g");
        // let input = String::from("s/sha/new_sha/g");
        let _parsed = SedParser::parse(Rule::substitute, &input).expect("failed to parse");
        let input = String::from("/c/ s/sha/new_sha/g");
        // let input = String::from("s/sha/new_sha/g");
        let _parsed = SedParser::parse(Rule::substitute, &input).expect("failed to parse");
        let input = String::from("/c/./d/ s/sha/new_sha/g");
        // let input = String::from("s/sha/new_sha/g");
        let _parsed = SedParser::parse(Rule::substitute, &input).expect("failed to parse");
        let input = String::from("/c/./d/./e/ s/sha/new_sha/g");
        // let input = String::from("s/sha/new_sha/g");
        let parsed = SedParser::parse(Rule::substitute, &input).expect("failed to parse");
        for pair in parsed.into_iter().next().unwrap().into_inner() {
            match pair.as_rule() {
                Rule::pattern => assert_eq!(pair.as_str(), "sha"),
                Rule::replacement => assert_eq!(pair.as_str(), "new_sha"),
                Rule::flags => assert_eq!(pair.as_str(), "g"),
                Rule::range_regex => assert_eq!(pair.as_str(), "/c/./d/./e/"),
                _ => {}
            }
        }
    }
    #[test]
    fn test_grammar_3() {
        let input = String::from("/commit/s/a/XXXX/g");
        let parsed = SedParser::parse(Rule::substitute, &input).expect("failed to parse");
        for pair in parsed.into_iter().next().unwrap().into_inner() {
            match pair.as_rule() {
                Rule::pattern => assert_eq!(pair.as_str(), "a"),
                Rule::replacement => assert_eq!(pair.as_str(), "XXXX"),
                Rule::flags => assert_eq!(pair.as_str(), "g"),
                Rule::range_regex => assert_eq!(pair.as_str(), "/commit/"),
                _ => {}
            }
        }
    }
    #[test]
    fn test_grammar_4() {
        let input = String::from("1,3s/a/XXXX/g");
        let (stack, command) = parse_grammar(&input);
        match command {
            JedCommand::Substitute(params) => {
                assert_eq!(params.pattern.as_str(), "a");
                assert_eq!(params.replacement, "XXXX");
                assert_eq!(params.flags, "g");
            }
            _ => assert!(false),
        }
        assert_eq!(stack.len(), 1);
        match stack.first().unwrap() {
            RangeType::Array(array_range) => {
                assert_eq!(array_range.begin, 1);
                assert_eq!(array_range.end, 3);
            }
            _ => assert!(false),
        }
        let input = String::from("/first_key/.1,3./second_key/s/a/b/g");
        let (stack, command) = parse_grammar(&input);
        match command {
            JedCommand::Substitute(params) => {
                assert_eq!(params.pattern.as_str(), "a");
                assert_eq!(params.replacement, "b");
                assert_eq!(params.flags, "g");
            }
            _ => assert!(false),
        }
        assert_eq!(stack.len(), 3);
        match &stack[0] {
            RangeType::Key(key_regex) => {
                assert_eq!(
                    key_regex.as_str(),
                    Regex::new("first_key").unwrap().as_str()
                );
            }
            _ => assert!(false),
        }
        match &stack[1] {
            RangeType::Array(array_range) => {
                assert_eq!(array_range.begin, 1);
                assert_eq!(array_range.end, 3);
            }
            _ => assert!(false),
        }
        match &stack[2] {
            RangeType::Key(key_regex) => {
                assert_eq!(
                    key_regex.as_str(),
                    Regex::new("second_key").unwrap().as_str()
                );
            }
            _ => assert!(false),
        }
    }
    #[test]
    fn test_grammar_5() {
        let input = String::from("1,3p");
        let (stack, command) = parse_grammar(&input);
        match command {
            JedCommand::Print => {
                assert!(true)
            }
            _ => assert!(false),
        }
    }
    #[test]
    fn test_filter_substitute_1() {
        let some_json = r#"
        {
          "commit": {
            "author": {
              "name": "bigmoonbit",
              "nombre": "hoola"
            }
        }
        }"#;
        let mut v: Value = serde_json::from_str(some_json).unwrap();
        let stack = vec![
            RangeType::Key(Regex::new("commit").unwrap()),
            RangeType::Key(Regex::new("author").unwrap()),
            RangeType::Key(Regex::new("name").unwrap()),
        ];
        let old_regex = Regex::new("oo").unwrap();
        let new_regex = String::from("AA");
        v = substitute_values_on_specified_ranges(v, &stack, &old_regex, &new_regex);
        println!("{}", serde_json::to_string_pretty(&v).unwrap());
        assert_eq!(v["commit"]["author"]["name"], "bigmAAnbit");
        assert_eq!(v["commit"]["author"]["nombre"], "hoola");
    }
    #[test]
    fn test_filter_substitute_with_arrays() {
        let some_json = r#"
        {
          "commit": [
            {
              "name": "camilo"
            },
            {
              "name": "andres"
            }
            ]
        }"#;
        let mut v: Value = serde_json::from_str(some_json).unwrap();
        let stack = vec![RangeType::Key(Regex::new("commit").unwrap())];
        let old_regex = Regex::new("a").unwrap();
        let new_regex = String::from("x");
        v = substitute_values_on_specified_ranges(v, &stack, &old_regex, &new_regex);
        println!("{}", serde_json::to_string_pretty(&v).unwrap());
        assert_eq!(v["commit"][0]["name"], "cxmilo");
        assert_eq!(v["commit"][1]["name"], "xndres");
    }
    #[test]
    fn test_filter_substitute_with_arrays_and_ranges() {
        let some_json = r#"
        {
          "commit": [
            {
              "name": "camilo"
            },
            {
              "name": "andres"
            }
            ]
        }"#;
        let mut v: Value = serde_json::from_str(some_json).unwrap();
        let stack = vec![
            RangeType::Key(Regex::new("commit").unwrap()),
            RangeType::Array(ArrayRange { begin: 0, end: 0 }),
        ];
        let search_regex = Regex::new("a").unwrap();
        let replace_with = String::from("x");
        v = substitute_values_on_specified_ranges(v, &stack, &search_regex, &replace_with);
        println!("{}", serde_json::to_string_pretty(&v).unwrap());
        assert_eq!(v["commit"][0]["name"], "cxmilo");
        assert_eq!(v["commit"][1]["name"], "andres");
    }
    #[test]
    fn test_filter_substitute_with_arrays_and_ranges_2() {
        let some_json = r#"
        {
          "commit": [
            {
              "name": "camilo"
            },
            {
              "name": "andres"
            }
            ]
        }"#;
        let mut v: Value = serde_json::from_str(some_json).unwrap();
        let stack = vec![
            RangeType::Key(Regex::new("commit").unwrap()),
            RangeType::Key(Regex::new("name").unwrap()),
        ];
        let search_regex = Regex::new("a").unwrap();
        let replace_with = String::from("x");
        v = substitute_values_on_specified_ranges(v, &stack, &search_regex, &replace_with);
        println!("{}", serde_json::to_string_pretty(&v).unwrap());
        assert_eq!(v["commit"][0]["name"], Value::Null);
        assert_eq!(v["commit"][1]["name"], Value::Null);
    }
    // /c/.,3./e/ -> vec!["c", "d", "e"]
    #[test]
    fn test_parsing_regex() {
        let range_regex = String::from("/c/./d/./e/");
        let answer = vec![String::from("c"), String::from("d"), String::from("e")];

        let vec_range_regex = range_regex
            .replace("/", "")
            .split(".")
            .map(|s| s.to_string())
            .collect::<Vec<String>>();
        assert_eq!(vec_range_regex, answer);
    }
    #[test]
    fn test_print_1() {
        let some_json = r#"
        {
          "commit": [
            {
              "name": "camilo"
            },
            {
              "name": "andres"
            }
            ]
        }"#;
        let mut v: Value = serde_json::from_str(some_json).unwrap();
        let stack = vec![
            RangeType::Key(Regex::new("commit").unwrap()),
            RangeType::Array(ArrayRange { begin: 0, end: 0 }),
        ];
        v = print_on_specified_ranges(v, &stack);
        println!("{}", serde_json::to_string_pretty(&v).unwrap());
        assert_eq!(v["commit"][0]["name"], "camilo");
        assert_eq!(v["commit"][1]["name"], Value::Null);

        let stack = vec![RangeType::Key(Regex::new("doesnt-exists").unwrap())];
        v = print_on_specified_ranges(v, &stack);
        println!("{}", serde_json::to_string_pretty(&v).unwrap());
        assert_eq!(v["commit"][0]["name"], Value::Null);
        assert_eq!(v["commit"][1]["name"], Value::Null);
    }
}
