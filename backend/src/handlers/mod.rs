pub mod books;
pub mod tags;
pub mod reading_status;
pub mod users;
pub mod reviews;
pub mod questions;
pub mod answers;
pub mod ai_answers;
pub mod images;

use serde::Serialize;

#[derive(Serialize)]
pub struct PaginatedResponse<T: Serialize> {
    pub data: Vec<T>,
    pub total: i64,
    pub page: i64,
    pub per_page: i64,
}
