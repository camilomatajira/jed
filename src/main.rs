use clap::Parser as ClapParser;
use colored_json::to_colored_json_auto;
use pest::Parser;
use pest_derive::Parser;
use regex::Regex;
use serde_json::{Map, Number, Value};
use anyhow::{Context, Result};

#[derive(Parser)]
#[grammar = "grammar.pest"]
struct SedParser;
use std::fs;
use std::io::Read;
/// Example: Options and flags
#[derive(ClapParser)]
struct Cli {
    #[clap(short, long, action)]
    expression: String,
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
    Value(Regex),
}
enum JedCommand {
    Substitute(SubstituteParams),
    SubstituteKeys(SubstituteParams),
    Print,
    Delete,
    Other(String),
}
struct SubstituteParams {
    pattern: Regex,
    replacement: String,
    flags: String,
}


fn main() -> Result<()> {
    // Restore default SIGPIPE handling
    unsafe {
        libc::signal(libc::SIGPIPE, libc::SIG_DFL);
    }

    let cli = Cli::parse();
    let mut file_contents = String::from("");

    match cli.input_file {
        Some(ref input_file) => {
            file_contents = std::fs::read_to_string(&input_file).with_context(|| format!("Could not read file"))?;
            file_contents = file_contents.parse::<String>()?;
        }
        None => {
            std::io::stdin().read_to_string(&mut file_contents)?;
        }
    }

    let input = &cli.expression;
    let mut v: Value = serde_json::from_str(&file_contents).with_context(|| format!("Could not parse file into JSON"))?;
    let (stack, command) = parse_grammar(input)?;

    match command {
        JedCommand::Substitute(params) => {
            let pattern = params.pattern;
            let replacement = params.replacement;
            if stack.len() > 0 {
                v = substitute_values_on_specified_ranges(v, stack, &pattern, &replacement);
            } else {
                v = substitute_values(v, &pattern, &replacement);
            }
        }
        JedCommand::SubstituteKeys(params) => {
            let pattern = params.pattern;
            let replacement = params.replacement;
            if stack.len() > 0 {
                v = substitute_keys_on_specified_ranges(v, stack, &pattern, &replacement);
            } else {
                v = substitute_keys(v, &pattern, &replacement);
            }
        }
        JedCommand::Print => {
            v = print_on_specified_ranges(v, stack);
        }
        JedCommand::Delete => {
            v = delete_on_specified_ranges(v, stack);
        }
        JedCommand::Other(_) => {
            println!("Only substitute command is supported for now");
            std::process::exit(1);
        }
    }

    println!("{}", to_colored_json_auto(&v).context("Failed to colorize JSON output")?);
    Ok(())
}

fn parse_grammar(input: &String) -> Result<(Vec<RangeType>, JedCommand)> {
    let mut stack = Vec::new();
    let parsed = SedParser::parse(Rule::substitute, &input).with_context(|| format!("Parsing the jed command failed: {input}"))?;
    let mut pattern = Regex::new("")?;
    let mut replacement = String::from("");
    let mut flags = String::from("");
    let mut sed_command = ' ';
    for pair in parsed.into_iter().next().context("Parsing the jed command failed")?.into_inner() {
        match pair.as_rule() {
            Rule::range_regex => {
                for inner_pair in pair.into_inner() {
                    match inner_pair.as_rule() {
                        Rule::key_range_regex => {
                            stack.push(RangeType::Key(
                                Regex::new(inner_pair.as_str().trim_matches('/')).context("Parsing the regex expression failed")?,
                            ));
                        }
                        Rule::array_range_regex => {
                            let mut begin = 0;
                            let mut end = 0;
                            for ip in inner_pair.into_inner() {
                                match ip.as_rule() {
                                    Rule::array_range_regex_begin => {
                                        begin = ip.as_str().parse::<usize>()?;
                                    }
                                    Rule::array_range_regex_end => {
                                        end = ip.as_str().parse::<usize>()?;
                                    }
                                    _ => (),
                                }
                            }

                            stack.push(RangeType::Array(ArrayRange { begin, end }));
                        }
                        Rule::value_range_regex => {
                            stack.push(RangeType::Value(
                                Regex::new(inner_pair.as_str().trim_matches('/')).context("Parsing the regex expression failed")?,
                            ));
                        }
                        _ => (),
                    }
                }
            }
            Rule::sed_command => {
                sed_command = pair.as_str().chars().next().context("Failed to parse the Jed command")?;
            }
            Rule::pattern => pattern = Regex::new(pair.as_str()).context("Parsing the search pattern failed")?,
            Rule::replacement => replacement = pair.as_str().to_string(),
            Rule::flags => flags = pair.as_str().to_string(),
            _ => {}
        }
    }

    if sed_command == 's' {
        return Ok((
            stack,
            JedCommand::Substitute(SubstituteParams {
                pattern,
                replacement,
                flags,
            }),
        ));
    }
    if sed_command == 'S' {
        return Ok((
            stack,
            JedCommand::SubstituteKeys(SubstituteParams {
                pattern,
                replacement,
                flags,
            }),
        ));
    }
    if sed_command == 'p' {
        return Ok((stack, JedCommand::Print));
    }
    if sed_command == 'd' {
        return Ok((stack, JedCommand::Delete));
    }
    return Ok((stack, JedCommand::Other(String::from("temporary"))));
}

