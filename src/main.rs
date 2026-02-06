use regex::Regex;
use serde_json::{Map, Number, Result, Value};
fn main() {
    untyped_example();
}

fn untyped_example() -> Result<()> {
    // Some JSON input data as a &str. Maybe this comes from the user.
    let data = r#"
        {
            "name": "John Doe",
            "age": 43,
            "phones": [
                "+44 1234567",
                "+44 2345678"
            ]
        }"#;

    // Parse the string of data into serde_json::Value.
    let v: Value = serde_json::from_str(data)?;

    // Access parts of the data by indexing with square brackets.
    println!("Please call {} at the number {}", v["name"], v["phones"][0]);
    print(&v);

    Ok(())
}
fn print(v: &Value) {
    match v {
        Value::Object(v) => {
            println! {"Object with {} keys", v.len()};
            for (k, v) in v {
                println! {"Key: {}", k};
                print(v);
            }
            // print(&v);
        }
        Value::String(v) => {
            println! {"String with length {}", v};
        }
        Value::Array(v) => {
            println! {"Array of length {}", v.len()};
        }
        Value::Null => {}
        Value::Bool(v) => {
            println! {"Boolean with value {}", v};
        }
        Value::Number(v) => {
            println! {"Number with value {}", v};
        }
    };
}

fn key_substitute(v: Value, old_regexp: &String, new_regexp: &String) -> Value {
    let re = Regex::new(&old_regexp).unwrap();
    let v = match v {
        Value::Object(old_map) => {
            let mut new_map: Map<String, Value> = Map::new();
            for (k, v) in old_map {
                let new_key = re.replace(&k, new_regexp).into_owned();
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
                // let new_key = re.replace(&k, new_regexp).into_owned();
                let new_v = value_substitute(v, old_regexp, new_regexp);
                new_map.insert(k, new_v);
            }
            Value::Object(new_map)
        }
        // Value::String(v) => Value::String(re.replace(&v, new_regexp).into_owned()),
        Value::String(v) => {
            println!("value: {}", v);
            println!(
                "value replaced: {}",
                re.replace(&v, new_regexp).into_owned()
            );
            Value::String(re.replace(&v, new_regexp).into_owned())
        }
        Value::Array(v) => {
            let mut new_vec = Vec::new();
            for value in v {
                let new_v = value_substitute(value, old_regexp, new_regexp);
                new_vec.push(new_v);
            }
            Value::Array(new_vec)
        }
        Value::Null => Value::Null,
        Value::Bool(v) => Value::Bool(v),
        Value::Number(v) => {
            let old_number = v.to_string();
            let old_number_replaced = re.replace(&old_number, new_regexp).into_owned();
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
}
