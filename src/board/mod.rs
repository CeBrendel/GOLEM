
pub mod dummy_board;
pub mod wrapped_board;


pub trait Move: Clone + Send + Sync + 'static {
    fn as_string(&self) -> String;
    fn from_algebraic(s: &str) -> Self;
}

pub trait Board<M: Move + 'static>: Send + 'static {
    fn put_into_startpos(&mut self);
    fn put_into_fen(&mut self, fen: &str);
    fn make(&mut self, r#move: M);
    fn unmake(&mut self);
    fn visualize(&self);
}