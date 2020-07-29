use serde::{Deserialize};

#[derive(Debug, Clone)]
pub struct Lockfile {
    pub process: String,
    pub pid: usize,
    pub port: usize,
    pub password: String,
    pub protocol: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct Perk {
    id: u64,
    name: String,
    iconPath: String,
    shortDesc: String,
    longDesc: String,
    tooltip: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct LolPerksPerkPage {
    pub autoModifiedSelections: Vec<i64>,
    pub current: bool,
    pub id: usize,
    pub isActive: bool,
    pub isDeletable: bool,
    pub isEditable: bool,
    pub isValid: bool,
    pub lastModified: i64,
    pub name: String,
    pub order: i64,
    pub primaryStyleId: i64,
    pub selectedPerkIds: Vec<i64>,
    pub subStyleId: i64,
}