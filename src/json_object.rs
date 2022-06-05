use crate::renderer;

#[derive(Debug)]
pub enum JSONObject {
    Object{children: Vec<(JSONObject, JSONObject)>},
    Float{value: f32},
    Integer{value: i64},
    String{value: String},
    Array{values: Vec<JSONObject>},
    Bool{value: bool},
    Null,
}

impl JSONObject {
    pub fn render(&self) -> String {
        renderer::render_indent(&self, 0, true)
    }
}