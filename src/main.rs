use clap::Parser as ClapParser;
use colored_json::to_colored_json_auto;
use pest::Parser;
use pest_derive::Parser;
use regex::Regex;
use serde_json::{Map, Number, Result, Value};

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

fn main() {
    let cli = Cli::parse();
    let mut file_contents = String::new();

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
    let parsed = SedParser::parse(Rule::substitute, input).expect("failed to parse");
    let mut pattern = String::new();
    let mut replacement = String::new();
    let mut flags = String::new();
    for pair in parsed.into_iter().next().unwrap().into_inner() {
        match pair.as_rule() {
            Rule::pattern => pattern = pair.as_str().to_string(),
            Rule::replacement => replacement = pair.as_str().to_string(),
            Rule::flags => flags = pair.as_str().to_string(),
            _ => {}
        }
    }

    let mut v: Value = serde_json::from_str(&file_contents).expect("pailla");

    v = value_substitute(v, &pattern, &replacement);
    // println!("{}", serde_json::to_string_pretty(&v).unwrap());
    //
    //
    // Restore default SIGPIPE handling
    unsafe {
        libc::signal(libc::SIGPIPE, libc::SIG_DFL);
    }

    println!("{}", to_colored_json_auto(&v).unwrap());
}

fn key_substitute(v: Value, old_regexp: &String, new_regexp: &String) -> Value {
    let re = Regex::new(&old_regexp).unwrap();
    let v = match v {
        Value::Object(old_map) => {
            let mut new_map: Map<String, Value> = Map::new();
            for (k, v) in old_map {
                let new_key = re.replace_all(&k, new_regexp).into_owned();
                let new_v = key_substitute(v, old_regexp, new_regexp);
                new_map.insert(new_key, new_v);
            }
            Value::Object(new_map)
        }
        Value::String(v) => Value::String(v),
        Value::Array(v) => {
            let mut new_vec = Vec::new();
            for value in v {
                let new_v = key_substitute(value, old_regexp, new_regexp);
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
fn value_substitute(v: Value, old_regexp: &String, new_regexp: &String) -> Value {
    let re = Regex::new(&old_regexp).unwrap();
    let v = match v {
        Value::Object(old_map) => {
            let mut new_map: Map<String, Value> = Map::new();
            for (k, v) in old_map {
                let new_v = value_substitute(v, old_regexp, new_regexp);
                new_map.insert(k, new_v);
            }
            Value::Object(new_map)
        }
        Value::String(v) => Value::String(re.replace_all(&v, new_regexp).into_owned()),
        Value::Array(v) => {
            let mut new_vec = Vec::new();
            for value in v {
                let new_v = value_substitute(value, old_regexp, new_regexp);
                new_vec.push(new_v);
            }
            Value::Array(new_vec)
        }
        Value::Null => {
            let old_null = "null".to_string();
            let old_null_replaced = re.replace_all(&old_null, new_regexp).into_owned();
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
            let old_bool_replaced = re.replace_all(&old_bool, new_regexp).into_owned();
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
            let old_number_replaced = re.replace_all(&old_number, new_regexp).into_owned();
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
                Value::String(v) => serde_json::Value::Null,
                Value::Array(v) => serde_json::Value::Null,
                Value::Null => serde_json::Value::Null,
                Value::Bool(v) => serde_json::Value::Null,
                Value::Number(v) => serde_json::Value::Null,
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
                Value::String(v) => serde_json::Value::Null,
                Value::Array(v) => serde_json::Value::Null,
                Value::Null => serde_json::Value::Null,
                Value::Bool(v) => serde_json::Value::Null,
                Value::Number(v) => serde_json::Value::Null,
            };
        } // _ => (),
    };
    return response;
}

mod tests {
    use super::*;
    #[test]
    fn test_key_substitute_1() {
        let some_json = r#"
        {"sha": "0eb3da11ed489189963045a3d4eb21ba343736cb", "node_id": "C_kwDOAE3WVdoAKDBlYjNkYTExZWQ0ODkxODk5NjMwNDVhM2Q0ZWIyMWJhMzQzNzM2Y2I"}"#;
        let mut v: Value = serde_json::from_str(some_json).unwrap();
        v = key_substitute(v, &String::from("sha"), &String::from("new_sha"));
        assert_eq!(v["new_sha"], "0eb3da11ed489189963045a3d4eb21ba343736cb");
    }

    #[test]
    fn test_key_substitute_recursivity() {
        let some_json = r#"
        {
          "commit": {
            "author": {
              "name": "bigmoonbit"
            }
        }
        }"#;
        let mut v: Value = serde_json::from_str(some_json).unwrap();
        v = key_substitute(v, &String::from("a"), &String::from("o"));
        assert_eq!(v["commit"]["outhor"]["nome"], "bigmoonbit");
    }
    #[test]
    fn test_key_substitute_repeated_keys_keeps_last() {
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
        v = key_substitute(v, &String::from("nombre"), &String::from("name"));
        assert_eq!(v["commit"]["author"]["name"], "hola");
    }
    #[test]
    fn test_key_substitute_recursivity_inside_lists() {
        let some_json = r#"
        {
          "commit": [
            { "author": "camilo" },
            { "author": "andres" }
            ]
        }"#;
        let mut v: Value = serde_json::from_str(some_json).unwrap();
        v = key_substitute(v, &String::from("author"), &String::from("autor"));
        assert_eq!(v["commit"][0]["autor"], "camilo");
        assert_eq!(v["commit"][1]["autor"], "andres");
    }
    #[test]
    fn test_value_substitute() {
        let some_json = r#"
        {
          "commit": {
            "author": {
              "name": "bigmoonbit"
            }
        }
        }"#;
        let mut v: Value = serde_json::from_str(some_json).unwrap();
        v = value_substitute(v, &String::from("oo"), &String::from("AAA"));
        assert_eq!(v["commit"]["author"]["name"], "bigmAAAnbit");
    }
    #[test]
    fn test_value_substitute_2() {
        let some_json = r#"
        {
          "commit": {
            "author": {
              "name": "bigmoonbit"
            }
        }
        }"#;
        let mut v: Value = serde_json::from_str(some_json).unwrap();
        v = value_substitute(v, &String::from("o"), &String::from("A"));
        assert_eq!(v["commit"]["author"]["name"], "bigmAAnbit");
    }
    #[test]
    fn test_value_substitute_recursivity_inside_lists() {
        let some_json = r#"
        {
          "commit": [
            { "author": "camilo" },
            { "author": "andres" }
            ]
        }"#;
        let mut v: Value = serde_json::from_str(some_json).unwrap();
        v = value_substitute(v, &String::from("andres"), &String::from("mata"));
        assert_eq!(v["commit"][1]["author"], "mata");
    }
    #[test]
    fn test_value_substitute_recursivity_with_list_in_the_root() {
        let some_json = r#"
        [
            { "author": "camilo" },
            { "author": "andres" }
        ]
        "#;
        let mut v: Value = serde_json::from_str(some_json).unwrap();
        v = value_substitute(v, &String::from("andres"), &String::from("mata"));
        assert_eq!(v[1]["author"], "mata");
    }
    #[test]
    fn test_value_substitute_numbers_can_be_replaced() {
        let some_json = r#"
        {
          "commit": {
            "author": {
              "name": 5
            }
        }
        }"#;
        let mut v: Value = serde_json::from_str(some_json).unwrap();
        v = value_substitute(v, &String::from("5"), &String::from("6"));
        assert_eq!(v["commit"]["author"]["name"], 6);
    }
    #[test]
    fn test_value_substitute_booleans_can_be_modified() {
        let some_json = r#"
        {
          "commit": {
            "author": {
              "name": true
            }
        }
        }"#;
        let mut v: Value = serde_json::from_str(some_json).unwrap();
        v = value_substitute(v, &String::from("true"), &String::from("false"));
        assert_eq!(v["commit"]["author"]["name"], false);
    }
    #[test]
    fn test_value_substitute_random_bug() {
        let some_json = r#"
        {
        "sha": "03cb1e19da91f0df728914d4c8717f7490df04e4"
        }"#;
        let mut v: Value = serde_json::from_str(some_json).unwrap();
        v = value_substitute(v, &String::from(".+"), &String::from("hola"));
        assert_eq!(v["sha"], "hola");
    }
    #[test]
    fn test_value_substitute_numbers_can_be_replaced_2() {
        let some_json = r#"
        {
        "sha": 0
        }"#;
        let mut v: Value = serde_json::from_str(some_json).unwrap();
        v = value_substitute(v, &String::from(".+"), &String::from("hola"));
        assert_eq!(v["sha"], "hola");
    }
    #[test]
    fn test_value_substitute_nulls_can_be_replaced() {
        let some_json = r#"
        {
        "sha": null 
        }"#;
        let mut v: Value = serde_json::from_str(some_json).unwrap();
        v = value_substitute(v, &String::from(".+"), &String::from("hola"));
        assert_eq!(v["sha"], "hola");
    }
    #[test]
    fn test_value_substitute_new_lines_can_be_replaced() {
        let some_json = r#"
        {
        "sha": "a\\nb"
        }"#;
        let mut v: Value = serde_json::from_str(some_json).unwrap();
        v = value_substitute(v, &String::from(".+"), &String::from("hola"));
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
    // #[test]
    // fn test_filter_4() {
    //     let some_json = r#"
    //     {
    //       "commit": {
    //         "author": {
    //           "name": "bigmoonbit",
    //           "nombre": "hola"
    //         }
    //     }
    //     }"#;
    //     let mut v: Value = serde_json::from_str(some_json).unwrap();
    //     let stack = vec![String::from("author"), String::from("name")];
    //     v = filter_key(v, &stack);
    //     println!("{}", serde_json::to_string_pretty(&v).unwrap());
    //     assert_eq!(v["commit"]["author"]["name"], "bigmoonbit");
    //     assert_eq!(v["commit"]["author"]["nombre"], Value::Null);
    // }
    // Posibilidades
    // 1. partir el string del filtro. y pasarlo como si fuera un stack al filtro
}
