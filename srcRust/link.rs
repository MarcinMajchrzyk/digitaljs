pub struct Link {
    pub source: LinkTarget,
    pub target: LinkTarget
}

#[derive(Clone)]
pub struct LinkTarget {
    pub id:     String,
    pub port:   String,
    pub magnet: String,
}