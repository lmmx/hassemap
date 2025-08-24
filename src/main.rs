use anyhow::Result;
mod hasse_map;
use hasse_map::Poset;
use std::io::{self, Read};
use serde_json::Value;

fn keys_in_order(v: Value) -> Vec<String> {
    match v {
        Value::Object(map) => map.keys().cloned().collect(),
        _ => panic!("row must be JSON object"),
    }
}

fn main() -> Result<()> {
    // Read all stdin (NDJSON or array-of-objects)
    let mut buf = String::new();
    io::stdin().read_to_string(&mut buf)?;
    let s = buf.trim();

    let rows: Vec<Vec<String>> = if s.starts_with('[') {
        let vals: Vec<Value> = serde_json::from_str(s)?;
        vals.into_iter().map(keys_in_order).collect()
    } else {
        buf.lines()
            .filter(|l| !l.trim().is_empty())
            .map(|l| {
                let v: Value = serde_json::from_str(l).expect("invalid JSON line");
                keys_in_order(v)
            })
            .collect()
    };

    let p = Poset::from_rows(&rows);
    println!("Topological order: {:?}", p.topo_one());
    println!("Hasse edges:");
    for (u, vs) in p.succ.iter().enumerate() {
        if !vs.is_empty() {
            let from = &p.keys[u];
            let tos: Vec<_> = vs.iter().map(|&j| p.keys[j].clone()).collect();
            println!("  {} -> {:?}", from, tos);
        }
    }
    println!("Ambiguities:");
    for (i, js) in p.amb.iter().enumerate() {
        if !js.is_empty() {
            let i_key = &p.keys[i];
            let j_keys: Vec<_> = js.iter().map(|&j| p.keys[j].clone()).collect();
            println!("  {} ? {:?}", i_key, j_keys);
        }
    }
    Ok(())
}