/// Performs a substitution on the keys of the JSON recursively.
fn substitute_keys(v: Value, replace_regex: &Regex, replace_with: &String) -> Value {
    let v = match v {
        Value::Object(old_map) => {
            let mut new_map: Map<String, Value> = Map::new();
            for (k, v) in old_map {
                let new_key = replace_regex.replace_all(&k, replace_with).into_owned();
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

/// This function performs the substitution only in the values that match the filter "stack"
fn print_on_specified_ranges(v: Value, stack: Vec<RangeType>) -> Value {
    fn operate_on_object(
        map: Map<String, Value>,
        re: Regex,
        stack: Vec<RangeType>,
        stack_anchored: bool,
    ) -> Value {
        let mut new_map: Map<String, Value> = Map::new();
        // It was already popped before
        if stack_anchored {
            for (k, v) in map {
                if re.find(&k).is_some() {
                    new_map.insert(k.clone(), v.clone());
                }
            }
        } else {
            for (k, v) in map {
                if re.find(&k).is_some() {
                    new_map.insert(k.clone(), v.clone());
                } else {
                    let new_v = apply_on_range(
                        v.clone(),
                        stack.clone(),
                        false,
                        false,
                        &operate_on_object,
                        &operate_on_array,
                        &operate_on_string,
                    );
                    match &new_v {
                        Value::Array(array) => {
                            if array.len() > 0 {
                                new_map.insert(k.clone(), new_v.clone());
                            }
                        }
                        Value::Object(object) => {
                            if object.len() > 0 {
                                new_map.insert(k.clone(), new_v.clone());
                            }
                        } // _ => new_map.insert(k.clone(), new_v),
                        Value::Null => {}
                        _ => {
                            new_map.insert(k.clone(), new_v.clone());
                        }
                    }
                    // if new_v != Value:: {
                    //     new_map.insert(k.clone(), new_v);
                    // }
                    // if new_v != Value::Null {
                    //     new_map.insert(k.clone(), new_v);
                    // }
                }
            }
        }
        if new_map.len() > 0 {
            return serde_json::Value::Object(new_map);
        }
        return serde_json::Value::Null;
    }
    fn operate_on_array(vec: Vec<Value>, array_range: ArrayRange) -> Value {
        let mut new_vec: Vec<Value> = Vec::new();
        for (i, val) in vec.iter().enumerate() {
            if i >= array_range.begin && i <= array_range.end {
                new_vec.push(val.clone());
            }
        }
        return serde_json::Value::Array(new_vec);
    }
    fn operate_on_string(input: String, re: Regex) -> Value {
        if re.find(&input).is_some() {
            return serde_json::Value::String(input);
        } else {
            return serde_json::Value::Null;
        }
    }
    apply_on_range(
        v,
        stack,
        false,
        false,
        &operate_on_object,
        &operate_on_array,
        &operate_on_string,
    )
}
fn delete_on_specified_ranges(v: Value, stack: Vec<RangeType>) -> Value {
    if stack.len() == 0 {
        return Value::Null;
    }
    fn operate_on_object(
        map: Map<String, Value>,
        re: Regex,
        stack: Vec<RangeType>,
        stack_anchored: bool,
    ) -> Value {
        let mut new_map: Map<String, Value> = Map::new();
        // It was already popped before
        if stack_anchored {
            for (k, v) in map {
                if re.find(&k).is_none() {
                    new_map.insert(k.clone(), v.clone());
                }
            }
        } else {
            for (k, v) in map {
                if re.find(&k).is_none() {
                    let new_v = apply_on_range(
                        v.clone(),
                        stack.clone(),
                        false,
                        true,
                        &operate_on_object,
                        &operate_on_array,
                        &operate_on_string,
                    );
                    match &new_v {
                        Value::Array(array) => {
                            // Allows empty arrays to be returned
                            new_map.insert(k.clone(), new_v.clone());
                        }
                        Value::Object(object) => {
                            if object.len() > 0 {
                                new_map.insert(k.clone(), new_v.clone());
                            }
                        }
                        Value::Null => {}
                        _ => {
                            new_map.insert(k.clone(), new_v.clone());
                        }
                    }
                } else {
                    // return Value::Null;
                    // let new_v = apply_on_range(
                    //     v.clone(),
                    //     stack.clone(),
                    //     true,
                    //     false,
                    //     &operate_on_object,
                    //     &operate_on_array,
                    //     &operate_on_string,
                    // );
                    // match &new_v {
                    //     Value::Array(array) => {
                    //         if array.len() > 0 {
                    //             new_map.insert(k.clone(), new_v.clone());
                    //         }
                    //     }
                    //     Value::Object(object) => {
                    //         if object.len() > 0 {
                    //             new_map.insert(k.clone(), new_v.clone());
                    //         }
                    //     }
                    //     Value::Null => {}
                    //     _ => {
                    //         new_map.insert(k.clone(), new_v.clone());
                    //     }
                    // }
                }
            }
        }
        if new_map.len() > 0 {
            return serde_json::Value::Object(new_map);
        }
        return serde_json::Value::Null;
    }
    fn operate_on_array(vec: Vec<Value>, array_range: ArrayRange) -> Value {
        let mut new_vec: Vec<Value> = Vec::new();
        for (i, val) in vec.iter().enumerate() {
            if i < array_range.begin || i > array_range.end {
                new_vec.push(val.clone());
            }
        }
        return serde_json::Value::Array(new_vec);
    }
    fn operate_on_string(input: String, re: Regex) -> Value {
        if re.find(&input).is_some() {
            return serde_json::Value::Null;
        } else {
            return serde_json::Value::String(input);
        }
    }
    apply_on_range(
        v,
        stack,
        false,
        true,
        &operate_on_object,
        &operate_on_array,
        &operate_on_string,
    )
}

fn apply_on_range(
    v: Value,
    stack: Vec<RangeType>,
    stack_anchored: bool,
    keep_non_matching: bool,
    operate_on_object: &dyn Fn(Map<String, Value>, Regex, Vec<RangeType>, bool) -> Value,
    operate_on_array: &dyn Fn(Vec<Value>, ArrayRange) -> Value,
    operate_on_string: &dyn Fn(String, Regex) -> Value,
) -> Value {
    let mut response = Value::Null;

    match &stack.len() {
        0 => {
            return v;
        }
        1 => {
            response = match v {
                Value::Object(current) => {
                    let mut new_stack = stack.clone();
                    match new_stack.remove(0) {
                        RangeType::Key(re) => {
                            return operate_on_object(current, re, stack, stack_anchored);
                        }
                        RangeType::Array(_) => {
                            if stack_anchored {
                                return if keep_non_matching {
                                    serde_json::Value::Object(current)
                                } else {
                                    serde_json::Value::Null
                                };
                            } else {
                                let mut new_map: Map<String, Value> = Map::new();
                                for (k, v) in &current {
                                    let new_v = apply_on_range(
                                        v.clone(),
                                        stack.clone(),
                                        false,
                                        keep_non_matching,
                                        operate_on_object,
                                        operate_on_array,
                                        operate_on_string,
                                    );
                                    if new_v != Value::Null {
                                        new_map.insert(k.clone(), new_v);
                                    }
                                }
                                if new_map.len() > 0 {
                                    return serde_json::Value::Object(new_map);
                                } else {
                                    return serde_json::Value::Null;
                                }
                            }
                        }
                        RangeType::Value(_) => {
                            if stack_anchored {
                                return if keep_non_matching {
                                    serde_json::Value::Object(current)
                                } else {
                                    serde_json::Value::Null
                                };
                            } else {
                                let mut new_map: Map<String, Value> = Map::new();
                                for (k, v) in &current {
                                    let new_v = apply_on_range(
                                        v.clone(),
                                        stack.clone(),
                                        false,
                                        keep_non_matching,
                                        operate_on_object,
                                        operate_on_array,
                                        operate_on_string,
                                    );
                                    if new_v != Value::Null {
                                        new_map.insert(k.clone(), new_v);
                                    }
                                }
                                if new_map.len() > 0 {
                                    return serde_json::Value::Object(new_map);
                                } else {
                                    return serde_json::Value::Null;
                                }
                            }
                        }
                    }
                }
                Value::String(v) => {
                    let mut new_stack = stack.clone();
                    match new_stack.remove(0) {
                        RangeType::Key(_) => {
                            if keep_non_matching {
                                return serde_json::Value::String(v);
                            }
                            return serde_json::Value::Null;
                        }
                        RangeType::Array(_) => return serde_json::Value::Null,
                        RangeType::Value(re) => return operate_on_string(v, re),
                    }
                }
                Value::Array(current) => {
                    let mut new_stack = stack.clone();
                    match new_stack.remove(0) {
                        RangeType::Key(_) => {
                            if stack_anchored {
                                if keep_non_matching {
                                    return serde_json::Value::Array(current);
                                } else {
                                    return serde_json::Value::Null;
                                }
                            } else {
                                let mut result: Vec<Value> = Vec::new();
                                for i in current {
                                    let new_v = apply_on_range(
                                        i.clone(),
                                        stack.clone(),
                                        stack_anchored,
                                        keep_non_matching,
                                        operate_on_object,
                                        operate_on_array,
                                        operate_on_string,
                                    );
                                    match &new_v {
                                        Value::Array(array) => {
                                            if array.len() > 0 {
                                                result.push(new_v);
                                            }
                                        }
                                        Value::Object(object) => {
                                            if object.len() > 0 {
                                                result.push(new_v);
                                            }
                                        }
                                        Value::Null => {}
                                        _ => {
                                            result.push(new_v);
                                        }
                                    }
                                }
                                // if result.len() > 0 {
                                //     return serde_json::Value::Array(result);
                                // }
                                return serde_json::Value::Array(result);
                                // return serde_json::Value::Null;
                            }
                        }
                        RangeType::Array(array_range) => {
                            return operate_on_array(current, array_range);
                        }
                        RangeType::Value(_) => {
                            if stack_anchored {
                                if keep_non_matching {
                                    return serde_json::Value::Array(current);
                                } else {
                                    return serde_json::Value::Null;
                                }
                            } else {
                                let mut result: Vec<Value> = Vec::new();
                                for i in current {
                                    let new_v = apply_on_range(
                                        i.clone(),
                                        stack.clone(),
                                        stack_anchored,
                                        keep_non_matching,
                                        operate_on_object,
                                        operate_on_array,
                                        operate_on_string,
                                    );
                                    match &new_v {
                                        Value::Array(array) => {
                                            if array.len() > 0 {
                                                result.push(new_v);
                                            }
                                        }
                                        Value::Object(object) => {
                                            if object.len() > 0 {
                                                result.push(new_v);
                                            }
                                        }
                                        Value::Null => {}
                                        _ => {
                                            result.push(new_v);
                                        }
                                    }
                                }
                                if result.len() > 0 {
                                return serde_json::Value::Array(result);
                                } else{
                                return serde_json::Value::Null;
                                }
                            }
                            }
                    }
                }
                Value::Null => serde_json::Value::Null,
                Value::Bool(b) => {
                    return if keep_non_matching {
                        serde_json::Value::Bool(b)
                    } else {
                        serde_json::Value::Null
                    };
                }
                Value::Number(n) => {
                    return if keep_non_matching {
                        serde_json::Value::Number(n)
                    } else {
                        serde_json::Value::Null
                    };
                }
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
                                if stack_anchored {
                                    if re.find(&k).is_some() {
                                        let new_v = apply_on_range(
                                            v.clone(),
                                            new_stack.clone(),
                                            true,
                                            keep_non_matching,
                                            operate_on_object,
                                            operate_on_array,
                                            operate_on_string,
                                        );
                                        if new_v != Value::Null {
                                            new_map.insert(k.clone(), new_v);
                                        }
                                    } else if keep_non_matching {
                                        new_map.insert(k.clone(), v.clone());
                                    }
                                } else {
                                    if re.find(&k).is_some() {
                                        let new_v = apply_on_range(
                                            v.clone(),
                                            new_stack.clone(),
                                            true,
                                            keep_non_matching,
                                            operate_on_object,
                                            operate_on_array,
                                            operate_on_string,
                                        );
                                        if new_v != Value::Null {
                                            new_map.insert(k.clone(), new_v);
                                        }
                                    } else {
                                        let new_v = apply_on_range(
                                            v.clone(),
                                            stack.clone(),
                                            false,
                                            keep_non_matching,
                                            operate_on_object,
                                            operate_on_array,
                                            operate_on_string,
                                        );
                                        if new_v != Value::Null {
                                            new_map.insert(k.clone(), new_v);
                                        }
                                    }
                                }
                            }
                            if new_map.len() > 0 {
                                return serde_json::Value::Object(new_map);
                            } else {
                                return serde_json::Value::Null;
                            }
                        }
                        RangeType::Array(_) => {
                            if stack_anchored {
                                return if keep_non_matching {
                                    serde_json::Value::Object(current)
                                } else {
                                    serde_json::Value::Null
                                };
                            } else {
                                let mut new_map: Map<String, Value> = Map::new();
                                for (k, v) in &current {
                                    let new_v = apply_on_range(
                                        v.clone(),
                                        stack.clone(),
                                        false,
                                        keep_non_matching,
                                        operate_on_object,
                                        operate_on_array,
                                        operate_on_string,
                                    );
                                    if new_v != Value::Null {
                                        new_map.insert(k.clone(), new_v);
                                    }
                                }
                                if new_map.len() > 0 {
                                    return serde_json::Value::Object(new_map);
                                } else {
                                    return serde_json::Value::Null;
                                }
                            }
                        }
                        RangeType::Value(_) => {
                            return if keep_non_matching {
                                serde_json::Value::Object(current)
                            } else {
                                serde_json::Value::Null
                            };
                        }
                    }
                }
                Value::String(s) => {
                    return if keep_non_matching {
                        serde_json::Value::String(s)
                    } else {
                        serde_json::Value::Null
                    };
                }
                Value::Array(v) => {
                    let mut new_stack = stack.clone();
                    match new_stack.remove(0) {
                        RangeType::Key(_) => {
                            if stack_anchored {
                                return if keep_non_matching {
                                    serde_json::Value::Array(v)
                                } else {
                                    serde_json::Value::Null
                                };
                            } else {
                                let mut new_vec: Vec<Value> = Vec::new();
                                for val in &v {
                                    let new_v = apply_on_range(
                                        val.clone(),
                                        stack.clone(),
                                        false,
                                        keep_non_matching,
                                        operate_on_object,
                                        operate_on_array,
                                        operate_on_string,
                                    );
                                    if new_v != serde_json::Value::Null {
                                        new_vec.push(new_v)
                                    }
                                }
                                if new_vec.len() > 0 {
                                    return serde_json::Value::Array(new_vec);
                                }
                                return serde_json::Value::Null;
                            }
                        }
                        RangeType::Array(array_range) => {
                            let mut new_vec: Vec<Value> = Vec::new();
                            for (i, val) in v.iter().enumerate() {
                                if i >= array_range.begin && i <= array_range.end {
                                    new_vec.push(apply_on_range(
                                        val.clone(),
                                        new_stack.clone(),
                                        true,
                                        keep_non_matching,
                                        operate_on_object,
                                        operate_on_array,
                                        operate_on_string,
                                    ));
                                } else if keep_non_matching && {
                                    i < array_range.begin || i > array_range.end
                                } {
                                    new_vec.push(val.clone());
                                }
                            }
                            return serde_json::Value::Array(new_vec);
                        }
                        RangeType::Value(_) => {
                            return if keep_non_matching {
                                serde_json::Value::Array(v)
                            } else {
                                serde_json::Value::Null
                            };
                        }
                    }
                }
                Value::Null => serde_json::Value::Null,
                Value::Bool(b) => {
                    return if keep_non_matching {
                        serde_json::Value::Bool(b)
                    } else {
                        serde_json::Value::Null
                    };
                }
                Value::Number(n) => {
                    return if keep_non_matching {
                        serde_json::Value::Number(n)
                    } else {
                        serde_json::Value::Null
                    };
                }
            };
        }
    };
    return response;
}

