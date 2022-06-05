use crate::json_object::JSONObject;

pub fn render_indent(object: &JSONObject, indent: u32, own_line: bool) -> String {
    let yes_padding = " ".repeat(indent as usize);
    let no_padding = "".to_string();
    let padding = if own_line {
        yes_padding.clone()
    } else {
        no_padding
    };
    let mut result: String = String::new();
    match object {
        JSONObject::Object { children } => {
            result.push_str("{\n");
            for (i, (key, value)) in children.iter().enumerate() {
                result.push_str(&render_indent(key, indent + 1, true));
                result.push_str(": ");
                result.push_str(&render_indent(value, indent + 1, false));
                if i != children.len() - 1 {
                    result.push_str(", \n");
                }
            }
            result.push('\n');
            result.push_str(&yes_padding);
            result.push('}');
        }
        JSONObject::Float { value } => {
            result.push_str(&padding);
            result.push_str(&format!("{}", value));
        }
        JSONObject::Integer { value } => {
            result.push_str(&padding);
            result.push_str(&format!("{}", value));
        }
        JSONObject::String { value } => {
            result.push_str(&padding);
            result.push('"');
            for c in value.chars() {
                match c {
                    '"' => result.push_str("\\\""),
                    '\t' => result.push_str("\\t"),
                    '\n' => result.push_str("\\n"),
                    '\r' => result.push_str("\\r"),
                    '\\' => result.push_str("\\\\"),
                    _ => {
                        // Need to export Unicode codes if the character is not ASCII
                        if !c.is_ascii() {
                            result.push_str(&format!("\\u{:04x}", c as u32));
                        } else {
                            result.push(c);
                        }
                    }
                }
            }
            result.push('"');
        }
        JSONObject::Array { values } => {
            result.push_str(&padding);
            result.push('[');
            for (i, e) in values.iter().enumerate() {
                result.push_str(&render_indent(e, indent, false));
                if i != values.len() - 1 {
                    result.push_str(", ");
                }
            }
            result.push(']');
        }
        JSONObject::Bool { value } => {
            result.push_str(if *value { "true" } else { "false" });
        }
        JSONObject::Null => result.push_str("null"),
    }
    result
}
