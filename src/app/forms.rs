use serde::{Deserialize, Serialize};

//struct for form that contains id and amount of a product
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ProductForm {
    pub(crate) product_code: String,
    pub(crate) amount: f32,
}