/// This function performs the substitution only in the values that match the filter "stack"
fn substitute_values_on_specified_ranges(
    v: Value,
    stack: Vec<RangeType>,
    old_regexp: &Regex,
    replace_with: &String,
) -> Value {
    fn operate_on_object(
        map: Map<String, Value>,
        re: Regex,
        stack: Vec<RangeType>,
        stack_anchored: bool,
        old_regexp: Regex,
        replace_with: String,
    ) -> Value {
        if stack_anchored {
            let mut new_map = Map::new();
            for (k, v) in map {
                if re.find(&k).is_some() {
                    new_map.insert(k, substitute_values(v, &old_regexp, &replace_with));
                } else {
                    new_map.insert(k, v);
                }
            }
            return Value::Object(new_map);
        } else {
            let mut new_map = Map::new();
            for (k, v) in map {
                if re.find(&k).is_some() {
                    // mal, se puede rompe con doble llave?
                    new_map.insert(k, substitute_values(v, &old_regexp, &replace_with));
                } else {
                    let new_v = apply_on_range(
                        v.clone(),
                        stack.clone(),
                        false,
                        true,
                        &|map, re, stack, stack_anchored| {
                            operate_on_object(
                                map,
                                re,
                                stack,
                                stack_anchored,
                                old_regexp.clone(),
                                replace_with.clone(),
                            )
                        },
                        &|vec, array_range| {
                            operate_on_array(
                                vec,
                                array_range,
                                old_regexp.clone(),
                                replace_with.clone(),
                            )
                        },
                        &|s, _re| Value::String(s), // strings aren't a range target here
                    );
                    if new_v != serde_json::Value::Null {
                        new_map.insert(k, substitute_values(v, &old_regexp, &replace_with));
                    }
                }
            }
            return Value::Object(new_map);
        }
    }
    fn operate_on_array(
        vec: Vec<Value>,
        array_range: ArrayRange,
        old_regexp: Regex,
        replace_with: String,
    ) -> Value {
        let new_vec = vec
            .into_iter()
            .enumerate()
            .map(|(i, val)| {
                if i >= array_range.begin && i <= array_range.end {
                    substitute_values(val, &old_regexp, &replace_with)
                } else {
                    val
                }
            })
            .collect();
        Value::Array(new_vec)
    }
    fn operate_on_string(
        string: String,
        value_range: Regex,
        old_regexp: Regex,
        replace_with: String,
    ) -> Value {
        if value_range.is_match(&string) {
            return substitute_values(Value::String(string), &old_regexp, &replace_with);
        }
        return Value::String(string);
    }
    apply_on_range(
        v,
        stack,
        false,
        true, // keep non-matching nodes (substitute keeps the whole doc)
        &|map, re, stack, stack_anchored| {
            operate_on_object(
                map,
                re,
                stack,
                stack_anchored,
                old_regexp.clone(),
                replace_with.clone(),
            )
        },
        &|vec, array_range| {
            operate_on_array(vec, array_range, old_regexp.clone(), replace_with.clone())
        },
        &|s, _re| {
            operate_on_string(s, _re, old_regexp.clone(), replace_with.clone())
        }
    )
}

