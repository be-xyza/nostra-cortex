with open("apps/cortex-desktop/src/gateway/server.rs", "r") as f:
    text = f.read()

# we look for: if let Some(has_files) = query.has_files {
idx = text.find("if let Some(has_files) = query.has_files {")
if idx == -1:
    print("Cannot find has_files filter")
    exit(1)

# insert attribute filter
insertion = """
    if let Some(attribute_query) = query.attribute.as_deref() {
        let (target_key, target_val) = match attribute_query.split_once(':') {
            Some((k, v)) => (Some(k.trim().to_ascii_lowercase()), v.trim().to_ascii_lowercase()),
            None => (None, attribute_query.trim().to_ascii_lowercase()),
        };
        rows.retain(|entry| {
            if let Some(attrs) = &entry.projection.attributes {
                if let Some(key) = &target_key {
                    if let Some(val) = attrs.get(*key) {
                        return val.to_ascii_lowercase() == target_val;
                    }
                } else {
                    return attrs.values().any(|val| val.to_ascii_lowercase() == target_val);
                }
            }
            false
        });
    }
"""

text = text[:idx] + insertion + text[idx:]
with open("apps/cortex-desktop/src/gateway/server.rs", "w") as f:
    f.write(text)
print("Added attribute filter!")
