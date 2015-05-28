use std::fmt;

pub enum Term {
    Any,
    File,
    Base,
    LastMod,
    Tag(String)
}

pub struct Clause(pub Term, pub String);

pub struct Query {
    pub clauses: Vec<Clause>,
    pub window: Option<(u32, Option<u32>)>,
}

pub struct Count {
    pub clauses: Option<Vec<Clause>>,
    pub group: Option<String>
}

impl fmt::Display for Term {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            Term::Any => f.write_str("any"),
            Term::File => f.write_str("file"),
            Term::Base => f.write_str("base"),
            Term::LastMod => f.write_str("modified-since"),
            Term::Tag(ref tag) => f.write_str(&*tag)
        }
    }
}

impl fmt::Display for Clause {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{} {}", self.0, self.1)
    }
}

impl fmt::Display for Query {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        for clause in &self.clauses {
            try!(clause.fmt(f));
        }

        match self.window {
            Some((a, Some(b))) => write!(f, " window {}:{}", a, b),
            Some((a, None)) => write!(f, " window {}:", a),
            None => Ok(())
        }
    }
}