fn substitute_keys_on_specified_ranges(
    v: Value,
    stack: Vec<RangeType>,
    replace_regex: &Regex,
    replace_with: &String,
) -> Value {
    apply_on_range(
        v,
        stack,
        false,
        true, // keep non-matching nodes (substitute keeps the whole doc)
        &|map, re, stack, stack_anchored| {
            let mut new_map: Map<String, Value> = Map::new();
            for (k, v) in map {
                if re.find(&k).is_some() {
                    let new_key = replace_regex.replace_all(&k, replace_with).into_owned();
                    let new_v = substitute_keys(v, replace_regex, replace_with);
                    new_map.insert(new_key, new_v);
                } else {
                    new_map.insert(k, v);
                }
            }
            Value::Object(new_map)
        },
        &|vec, array_range| {
            let new_vec = vec
                .into_iter()
                .enumerate()
                .map(|(i, val)| {
                    if i >= array_range.begin && i <= array_range.end {
                        substitute_keys(val, replace_regex, replace_with)
                    } else {
                        val
                    }
                })
                .collect();
            Value::Array(new_vec)
        },
        &|s, _re| Value::String(s), // strings aren't a range target here
    )
}

mod tests {
    use super::*;
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
        v = substitute_keys_on_specified_ranges(v, stack, &replace_regex, &String::from("nom"));
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
        v = substitute_keys_on_specified_ranges(v, stack, &replace_regex, &String::from("nom"));
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
        let (stack, command) = parse_grammar(&input).unwrap();
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
        let (stack, command) = parse_grammar(&input).unwrap();
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
        let input = String::from("1,30p");
        let (stack, command) = parse_grammar(&input).unwrap();
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
                assert_eq!(
                    value_regex.as_str(),
                    Regex::new("camilo").unwrap().as_str()
                );
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
        v = substitute_values_on_specified_ranges(v, stack, &old_regex, &new_regex);
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
        v = substitute_values_on_specified_ranges(v, stack, &old_regex, &new_regex);
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
        v = substitute_values_on_specified_ranges(v, stack, &search_regex, &replace_with);
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
        v = substitute_values_on_specified_ranges(v, stack, &search_regex, &replace_with);
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
        v = substitute_values_on_specified_ranges(v, stack, &search_regex, &replace_with);
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
        v = substitute_values_on_specified_ranges(v, stack, &search_regex, &replace_with);
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
        v = substitute_values_on_specified_ranges(v, stack, &search_regex, &replace_with);
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
        let stack = vec![
            RangeType::Value(Regex::new("spanish").unwrap()),
        ];
        let search_regex = Regex::new("credentials").unwrap();
        let replace_with = String::from("credenciales");
        v = substitute_values_on_specified_ranges(v, stack, &search_regex, &replace_with);
        println!("{}", serde_json::to_string_pretty(&v).unwrap());
        assert_eq!(v["connectors"][0]["auth_mechanism"], "credenciales_spanish");
        assert_eq!(v["connectors"][0]["available_auth_mechanisms"][1], "credentials_french");
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
        v = print_on_specified_ranges(v, stack);
        println!("{}", serde_json::to_string_pretty(&v).unwrap());
        assert_eq!(v["root"]["commit"][0]["name"], "camilo");
        assert_eq!(v["root"]["commit"][1]["name"], Value::Null);

