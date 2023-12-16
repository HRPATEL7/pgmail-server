#[derive(Default)]
pub struct Mail {
    pub from: String,
    pub to: Vec<String>,
    pub data: String,
    pub sub: String,
}