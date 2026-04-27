use crate::grammar::{ArrayRange, RangeType};
use regex::Regex;
use serde_json::{Map, Number, Value};

/// Performs a substitution on the keys of the JSON recursively.
pub fn substitute_keys(v: Value, replace_regex: &Regex, replace_with: &String) -> Value {
    match v {
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
    }
}
pub fn substitute_values(v: Value, search_regexp: &Regex, replace_with: &String) -> Value {
    match v {
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
            if old_null == old_null_replaced {
                return Value::Null;
            } else {
                if let Ok(int) = &old_null_replaced.parse::<i128>() {
                    return Value::Number(Number::from_i128(*int).unwrap());
                }
                if let Ok(float) = &old_null_replaced.parse::<f64>() {
                    return Value::Number(Number::from_f64(*float).unwrap());
                }
                if let Ok(new_bool) = &old_null_replaced.parse::<bool>() {
                    return Value::Bool(*new_bool);
                }
            }
            Value::String(old_null_replaced)
        }
        Value::Bool(v) => {
            let old_bool = v.to_string();
            let old_bool_replaced = search_regexp
                .replace_all(&old_bool, replace_with)
                .into_owned();
            if old_bool == old_bool_replaced {
                return Value::Bool(v);
            } else if let Ok(new_bool) = &old_bool_replaced.parse::<bool>() {
                return Value::Bool(*new_bool);
            }
            Value::String(old_bool_replaced)
        }
        Value::Number(v) => {
            let old_number = v.to_string();
            let old_number_replaced = search_regexp
                .replace_all(&old_number, replace_with)
                .into_owned();
            if old_number == old_number_replaced {
                return Value::Number(v);
            } else {
                if let Ok(int) = &old_number_replaced.parse::<i128>() {
                    return Value::Number(Number::from_i128(*int).unwrap());
                }
                if let Ok(float) = &old_number_replaced.parse::<f64>() {
                    return Value::Number(Number::from_f64(*float).unwrap());
                }
            }
            Value::String(old_number_replaced)
        }
    }
}

/// This function performs the substitution only in the values that match the filter "stack"
pub fn print_on_specified_ranges(v: Value, stack: &[RangeType]) -> Value {
    fn operate_on_object(
        map: Map<String, Value>,
        re: Regex,
        stack: &[RangeType],
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
                        stack,
                        false,
                        false,
                        &OperateOnCallbacks {
                            operate_on_object: &operate_on_object,
                            operate_on_array: &operate_on_array,
                            operate_on_string: &operate_on_string,
                        },
                    );
                    match &new_v {
                        Value::Array(array) => {
                            if !array.is_empty() {
                                new_map.insert(k.clone(), new_v.clone());
                            }
                        }
                        Value::Object(object) => {
                            if !object.is_empty() {
                                new_map.insert(k.clone(), new_v.clone());
                            }
                        } // _ => new_map.insert(k.clone(), new_v),
                        Value::Null => {}
                        _ => {
                            new_map.insert(k.clone(), new_v.clone());
                        }
                    }
                }
            }
        }
        if !new_map.is_empty() {
            return serde_json::Value::Object(new_map);
        }
        serde_json::Value::Null
    }
    fn operate_on_array(vec: Vec<Value>, array_range: ArrayRange) -> Value {
        let mut new_vec: Vec<Value> = Vec::new();
        for (i, val) in vec.iter().enumerate() {
            if i >= array_range.begin && i <= array_range.end {
                new_vec.push(val.clone());
            }
        }
        serde_json::Value::Array(new_vec)
    }
    fn operate_on_string(input: String, re: Regex) -> Value {
        if re.find(&input).is_some() {
            serde_json::Value::String(input)
        } else {
            serde_json::Value::Null
        }
    }
    let callbacks = OperateOnCallbacks {
        operate_on_object: &operate_on_object,
        operate_on_array: &operate_on_array,
        operate_on_string: &operate_on_string,
    };
    apply_on_range(v, stack, false, false, &callbacks)
}

