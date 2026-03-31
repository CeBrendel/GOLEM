

pub trait Bool {
    type Negation: Bool;
    const VALUE: bool;
}

pub struct True;

pub struct False;

impl Bool for True {
    type Negation = False;
    const VALUE: bool = true;
}

impl Bool for False {
    type Negation = True;
    const VALUE: bool = false;
}

pub trait Optimizer {
    type Enemy: Optimizer;
    const IS_MAXIMIZER: bool;
}

pub struct Maximizer;

pub struct Minimizer;

impl Optimizer for Maximizer {
    type Enemy = Minimizer;
    const IS_MAXIMIZER: bool = true;
}

impl Optimizer for Minimizer {
    type Enemy = Maximizer;
    const IS_MAXIMIZER: bool = false;
}