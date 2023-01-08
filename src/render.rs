use std::collections::HashMap;

use serde_json::{json, Value};

pub fn filter_fmt2f(val: &Value, args: &HashMap<String, Value>) -> tera::Result<Value> {
    let resutl = if args.get("display_positive").is_some() {
        format!(
            "{:+.2}",
            val.as_f64()
                .ok_or(tera::Error::msg(format!("not float: {}", val)))?
        )
    } else {
        format!(
            "{:.2}",
            val.as_f64()
                .ok_or(tera::Error::msg(format!("not float: {}", val)))?
        )
    };
    Ok(Value::String(resutl))
}

pub fn filter_atof(val: &Value, _args: &HashMap<String, Value>) -> tera::Result<Value> {
    let v = val
        .as_str()
        .ok_or(tera::Error::msg(format!("not str: {}", val)))?
        .parse::<f64>()
        .map_err(|e| tera::Error::msg(format!("cant parse to float: {}, {}", val, e)))?;
    Ok(json!(v))
}

pub fn filter_qoutevolume(val: &Value, args: &HashMap<String, Value>) -> tera::Result<Value> {
    let v = val
        .as_f64()
        .ok_or(tera::Error::msg(format!("not float: {}", val)))?;
    if v > 1e8 {
        return Ok(Value::String(format!(
            "{:.2}{}",
            v / 1e8,
            args.get("e8").unwrap_or(&json!("")).as_str().unwrap()
        )));
    } else if v > 1e4 {
        return Ok(Value::String(format!(
            "{:.2}{}",
            v / 1e4,
            args.get("e4").unwrap_or(&json!("")).as_str().unwrap()
        )));
    } else {
        return Ok(Value::String(format!("{:.2}", v)));
    }
}

pub fn filter_emoji(val: &Value, args: &HashMap<String, Value>) -> tera::Result<Value> {
    let positive = args
        .get("positive")
        .ok_or(tera::Error::msg(format!("need arg positive")))?
        .as_array()
        .unwrap()
        .into_iter()
        .filter_map(|e| e.as_str())
        .collect::<Vec<&str>>();
    let negative = args
        .get("negative")
        .ok_or(tera::Error::msg(format!("need arg negative")))?
        .as_array()
        .unwrap()
        .into_iter()
        .filter_map(|e| e.as_str())
        .collect::<Vec<&str>>();

    let v = val
        .as_f64()
        .ok_or(tera::Error::msg(format!("not float: {}", val)))?;
    let vec = if v > 0.0 { positive } else { negative };
    if vec.is_empty() {
        return Ok(Value::String("".into()));
    }
    let v = v.abs() as usize;
    let mut idx;
    if v < 10 {
        idx = v / 2; // 0 1 2 3 4
    } else {
        idx = 4 + v / 10; // 5 6 ...
    }
    idx = idx.min(vec.len() - 1);

    Ok(Value::String(vec[idx].into()))
}