pub fn delete_on_specified_ranges(v: Value, stack: &[RangeType]) -> Value {
    if stack.is_empty() {
        return Value::Null;
    }
    fn operate_on_object(
        map: Map<String, Value>,
        re: Regex,
        stack: &[RangeType],
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
                        stack,
                        false,
                        true,
                        &OperateOnCallbacks {
                            operate_on_object: &operate_on_object,
                            operate_on_array: &operate_on_array,
                            operate_on_string: &operate_on_string,
                        },
                    );
                    match &new_v {
                        Value::Array(_) => {
                            // Allows empty arrays to be returned
                            new_map.insert(k.clone(), new_v.clone());
                        }
                        Value::Object(object) => {
                            if !object.is_empty() {
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
                    //     stack,
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
        if !new_map.is_empty() {
            return serde_json::Value::Object(new_map);
        }
        serde_json::Value::Null
    }
    fn operate_on_array(vec: Vec<Value>, array_range: ArrayRange) -> Value {
        let mut new_vec: Vec<Value> = Vec::new();
        for (i, val) in vec.iter().enumerate() {
            if i < array_range.begin || i > array_range.end {
                new_vec.push(val.clone());
            }
        }
        serde_json::Value::Array(new_vec)
    }
    fn operate_on_string(input: String, re: Regex) -> Value {
        if re.find(&input).is_some() {
            serde_json::Value::Null
        } else {
            serde_json::Value::String(input)
        }
    }
    let callbacks = OperateOnCallbacks {
        operate_on_object: &operate_on_object,
        operate_on_array: &operate_on_array,
        operate_on_string: &operate_on_string,
    };
    apply_on_range(v, stack, false, true, &callbacks)
}

fn keep_or_null(keep_non_matching: bool, value: Value) -> Value {
    if keep_non_matching {
        value
    } else {
        serde_json::Value::Null
    }
}

struct OperateOnCallbacks<'a> {
    operate_on_object: &'a dyn Fn(Map<String, Value>, Regex, &[RangeType], bool) -> Value,
    operate_on_array: &'a dyn Fn(Vec<Value>, ArrayRange) -> Value,
    operate_on_string: &'a dyn Fn(String, Regex) -> Value,
}

fn apply_on_range(
    v: Value,
    stack: &[RangeType],
    stack_anchored: bool,
    keep_non_matching: bool,
    operate_on_callbacks: &OperateOnCallbacks,
) -> Value {
    let Some((stack_head, stack_tail)) = stack.split_first() else {
        return v;
    };
    if stack_tail.is_empty() {
        match v {
            Value::Object(current) => match stack_head {
                RangeType::Key(re) => {
                    return (operate_on_callbacks.operate_on_object)(
                        current,
                        re.to_owned(),
                        stack,
                        stack_anchored,
                    );
                }
                RangeType::Array(_) => {
                    if stack_anchored {
                        keep_or_null(keep_non_matching, serde_json::Value::Object(current))
                    } else {
                        let mut new_map: Map<String, Value> = Map::new();
                        for (k, v) in &current {
                            let new_v = apply_on_range(
                                v.clone(),
                                stack,
                                false,
                                keep_non_matching,
                                &operate_on_callbacks,
                            );
                            if new_v != Value::Null {
                                new_map.insert(k.clone(), new_v);
                            }
                        }
                        if !new_map.is_empty() {
                            return serde_json::Value::Object(new_map);
                        } else {
                            return serde_json::Value::Null;
                        }
                    }
                }
                RangeType::Value(_) => {
                    if stack_anchored {
                        keep_or_null(keep_non_matching, serde_json::Value::Object(current))
                    } else {
                        let mut new_map: Map<String, Value> = Map::new();
                        for (k, v) in &current {
                            let new_v = apply_on_range(
                                v.clone(),
                                stack,
                                false,
                                keep_non_matching,
                                &operate_on_callbacks,
                            );
                            if new_v != Value::Null {
                                new_map.insert(k.clone(), new_v);
                            }
                        }
                        if !new_map.is_empty() {
                            return serde_json::Value::Object(new_map);
                        } else {
                            return serde_json::Value::Null;
                        }
                    }
                }
            },
            Value::String(v) => match stack_head {
                RangeType::Key(_) => keep_or_null(keep_non_matching, serde_json::Value::String(v)),
                RangeType::Array(_) => return serde_json::Value::Null,
                RangeType::Value(re) => {
                    return (&operate_on_callbacks.operate_on_string)(v, re.to_owned())
                }
            },
            Value::Array(current) => match stack_head {
                RangeType::Key(_) => {
                    if stack_anchored {
                        keep_or_null(keep_non_matching, serde_json::Value::Array(current))
                    } else {
                        let mut result: Vec<Value> = Vec::new();
                        for i in current {
                            let new_v = apply_on_range(
                                i.clone(),
                                stack,
                                stack_anchored,
                                keep_non_matching,
                                &operate_on_callbacks,
                            );
                            match &new_v {
                                Value::Array(array) => {
                                    if !array.is_empty() {
                                        result.push(new_v);
                                    }
                                }
                                Value::Object(object) => {
                                    if !object.is_empty() {
                                        result.push(new_v);
                                    }
                                }
                                Value::Null => {}
                                _ => {
                                    result.push(new_v);
                                }
                            }
                        }
                        return serde_json::Value::Array(result);
                    }
                }
                RangeType::Array(array_range) => {
                    return (&operate_on_callbacks.operate_on_array)(
                        current,
                        array_range.to_owned(),
                    );
                }
                RangeType::Value(_) => {
                    if stack_anchored {
                        keep_or_null(keep_non_matching, serde_json::Value::Array(current))
                    } else {
                        let mut result: Vec<Value> = Vec::new();
                        for i in current {
                            let new_v = apply_on_range(
                                i.clone(),
                                stack,
                                stack_anchored,
                                keep_non_matching,
                                &operate_on_callbacks,
                            );
                            match &new_v {
                                Value::Array(array) => {
                                    if !array.is_empty() {
                                        result.push(new_v);
                                    }
                                }
                                Value::Object(object) => {
                                    if !object.is_empty() {
                                        result.push(new_v);
                                    }
                                }
                                Value::Null => {}
                                _ => {
                                    result.push(new_v);
                                }
                            }
                        }
                        if !result.is_empty() {
                            return serde_json::Value::Array(result);
                        } else {
                            return serde_json::Value::Null;
                        }
                    }
                }
            },
            Value::Null => serde_json::Value::Null,
            Value::Bool(b) => keep_or_null(keep_non_matching, serde_json::Value::Bool(b)),
            Value::Number(n) => keep_or_null(keep_non_matching, serde_json::Value::Number(n)),
        }
    } else {
        match v {
            Value::Object(current) => match stack_head {
                RangeType::Key(re) => {
                    let mut new_map: Map<String, Value> = Map::new();
                    for (k, v) in &current {
                        if stack_anchored {
                            if re.find(k).is_some() {
                                let new_v = apply_on_range(
                                    v.clone(),
                                    stack_tail,
                                    true,
                                    keep_non_matching,
                                    &operate_on_callbacks,
                                );
                                if new_v != Value::Null {
                                    new_map.insert(k.clone(), new_v);
                                }
                            } else if keep_non_matching {
                                new_map.insert(k.clone(), v.clone());
                            }
                        } else if re.find(k).is_some() {
                            let new_v = apply_on_range(
                                v.clone(),
                                stack_tail,
                                true,
                                keep_non_matching,
                                &operate_on_callbacks,
                            );
                            if new_v != Value::Null {
                                new_map.insert(k.clone(), new_v);
                            }
                        } else {
                            let new_v = apply_on_range(
                                v.clone(),
                                stack,
                                false,
                                keep_non_matching,
                                &operate_on_callbacks,
                            );
                            if new_v != Value::Null {
                                new_map.insert(k.clone(), new_v);
                            }
                        }
                    }
                    if !new_map.is_empty() {
                        return serde_json::Value::Object(new_map);
                    } else {
                        return serde_json::Value::Null;
                    }
                }
                RangeType::Array(_) => {
                    if stack_anchored {
                        keep_or_null(keep_non_matching, serde_json::Value::Object(current))
                    } else {
                        let mut new_map: Map<String, Value> = Map::new();
                        for (k, v) in &current {
                            let new_v = apply_on_range(
                                v.clone(),
                                stack,
                                false,
                                keep_non_matching,
                                &operate_on_callbacks,
                            );
                            if new_v != Value::Null {
                                new_map.insert(k.clone(), new_v);
                            }
                        }
                        if !new_map.is_empty() {
                            return serde_json::Value::Object(new_map);
                        } else {
                            return serde_json::Value::Null;
                        }
                    }
                }
                RangeType::Value(_) => {
                    keep_or_null(keep_non_matching, serde_json::Value::Object(current))
                }
            },
            Value::String(s) => keep_or_null(keep_non_matching, serde_json::Value::String(s)),
            Value::Array(v) => match stack_head {
                RangeType::Key(_) => {
                    if stack_anchored {
                        keep_or_null(keep_non_matching, serde_json::Value::Array(v))
                    } else {
                        let mut new_vec: Vec<Value> = Vec::new();
                        for val in &v {
                            let new_v = apply_on_range(
                                val.clone(),
                                stack,
                                false,
                                keep_non_matching,
                                &operate_on_callbacks,
                            );
                            if new_v != serde_json::Value::Null {
                                new_vec.push(new_v)
                            }
                        }
                        if !new_vec.is_empty() {
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
                                stack_tail,
                                true,
                                keep_non_matching,
                                &operate_on_callbacks,
                            ));
                        } else if keep_non_matching && {
                            i < array_range.begin || i > array_range.end
                        } {
                            new_vec.push(val.clone());
                        }
                    }
                    return serde_json::Value::Array(new_vec);
                }
                RangeType::Value(_) => keep_or_null(keep_non_matching, serde_json::Value::Array(v)),
            },
            Value::Null => serde_json::Value::Null,
            Value::Bool(b) => keep_or_null(keep_non_matching, serde_json::Value::Bool(b)),
            Value::Number(n) => keep_or_null(keep_non_matching, serde_json::Value::Number(n)),
        }
    }
}

