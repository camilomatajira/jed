use super::grammar::{ArrayRange, RangeType, Rule, SedParser};
use super::*;
use pest::Parser;
use regex::Regex;

#[test]
fn test_substitute_keys_1() {
    let some_json = r#"
    {"sha": "0eb3da11ed489189963045a3d4eb21ba343736cb", "node_id": "C_kwDOAE3WVdoAKDBlYjNkYTExZWQ0ODkxODk5NjMwNDVhM2Q0ZWIyMWJhMzQzNzM2Y2I"}"#;
    let mut v: Value = serde_json::from_str(some_json).unwrap();
    let replace_regex = Regex::new("sha").unwrap();
    v = substitute_keys(v, &replace_regex, &String::from("new_sha"));
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
    let replace_regex = Regex::new("a").unwrap();
    v = substitute_keys(v, &replace_regex, &String::from("o"));
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
    let replace_regex = Regex::new("nombre").unwrap();
    v = substitute_keys(v, &replace_regex, &String::from("name"));
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
    let replace_regex = Regex::new("author").unwrap();
    v = substitute_keys(v, &replace_regex, &String::from("autor"));
    assert_eq!(v["commit"][0]["autor"], "camilo");
    assert_eq!(v["commit"][1]["autor"], "andres");
}
#[test]
fn test_substitute_keys_with_filters() {
    let some_json = r#"
    {
    "root":    {
          "commit": {
            "author": {
              "name": "camilo"
            },
            "contributor": {
              "name": "camilo"
            }
        }
        }
    }"#;
    let mut v: Value = serde_json::from_str(some_json).unwrap();
    let stack = vec![
        RangeType::Key(Regex::new("commit").unwrap()),
        RangeType::Key(Regex::new("author").unwrap()),
    ];
    let replace_regex = Regex::new("name").unwrap();
    v = substitute_keys_on_specified_ranges(v, &stack, &replace_regex, &String::from("nom"));
    println!("{}", serde_json::to_string_pretty(&v).unwrap());
    assert_eq!(v["root"]["commit"]["author"]["nom"], "camilo");
    assert_eq!(v["root"]["commit"]["contributor"]["name"], "camilo");
}
#[test]
fn test_substitute_keys_with_filters_2() {
    let some_json = r#"
    {
    "root" : {
      "commit": [
        { "author": "camilo" },
        { "author": "andres" }
        ]
        }

    }"#;
    let mut v: Value = serde_json::from_str(some_json).unwrap();
    let stack = vec![
        RangeType::Key(Regex::new("commit").unwrap()),
        RangeType::Array(ArrayRange { begin: 0, end: 0 }),
    ];
    let replace_regex = Regex::new("author").unwrap();
    v = substitute_keys_on_specified_ranges(v, &stack, &replace_regex, &String::from("nom"));
    println!("{}", serde_json::to_string_pretty(&v).unwrap());
    assert_eq!(v["root"]["commit"][0]["nom"], "camilo");
    assert_eq!(v["root"]["commit"][1]["author"], "andres");
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
fn test_grammar_1() {
    let input = String::from("s/sha/new_sha/g");
    let parsed = SedParser::parse(Rule::substitute, &input).expect("failed to parse");
    let mut _pattern: String;
    let mut _replacement: String;
    for pair in parsed.into_iter().next().unwrap().into_inner() {
        match pair.as_rule() {
            Rule::pattern => _pattern = pair.as_str().to_string(),
            Rule::replacement => _replacement = pair.as_str().to_string(),
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
            Rule::range_regex => assert_eq!(pair.as_str(), "/commit/"),
            _ => {}
        }
    }
}
#[test]
fn test_grammar_4() {
    let input = String::from("1,3s/a/XXXX/g");
    let (stack, command) = parse_grammar(&input).unwrap();
    match command {
        JedCommand::Substitute(params) => {
            assert_eq!(params.pattern.as_str(), "a");
            assert_eq!(params.replacement, "XXXX");
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
    let (stack, command) = parse_grammar(&input).unwrap();
    match command {
        JedCommand::Substitute(params) => {
            assert_eq!(params.pattern.as_str(), "a");
            assert_eq!(params.replacement, "b");
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
    let input = String::from("1,30p");
    let (_stack, command) = parse_grammar(&input).unwrap();
    match command {
        JedCommand::Print => {
            assert!(true)
        }
        _ => assert!(false),
    }
    let input = String::from("/connectors/.1,30p");
    let (stack, command) = parse_grammar(&input).unwrap();
    match command {
        JedCommand::Print => {
            assert!(true)
        }
        _ => assert!(false),
    }
    match &stack[0] {
        RangeType::Key(key_regex) => {
            assert_eq!(
                key_regex.as_str(),
                Regex::new("connectors").unwrap().as_str()
            );
        }
        _ => assert!(false),
    }
    match &stack[1] {
        RangeType::Array(array_range) => {
            assert_eq!(array_range.begin, 1);
            assert_eq!(array_range.end, 30);
        }
        _ => assert!(false),
    }
    let input = String::from("/connectors/.1,2 p");
    let (_, _) = parse_grammar(&input).unwrap();
}
#[test]
fn test_grammar_7() {
    let input = String::from(":/camilo/p");
    let (stack, command) = parse_grammar(&input).unwrap();
    match command {
        JedCommand::Print => {
            assert!(true)
        }
        _ => assert!(false),
    }
    match &stack[0] {
        RangeType::Value(value_regex) => {
            assert_eq!(value_regex.as_str(), Regex::new("camilo").unwrap().as_str());
        }
        _ => assert!(false),
    }
}
#[test]
fn test_filter_substitute_1() {
    let some_json = r#"
    { 
    "root": {
      "commit": {
        "author": {
          "name": "bigmoonbit",
          "nombre": "hoola"
        }
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
    assert_eq!(v["root"]["commit"]["author"]["name"], "bigmAAnbit");
    assert_eq!(v["root"]["commit"]["author"]["nombre"], "hoola");
}
#[test]
fn test_filter_substitute_with_arrays() {
    let some_json = r#"
    { "root":
    {
      "commit": [
        {
          "name": "camilo"
        },
        {
          "name": "andres"
        }
        ]
        }
    }"#;
    let mut v: Value = serde_json::from_str(some_json).unwrap();
    let stack = vec![RangeType::Key(Regex::new("commit").unwrap())];
    let old_regex = Regex::new("a").unwrap();
    let new_regex = String::from("x");
    v = substitute_values_on_specified_ranges(v, &stack, &old_regex, &new_regex);
    println!("{}", serde_json::to_string_pretty(&v).unwrap());
    assert_eq!(v["root"]["commit"][0]["name"], "cxmilo");
    assert_eq!(v["root"]["commit"][1]["name"], "xndres");
}
#[test]
fn test_filter_substitute_with_arrays_and_ranges() {
    let some_json = r#"
    { "root":
    {
      "commit": [
        {
          "name": "camilo"
        },
        {
          "name": "andres"
        }
        ]
        }
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
    assert_eq!(v["root"]["commit"][0]["name"], "cxmilo");
    assert_eq!(v["root"]["commit"][1]["name"], "andres");
}
#[test]
fn test_filter_substitute_with_arrays_and_ranges_2() {
    let some_json = r#"
    { "root":
    {
      "commit": [
        {
          "name": "camilo"
        },
        {
          "name": "andres"
        }
        ]
        }
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
#[test]
fn test_filter_substitute_with_arrays_and_ranges_3() {
    let some_json = r#"
    { "root":
    {
      "connectors": [
        {
          "auth_mechanism": "credentials",
          "available_auth_mechanisms": [
            "webauth",
            "credentials"
          ],
          "name": "Aachener Bank eG",
          "uuid": "c64a18a7-e071-487e-8318-f01c76896a29"
        }
      ]
      }
    }"#;
    let mut v: Value = serde_json::from_str(some_json).unwrap();
    let stack = vec![RangeType::Key(
        Regex::new("auth_mechanis|uuid|name").unwrap(),
    )];
    let search_regex = Regex::new("Bank|webauth").unwrap();
    let replace_with = String::from("PERRO");
    v = substitute_values_on_specified_ranges(v, &stack, &search_regex, &replace_with);
    println!("{}", serde_json::to_string_pretty(&v).unwrap());
    assert_eq!(v["root"]["connectors"][0]["name"], "Aachener PERRO eG");
    assert_eq!(
        v["root"]["connectors"][0]["available_auth_mechanisms"][0],
        "PERRO"
    );

    let mut v: Value = serde_json::from_str(some_json).unwrap();
    let stack = vec![
        RangeType::Array(ArrayRange { begin: 0, end: 0 }),
        RangeType::Key(Regex::new("auth_mechanis|uuid|name").unwrap()),
    ];
    let search_regex = Regex::new("Bank|webauth").unwrap();
    let replace_with = String::from("PERRO");
    v = substitute_values_on_specified_ranges(v, &stack, &search_regex, &replace_with);
    println!("{}", serde_json::to_string_pretty(&v).unwrap());
    assert_eq!(v["root"]["connectors"][0]["name"], "Aachener PERRO eG");
    assert_eq!(
        v["root"]["connectors"][0]["available_auth_mechanisms"][0],
        "PERRO"
    );

    let some_json = r#"
    { "root":
    {
      "connectors": 
        {
          "auth_mechanism": "credentials",
          "available_auth_mechanisms": [
            "webauth",
            "credentials"
          ],
          "name": "Aachener Bank eG",
          "uuid": "c64a18a7-e071-487e-8318-f01c76896a29"
        }
    }
      
    }"#;
    let mut v: Value = serde_json::from_str(some_json).unwrap();
    let stack = vec![
        RangeType::Key(Regex::new("connectors").unwrap()),
        RangeType::Key(Regex::new("auth_mechanis|uuid|name").unwrap()),
    ];
    let search_regex = Regex::new("Bank|webauth").unwrap();
    let replace_with = String::from("PERRO");
    v = substitute_values_on_specified_ranges(v, &stack, &search_regex, &replace_with);
    println!("{}", serde_json::to_string_pretty(&v).unwrap());
    assert_eq!(v["root"]["connectors"]["name"], "Aachener PERRO eG");
    assert_eq!(
        v["root"]["connectors"]["available_auth_mechanisms"][0],
        "PERRO"
    );
}
#[test]
fn test_substitute_value_ranges() {
    let some_json = r#"
    {
      "connectors": [
        {
          "auth_mechanism": "credentials_spanish",
          "available_auth_mechanisms": [
            "webauth",
            "credentials_french"
          ]
        }
      ]
    }
    "#;
    let mut v: Value = serde_json::from_str(some_json).unwrap();
    let stack = vec![RangeType::Value(Regex::new("spanish").unwrap())];
    let search_regex = Regex::new("credentials").unwrap();
    let replace_with = String::from("credenciales");
    v = substitute_values_on_specified_ranges(v, &stack, &search_regex, &replace_with);
    println!("{}", serde_json::to_string_pretty(&v).unwrap());
    assert_eq!(v["connectors"][0]["auth_mechanism"], "credenciales_spanish");
    assert_eq!(
        v["connectors"][0]["available_auth_mechanisms"][1],
        "credentials_french"
    );
}
// // /c/.,3./e/ -> vec!["c", "d", "e"]
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
    { "root":
    {
      "commit": [
        {
          "name": "camilo"
        },
        {
          "name": "andres"
        }
        ]
        }
    }"#;
    let mut v: Value = serde_json::from_str(some_json).unwrap();
    let stack = vec![
        RangeType::Key(Regex::new("commit").unwrap()),
        RangeType::Array(ArrayRange { begin: 0, end: 0 }),
    ];
    v = print_on_specified_ranges(v, &stack);
    println!("{}", serde_json::to_string_pretty(&v).unwrap());
    assert_eq!(v["root"]["commit"][0]["name"], "camilo");
    assert_eq!(v["root"]["commit"][1]["name"], Value::Null);

    let stack = vec![RangeType::Key(Regex::new("doesnt-exists").unwrap())];
    v = print_on_specified_ranges(v, &stack);
    println!("{}", serde_json::to_string_pretty(&v).unwrap());
    assert_eq!(v["root"]["commit"][0]["name"], Value::Null);
    assert_eq!(v["root"]["commit"][1]["name"], Value::Null);
}
#[test]
fn test_print_2() {
    let some_json = r#"
    { "root":
    {
      "connectors": [
        {
          "account_types": [
            "checking"
          ],
          "account_usages": []
         }
      ]
      }
    }"#;
    let mut v: Value = serde_json::from_str(some_json).unwrap();
    let stack = vec![
        RangeType::Key(Regex::new("connectors").unwrap()),
        RangeType::Array(ArrayRange { begin: 0, end: 0 }),
        RangeType::Key(Regex::new(".*type.*").unwrap()),
    ];
    v = print_on_specified_ranges(v, &stack);
    println!("{}", serde_json::to_string_pretty(&v).unwrap());
    assert_eq!(v["root"]["connectors"][0]["account_types"][0], "checking");
    assert_eq!(v["root"]["connectors"][0]["account_usages"], Value::Null);
}
#[test]
fn test_print_1_flexible() {
    let some_json = r#"
    {
      "connectors":
        {
          "account_types": [
            "checking"
          ],
          "account_usages": []
         }
    }"#;
    let mut v: Value = serde_json::from_str(some_json).unwrap();
    let stack = vec![RangeType::Key(Regex::new("account_types").unwrap())];
    v = print_on_specified_ranges(v, &stack);
    println!("{}", serde_json::to_string_pretty(&v).unwrap());
    assert_eq!(v["connectors"]["account_types"][0], "checking");
    assert_eq!(v["connectors"]["account_usages"], Value::Null);
    match v["connectors"].get("account_usages") {
        Some(_) => assert!(false),
        None => assert!(true),
    };
}
#[test]
fn test_print_2_flexible() {
    let some_json = r#"
    {
      "connectors":[
        {
          "account_types": [
            "checking"
          ],
          "account_usages": []
         }
         ]
    }"#;
    let mut v: Value = serde_json::from_str(some_json).unwrap();
    let stack = vec![RangeType::Key(Regex::new("account_types").unwrap())];
    v = print_on_specified_ranges(v, &stack);
    println!("{}", serde_json::to_string_pretty(&v).unwrap());
    assert_eq!(v["connectors"][0]["account_types"][0], "checking");
    match v["connectors"][0].get("account_usages") {
        Some(_) => assert!(false),
        None => assert!(true),
    };
}
#[test]
fn test_print_3_flexible() {
    let some_json = r#"
    {
      "connectors":[
        {
          "account_types": [
            "checking"
          ],
          "account_usages": []
         }
         ]
    }"#;
    let mut v: Value = serde_json::from_str(some_json).unwrap();
    let stack = vec![RangeType::Key(
        Regex::new("something that does not exists").unwrap(),
    )];
    v = print_on_specified_ranges(v, &stack);
    println!("{}", serde_json::to_string_pretty(&v).unwrap());
    assert_eq!(v, Value::Null);
}
#[test]
fn test_print_4_flexible() {
    let some_json = r#"
    {
      "key1":{
      "key11": {
      "key111": "a",
      "key112": "b"
      },
      "key112": "b"
      },

      "key2":{
      "key11": "c",
      "key12": "d"
      }
    }"#;
    let mut v: Value = serde_json::from_str(some_json).unwrap();
    let stack = vec![
        RangeType::Key(Regex::new("key1").unwrap()),
        RangeType::Key(Regex::new("key112").unwrap()),
    ];
    v = print_on_specified_ranges(v, &stack);
    println!("{}", serde_json::to_string_pretty(&v).unwrap());
    assert_eq!(v["key1"]["key112"], "b");
    match v["key1"].get("key11") {
        Some(_) => assert!(false),
        None => assert!(true),
    };
}
#[test]
fn test_print_5_flexible() {
    let some_json = r#"
    {
      "connectors": [
        {
          "stability": {
            "last_update": "a"
          }
        }
      ]

    }"#;
    let mut v: Value = serde_json::from_str(some_json).unwrap();
    let stack = vec![
        RangeType::Key(Regex::new("stability").unwrap()),
        RangeType::Key(Regex::new("last_update").unwrap()),
    ];
    v = print_on_specified_ranges(v, &stack);
    println!("{}", serde_json::to_string_pretty(&v).unwrap());
    assert_eq!(v["connectors"][0]["stability"]["last_update"], "a");

    let mut v: Value = serde_json::from_str(some_json).unwrap();
    let stack = vec![
        RangeType::Array(ArrayRange { begin: 0, end: 0 }),
        RangeType::Key(Regex::new("stability").unwrap()),
    ];
    v = print_on_specified_ranges(v, &stack);
    println!("{}", serde_json::to_string_pretty(&v).unwrap());
    assert_eq!(v["connectors"][0]["stability"]["last_update"], "a");
}
#[test]
fn test_print_6_flexible() {
    let some_json = r#"
    {
      "connectors": [
      "1", "2", "3"
      ]

    }"#;
    let mut v: Value = serde_json::from_str(some_json).unwrap();
    let stack = vec![RangeType::Array(ArrayRange { begin: 0, end: 0 })];
    v = print_on_specified_ranges(v, &stack);
    println!("{}", serde_json::to_string_pretty(&v).unwrap());
    assert_eq!(v["connectors"][0], "1");
}
#[test]
fn test_print_7_flexible() {
    let some_json = r#"
    {
      "connectors": [
        {
          "account_types": [
            "checking"
          ],
          "account_usages": []
        },
        {
          "account_types": true,
          "something_that_should_not": 1
        }
      ]

    }"#;
    let mut v: Value = serde_json::from_str(some_json).unwrap();
    let stack = vec![
        RangeType::Array(ArrayRange { begin: 0, end: 1 }),
        RangeType::Key(Regex::new("account").unwrap()),
    ];
    v = print_on_specified_ranges(v, &stack);
    println!("{}", serde_json::to_string_pretty(&v).unwrap());
    match v["connectors"][1].get("something_that_should_not") {
        Some(_) => assert!(false),
        None => assert!(true),
    };
}
#[test]
fn test_print_3() {
    let some_json = r#"
    { "root":
    {
    "connectors": [
    {
      "siret": null,
      "slug": null,
      "stability": {
        "last_update": "2026-02-07 16:03:02"
      },
      "sync_periodicity": null
    }
    ]
    }
    }"#;
    let mut v: Value = serde_json::from_str(some_json).unwrap();
    let stack = vec![
        RangeType::Key(Regex::new("connectors").unwrap()),
        RangeType::Array(ArrayRange { begin: 0, end: 0 }),
        RangeType::Key(Regex::new("^s").unwrap()),
        RangeType::Key(Regex::new("last").unwrap()),
    ];
    v = print_on_specified_ranges(v, &stack);
    println!("{}", serde_json::to_string_pretty(&v).unwrap());
    match v["root"]["connectors"][0].get("siret") {
        Some(_) => assert!(false),
        None => assert!(true),
    };
}
#[test]
fn test_print_4() {
    let some_json = r#"
    {
      "siret": null
    }"#;
    let mut v: Value = serde_json::from_str(some_json).unwrap();
    let stack = vec![];
    v = print_on_specified_ranges(v, &stack);
    println!("{}", serde_json::to_string_pretty(&v).unwrap());
    match v.get("siret") {
        Some(_) => assert!(true),
        None => assert!(false),
    };
}
#[test]
fn test_print_6() {
    let some_json = r#"
    { "root":
    {
      "commit": [
        {
          "name": "camilo"
        },
        {
          "name": "andres"
        }
        ]
        }
    }"#;
    let mut v: Value = serde_json::from_str(some_json).unwrap();
    let stack = vec![
        RangeType::Key(Regex::new("commit").unwrap()),
        RangeType::Array(ArrayRange { begin: 0, end: 100 }),
        RangeType::Key(Regex::new(".*").unwrap()),
        RangeType::Value(Regex::new("^c").unwrap()),
    ];
    v = print_on_specified_ranges(v, &stack);
    println!("{}", serde_json::to_string_pretty(&v).unwrap());
    assert_eq!(v["root"]["commit"][0]["name"], "camilo");
    assert_eq!(v["root"]["commit"][1], Value::Null);
}
#[test]
fn test_print_7() {
    let some_json = r#"
    {
      "connectors": [
        {
          "auth_mechanism": "credentials",
          "available_auth_mechanisms": [
            "webauth",
            "credentials"
          ],
          "name": {
              "a": "b"
          },
          "uuid": "c64a18a7-e071-487e-8318-f01c76896a29"
        }
      ]
    }
    "#;
    let mut v: Value = serde_json::from_str(some_json).unwrap();
    let stack = vec![
        RangeType::Key(Regex::new("name|uuid|auth").unwrap()),
        RangeType::Key(Regex::new("^a").unwrap()),
    ];
    v = print_on_specified_ranges(v, &stack);
    println!("{}", serde_json::to_string_pretty(&v).unwrap());
    match v["connectors"][0].get("uuid") {
        Some(_) => assert!(false),
        None => assert!(true),
    };
    match v["connectors"][0].get("auth_mechanism") {
        Some(_) => assert!(false),
        None => assert!(true),
    };
    assert_eq!(v["connectors"][0]["name"]["a"], "b");
}
#[test]
fn test_print_8() {
    let some_json = r#"
    {
      "connectors": [
        {
          "auth_mechanism": "credentials",
          "available_auth_mechanisms": [
            "webauth",
            "credentials"
          ],
          "some_array": [ "a", "b", "c"],
          "name": {
              "a": "b"
          },
          "uuid": "c64a18a7-e071-487e-8318-f01c76896a29"
        }
      ]
    }
    "#;
    let mut v: Value = serde_json::from_str(some_json).unwrap();
    let stack = vec![RangeType::Value(Regex::new("credentials").unwrap())];
    v = print_on_specified_ranges(v, &stack);
    println!("{}", serde_json::to_string_pretty(&v).unwrap());
    match v["connectors"][0].get("uuid") {
        Some(_) => assert!(false),
        None => assert!(true),
    };
    match v["connectors"][0].get("some_array") {
        Some(_) => assert!(false),
        None => assert!(true),
    };
    match v["connectors"][0].get("name") {
        Some(_) => assert!(false),
        None => assert!(true),
    };
    assert_eq!(v["connectors"][0]["auth_mechanism"], "credentials");
    assert_eq!(
        v["connectors"][0]["available_auth_mechanisms"][0],
        "credentials"
    );
}
#[test]
fn test_delete_1() {
    let some_json = r#"
    { "root":
    {
      "commit": [
        {
          "name": "camilo"
        },
        {
          "name": "andres"
        }
        ]
        }
    }"#;
    let mut v: Value = serde_json::from_str(some_json).unwrap();
    let stack = vec![
        RangeType::Key(Regex::new("commit").unwrap()),
        RangeType::Array(ArrayRange { begin: 0, end: 0 }),
    ];
    v = delete_on_specified_ranges(v, &stack);
    println!("Result 1:");
    println!("{}", serde_json::to_string_pretty(&v).unwrap());
    assert_eq!(v["root"]["commit"][0]["name"], "andres");

    let mut v: Value = serde_json::from_str(some_json).unwrap();
    let stack = vec![RangeType::Key(Regex::new("doesnt-exists").unwrap())];
    v = delete_on_specified_ranges(v, &stack);
    println!("Result 2:");
    println!("{}", serde_json::to_string_pretty(&v).unwrap());
    assert_eq!(v["root"]["commit"][0]["name"], "camilo");
    assert_eq!(v["root"]["commit"][1]["name"], "andres");
}
#[test]
fn test_delete_3() {
    let some_json = r#"
    { "root":
    {
      "commit": {
        "url": "https://api.github.com/repos/jqlang/jq/git/commits/88b9c4920e643190eebddcf41e373856b5b9292e",
        "verification": {
          "payload": "tree ",
          "reason": "valid",
          "signature": "000",
          "verified": true,
          "verified_at": "2025-12-12T09:54:40Z"
        }
      },
      "node_id": "C_kwDOAE3WVdoAKDg4YjljNDkyMGU2NDMxOTBlZWJkZGNmNDFlMzczODU2YjViOTI5MmU",
      "sha": "88b9c4920e643190eebddcf41e373856b5b9292e"
      }
    }"#;
    let mut v: Value = serde_json::from_str(some_json).unwrap();
    let stack = vec![
        RangeType::Key(Regex::new("commit").unwrap()),
        RangeType::Key(Regex::new("verification").unwrap()),
    ];
    v = delete_on_specified_ranges(v, &stack);
    println!("{}", serde_json::to_string_pretty(&v).unwrap());
    match v["root"]["commit"].get("verification") {
        Some(_) => assert!(false),
        None => assert!(true),
    };
    match v["root"]["commit"].get("url") {
        Some(_) => assert!(true),
        None => assert!(false),
    };
    assert!(v["root"].get("node_id").is_some());
}
#[test]
fn test_delete_4() {
    let some_json = r#"
    {
      "siret": null
    }"#;
    let mut v: Value = serde_json::from_str(some_json).unwrap();
    let stack = vec![];
    v = delete_on_specified_ranges(v, &stack);
    println!("{}", serde_json::to_string_pretty(&v).unwrap());
    match v.get("siret") {
        Some(_) => assert!(false),
        None => assert!(true),
    };
}
#[test]
fn test_delete_5() {
    let some_json = r#"
    {
      "connectors": [
        {
          "a": true,
          "b": true
        },
        {
          "a": true,
          "b": true
        }
      ],
      "total": 1839
    }"#;
    let mut v: Value = serde_json::from_str(some_json).unwrap();
    let stack = vec![
        RangeType::Array(ArrayRange { begin: 0, end: 0 }),
        RangeType::Key(Regex::new("a").unwrap()),
    ];
    v = delete_on_specified_ranges(v, &stack);
    println!("{}", serde_json::to_string_pretty(&v).unwrap());
    match v.get("total") {
        Some(_) => assert!(true),
        None => assert!(false),
    };
    match v["connectors"][0].get("a") {
        Some(_) => assert!(false),
        None => assert!(true),
    };
    match v["connectors"][0].get("b") {
        Some(_) => assert!(true),
        None => assert!(false),
    };
    match v["connectors"][1].get("a") {
        Some(_) => assert!(true),
        None => assert!(false),
    };
    match v["connectors"][1].get("b") {
        Some(_) => assert!(true),
        None => assert!(false),
    };
}
#[test]
fn test_delete_6() {
    let some_json = r#"
    {
  "connectors": [
    {
      "account_types": [
        "checking"
      ]
    }]
    }"#;
    let mut v: Value = serde_json::from_str(some_json).unwrap();
    let stack = vec![RangeType::Key(Regex::new("^a").unwrap())];
    v = delete_on_specified_ranges(v, &stack);
    println!("{}", serde_json::to_string_pretty(&v).unwrap());
    match v["connectors"].get(0) {
        Some(_) => assert!(false),
        None => assert!(true),
    };
}
#[test]
fn test_delete_7() {
    let some_json = r#"
      "connectors"
    "#;
    let mut v: Value = serde_json::from_str(some_json).unwrap();
    let stack = vec![RangeType::Key(Regex::new("doesnt exists").unwrap())];
    v = delete_on_specified_ranges(v, &stack);
    println!("{}", serde_json::to_string_pretty(&v).unwrap());
    assert_eq!(v, "connectors");
}
#[test]
fn test_delete_8() {
    let some_json = r#"
    {
      "connectors": [
        {
          "capabilities": [
            "bank",
            "twofarenew"
          ],
          "categories": []
        },
        {
          "capabilities": [
            "bank",
            "accountcheck"
          ],
          "categories": []
        }
      ]
    }
    "#;
    //#~/powens/learning_projects/jed (next*) » cat connectors.json| jed -e '0,1 p' | jed -e "/connector/.0,2./^ca/p" | jed -e '/connectors/./nada/ d'
    let mut v: Value = serde_json::from_str(some_json).unwrap();
    let stack = vec![
        RangeType::Key(Regex::new("connectors").unwrap()),
        RangeType::Key(Regex::new("doesnt-exists").unwrap()),
    ];
    v = delete_on_specified_ranges(v, &stack);
    println!("{}", serde_json::to_string_pretty(&v).unwrap());
    assert_eq!(v["connectors"][0]["capabilities"][0], "bank");
}
#[test]
fn test_delete_9() {
    let some_json = r#"
    {
      "connectors": [
        {
          "stability": {
            "last_update": "2026-02-07 16:03:02"
          }
        }
      ]
    }

    "#;
    let mut v: Value = serde_json::from_str(some_json).unwrap();
    let stack = vec![RangeType::Key(Regex::new("^s").unwrap())];
    v = delete_on_specified_ranges(v, &stack);
    println!("{}", serde_json::to_string_pretty(&v).unwrap());
    match v.get("connectors") {
        Some(_) => assert!(true),
        None => assert!(false),
    };
    match v["connectors"].get(0) {
        Some(_) => assert!(false),
        None => assert!(true),
    };
}
#[test]
fn test_delete_10() {
    let some_json = r#"
    {
      "connectors": [
        {
          "auth_mechanism": "credentials",
          "available_auth_mechanisms": [
            "webauth",
            "credentials"
          ],
          "some_array": [ "a", "b", "c"],
          "name": {
              "a": "b"
          },
          "uuid": "c64a18a7-e071-487e-8318-f01c76896a29"
        }
      ]
    }
    "#;
    let mut v: Value = serde_json::from_str(some_json).unwrap();
    let stack = vec![RangeType::Value(Regex::new("credentials").unwrap())];
    v = delete_on_specified_ranges(v, &stack);
    println!("{}", serde_json::to_string_pretty(&v).unwrap());
    match v["connectors"][0].get("auth_mechanism") {
        Some(_) => assert!(false),
        None => assert!(true),
    };
    assert_eq!(
        v["connectors"][0]["available_auth_mechanisms"][0],
        "webauth"
    );
    match v["connectors"][0]["available_auth_mechanisms"].get(1) {
        Some(_) => assert!(false),
        None => assert!(true),
    };
}
