#[derive(Debug, Clone)]
pub struct Lockfile {
    pub process: String,
    pub pid: usize,
    pub port: usize,
    pub password: String,
    pub protocol: String,
}