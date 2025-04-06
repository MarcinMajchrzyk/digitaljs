pub struct Link {
    pub from:   LinkTarget,
    pub to:     LinkTarget
}

#[derive(Clone)]
pub struct LinkTarget {
    pub id:     String,
    pub port:   String,
    pub magnet: String,
}