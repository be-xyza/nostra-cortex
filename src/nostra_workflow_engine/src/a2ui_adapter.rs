use super::execution::FormField;

/// Converts workflow form fields to A2UI JSON schema
pub fn generate_a2ui_form(fields: &[FormField]) -> serde_json::Value {
    let mut form_fields = Vec::new();

    for field in fields {
        let mut field_spec = serde_json::json!({
            "id": field.name,
            "label": field.label,
            "required": field.required,
        });

        // Map workflow field types to A2UI component types
        match field.field_type.as_str() {
            "Text" => {
                field_spec["componentProperties"] = serde_json::json!({
                    "TextField": {
                        "label": field.label,
                        "value": "",
                        "a11y": {
                            "label": field.label,
                            "required": field.required
                        }
                    }
                });
            }
            "Date" => {
                field_spec["componentProperties"] = serde_json::json!({
                    "DateTimeInput": {
                        "label": field.label,
                        "value": "",
                        "a11y": {
                            "label": field.label,
                            "required": field.required
                        }
                    }
                });
            }
            "Enum" => {
                field_spec["componentProperties"] = serde_json::json!({
                    "MultipleChoice": {
                        "label": field.label,
                        "selections": field.options.clone().unwrap_or_default(),
                        "a11y": {
                            "label": field.label,
                            "required": field.required
                        }
                    }
                });
            }
            "Boolean" => {
                field_spec["componentProperties"] = serde_json::json!({
                    "CheckBox": {
                        "label": field.label,
                        "value": false,
                        "a11y": {
                            "label": field.label,
                            "required": field.required
                        }
                    }
                });
            }
            _ => {
                // Default to text field
                field_spec["componentProperties"] = serde_json::json!({
                    "TextField": {
                        "label": field.label,
                        "value": "",
                        "a11y": {
                            "label": field.label,
                            "required": field.required
                        }
                    }
                });
            }
        }

        form_fields.push(field_spec);
    }

    // Generate A2UI message structure
    serde_json::json!({
        "surfaceUpdate": {
            "components": form_fields,
            "root": "form_root",
        }
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generate_text_field() {
        let fields = vec![FormField {
            name: "title".to_string(),
            label: "Document Title".to_string(),
            field_type: "Text".to_string(),
            required: true,
            options: None,
        }];

        let a2ui = generate_a2ui_form(&fields);
        assert!(
            a2ui["surfaceUpdate"]["components"][0]["componentProperties"]["TextField"].is_object()
        );
    }
}
