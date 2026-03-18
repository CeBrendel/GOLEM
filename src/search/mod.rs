
pub mod traits;
pub mod dummy_search;

use crate::board::traits::Move;

#[derive(Clone)]
pub struct SearchInfo<M: Move> {
    pub r#move: Option<M>,
    pub principal_variation_line: Option<Vec<M>>
}
