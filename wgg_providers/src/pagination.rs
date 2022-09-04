use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, PartialEq, Eq, Debug)]
pub struct OffsetPagination<T> {
    pub items: Vec<T>,
    pub total_items: usize,
    pub offset: u32,
}