/// This function performs the substitution only in the values that match the filter "stack"
pub fn substitute_values_on_specified_ranges(
    v: Value,
    stack: &[RangeType],
    old_regexp: &Regex,
    replace_with: &str,
) -> Value {
    fn operate_on_object(
        map: Map<String, Value>,
        re: Regex,
        stack: &[RangeType],
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
            Value::Object(new_map)
        } else {
            let mut new_map = Map::new();
            for (k, v) in map {
                if re.find(&k).is_some() {
                    // mal, se puede rompe con doble llave?
                    new_map.insert(k, substitute_values(v, &old_regexp, &replace_with));
                } else {
                    let new_v = apply_on_range(
                        v.clone(),
                        stack,
                        false,
                        true,
                        &OperateOnCallbacks {
                            operate_on_object: &|map, re, stack, stack_anchored| {
                                operate_on_object(
                                    map,
                                    re,
                                    stack,
                                    stack_anchored,
                                    old_regexp.clone(),
                                    replace_with.clone(),
                                )
                            },
                            operate_on_array: &|vec, array_range| {
                                operate_on_array(
                                    vec,
                                    array_range,
                                    old_regexp.clone(),
                                    replace_with.clone(),
                                )
                            },
                            operate_on_string: &|s, _re| Value::String(s), // strings aren't a range target here
                        },
                    );
                    if new_v != serde_json::Value::Null {
                        new_map.insert(k, substitute_values(v, &old_regexp, &replace_with));
                    }
                }
            }
            Value::Object(new_map)
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
        Value::String(string)
    }

    apply_on_range(
        v,
        stack,
        false,
        true, // keep non-matching nodes (substitute keeps the whole doc)
        &OperateOnCallbacks {
            operate_on_object: &|map, re, stack, stack_anchored| {
                operate_on_object(
                    map,
                    re,
                    stack,
                    stack_anchored,
                    old_regexp.clone(),
                    replace_with.to_owned(),
                )
            },
            operate_on_array: &|vec, array_range| {
                operate_on_array(
                    vec,
                    array_range,
                    old_regexp.clone(),
                    replace_with.to_owned(),
                )
            },
            operate_on_string: &|s, _re| {
                operate_on_string(s, _re, old_regexp.clone(), replace_with.to_owned())
            },
        },
    )
}

pub fn substitute_keys_on_specified_ranges(
    v: Value,
    stack: &[RangeType],
    replace_regex: &Regex,
    replace_with: &String,
) -> Value {
    apply_on_range(
        v,
        stack,
        false,
        true, // keep non-matching nodes (substitute keeps the whole doc)
        &OperateOnCallbacks {
            operate_on_object: &|map, re, _stack, _stack_anchored| {
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
            operate_on_array: &|vec, array_range| {
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
            operate_on_string: &|s, _re| Value::String(s), // strings aren't a range target here
        },
    )
}
