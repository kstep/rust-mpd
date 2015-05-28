pub enum Term {
    Any,
    File,
    Base,
    LastMod,
    Tag(String)
}

pub struct Clause(Term, String);

pub struct Query {
    clauses: Vec<Clause>,
    window: Option<(u32, Option<u32>)>,
}

