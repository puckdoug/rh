#[derive(Debug)]
pub struct Tool {
    pub name: String,
    pub description: String,
    pub parameters: Vec<String>,
    pub command: String,
}