        let stack = vec![RangeType::Key(Regex::new("doesnt-exists").unwrap())];
        v = print_on_specified_ranges(v, stack);
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
        v = print_on_specified_ranges(v, stack);
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
        let mut stack = vec![RangeType::Key(Regex::new("account_types").unwrap())];
        v = print_on_specified_ranges(v, stack);
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
        let mut stack = vec![RangeType::Key(Regex::new("account_types").unwrap())];
        v = print_on_specified_ranges(v, stack);
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
        let mut stack = vec![RangeType::Key(
            Regex::new("something that does not exists").unwrap(),
        )];
        v = print_on_specified_ranges(v, stack);
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
        let mut stack = vec![
            RangeType::Key(Regex::new("key1").unwrap()),
            RangeType::Key(Regex::new("key112").unwrap()),
        ];
        v = print_on_specified_ranges(v, stack);
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
        let mut stack = vec![
            RangeType::Key(Regex::new("stability").unwrap()),
            RangeType::Key(Regex::new("last_update").unwrap()),
        ];
        v = print_on_specified_ranges(v, stack);
        println!("{}", serde_json::to_string_pretty(&v).unwrap());
        assert_eq!(v["connectors"][0]["stability"]["last_update"], "a");

        let mut v: Value = serde_json::from_str(some_json).unwrap();
        let mut stack = vec![
            RangeType::Array(ArrayRange { begin: 0, end: 0 }),
            RangeType::Key(Regex::new("stability").unwrap()),
        ];
        v = print_on_specified_ranges(v, stack);
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
        let mut stack = vec![RangeType::Array(ArrayRange { begin: 0, end: 0 })];
        v = print_on_specified_ranges(v, stack);
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
        let mut stack = vec![
            RangeType::Array(ArrayRange { begin: 0, end: 1 }),
            RangeType::Key(Regex::new("account").unwrap()),
        ];
        v = print_on_specified_ranges(v, stack);
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
        v = print_on_specified_ranges(v, stack);
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
        v = print_on_specified_ranges(v, stack);
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
        v = print_on_specified_ranges(v, stack);
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
        v = print_on_specified_ranges(v, stack);
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
        let stack = vec![
            RangeType::Value(Regex::new("credentials").unwrap()),
        ];
        v = print_on_specified_ranges(v, stack);
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
        assert_eq!(v["connectors"][0]["available_auth_mechanisms"][0], "credentials");
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
        v = delete_on_specified_ranges(v, stack);
        println!("Result 1:");
        println!("{}", serde_json::to_string_pretty(&v).unwrap());
        assert_eq!(v["root"]["commit"][0]["name"], "andres");

        let mut v: Value = serde_json::from_str(some_json).unwrap();
        let stack = vec![RangeType::Key(Regex::new("doesnt-exists").unwrap())];
        v = delete_on_specified_ranges(v, stack);
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
        v = delete_on_specified_ranges(v, stack);
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
        v = delete_on_specified_ranges(v, stack);
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
        v = delete_on_specified_ranges(v, stack);
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
        v = delete_on_specified_ranges(v, stack);
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
        v = delete_on_specified_ranges(v, stack);
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
        v = delete_on_specified_ranges(v, stack);
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
        v = delete_on_specified_ranges(v, stack);
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
        let stack = vec![
            RangeType::Value(Regex::new("credentials").unwrap()),
        ];
        v = delete_on_specified_ranges(v, stack);
        println!("{}", serde_json::to_string_pretty(&v).unwrap());
        match v["connectors"][0].get("auth_mechanism") {
            Some(_) => assert!(false),
            None => assert!(true),
        };
        assert_eq!(v["connectors"][0]["available_auth_mechanisms"][0], "webauth");
        match v["connectors"][0]["available_auth_mechanisms"].get(1) {
            Some(_) => assert!(false),
            None => assert!(true),
        };
    }
}
