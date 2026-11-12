use loro::{LoroDoc, LoroMap, LoroValue, ValueOrContainer};
use toml::Value;
use std::collections::HashMap;

fn toml_to_loro_value(value: &Value) -> LoroValue {
    match value {
        Value::Table(t) => {
            let mut map = HashMap::new();
            for (k, v) in t {
                map.insert(k.clone(), toml_to_loro_value(v));
            }
            LoroValue::Map(map.into())
        }
        Value::Array(a) => {
            let mut list = Vec::new();
            for v in a {
                list.push(toml_to_loro_value(v));
            }
            LoroValue::List(list.into())
        }
        Value::String(s) => LoroValue::from(s.clone()),
        Value::Integer(i) => LoroValue::from(*i),
        Value::Float(f) => LoroValue::from(*f),
        Value::Boolean(b) => LoroValue::from(*b),
        Value::Datetime(d) => LoroValue::from(d.to_string()),
    }
}

fn embed_toml(doc: &mut LoroDoc, value: &Value) -> Result<(), Box<dyn std::error::Error>> {
    let root = doc.get_map("root");
    let loro_val = toml_to_loro_value(value);
    root.insert("config", loro_val)?;
    Ok(())
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let toml_content = r#"
        title = "Sheetah Configuration"

        [server]
        port = 8080
        host = "localhost"
        enable_logging = true

        [[database.nodes]]
        id = "node-1"
        role = "primary"

        [[database.nodes]]
        id = "node-2"
        role = "replica"

        [metadata]
        tags = ["production", "rust", "crdt"]
    "#;

    let parsed_toml: Value = toml::from_str(toml_content)?;
    let mut doc = LoroDoc::new();

    println!("--- Embedding TOML into Loro ---");
    embed_toml(&mut doc, &parsed_toml)?;

    println!("\n--- Verification ---");
    let root = doc.get_map("root");
    
    // Check root title (wait, title is inside config)
    if let Some(config_val) = root.get("config") {
        match config_val {
            ValueOrContainer::Value(LoroValue::Map(config)) => {
                println!("Config Title: {:?}", config.get("title"));
                
                if let Some(server_val) = config.get("server") {
                    if let LoroValue::Map(server) = server_val {
                        println!("Server Port: {:?}", server.get("port"));
                        println!("Server Host: {:?}", server.get("host"));
                        println!("Server Logging: {:?}", server.get("enable_logging"));
                    }
                }

                if let Some(database_val) = config.get("database") {
                    if let LoroValue::Map(database) = database_val {
                        if let Some(nodes_val) = database.get("nodes") {
                            if let LoroValue::List(nodes) = nodes_val {
                                println!("Number of database nodes: {}", nodes.len());
                                if let Some(first_node_val) = nodes.get(0) {
                                    println!("First node info: {:?}", first_node_val);
                                }
                            }
                        }
                    }
                }

                if let Some(metadata_val) = config.get("metadata") {
                   if let LoroValue::Map(metadata) = metadata_val {
                       if let Some(tags_val) = metadata.get("tags") {
                           if let LoroValue::List(tags) = tags_val {
                               println!("Metadata tags: {:?}", tags);
                           }
                       }
                   }
                }
            }
            _ => println!("Config is not a map"),
        }
    }

    Ok(())
}
