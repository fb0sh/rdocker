mod docker;
pub use docker::Docker;
pub use docker::DockerResult;

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_new() {
        let mut d = Docker::new().unwrap();
        println!("{d}");
        let p = d.head("/_ping");
        println!("{}", p.status_code());
    }
}
