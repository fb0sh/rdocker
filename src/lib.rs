use std::collections::HashMap;
use std::fmt::Display;
use std::io::BufReader;
use std::os::unix::net::UnixStream;
use std::{io, io::prelude::*};

#[derive(Debug)]
pub struct Docker {
    version: String,
    api_version: String,
    os_version: String,
    stream: UnixStream,
}

#[derive(Debug)]
pub struct DockerResult {
    headers: HashMap<String, String>,
    data: serde_json::Result<serde_json::Value>,
}

impl Display for Docker {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "Docker Version: {}\nApi Version: v{}\nOs Version: {}\n",
            self.version, self.api_version, self.os_version
        )
    }
}

impl Display for DockerResult {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut header = String::new();
        for (k, v) in &self.headers {
            header += &format!("{k}: {v}\n");
        }
        let data = self.data.as_ref();
        write!(f, "{header}\n{:?}", data)
    }
}

impl DockerResult {
    fn parse(stream: &mut UnixStream) -> (HashMap<String, String>, String) {
        // recv
        let mut bf = BufReader::new(stream);
        let mut headers = HashMap::new();

        // for http first line
        let mut first_line = String::new();
        bf.read_line(&mut first_line).unwrap();
        let head = first_line.split(" ").collect::<Vec<&str>>();
        headers.insert("http_version".to_string(), head[0].to_string());
        headers.insert("status_code".to_string(), head[1].to_string());
        let mut status = String::new();
        for i in 2..head.len() {
            status += head[i];
            status += " ";
        }
        headers.insert("status".to_string(), status.trim_end().to_string());

        loop {
            let mut line = String::new();
            let size = bf.read_line(&mut line).unwrap();

            if size == 2 {
                // \r\n\r\n
                break;
            }

            let each = line.split(":").collect::<Vec<&str>>();
            let first = each[0].replace("\"", "");
            let second = each[1].trim_start().replace("\"", "").replace("\r\n", "");
            headers.insert(first, second);
        }

        let mut res = String::new();
        if let Some(content_length) = headers.get("Content-Length") {
            if content_length != "0" {
                let _size = bf.read_line(&mut res).unwrap();
            }
        }

        (headers, res)
    }

    fn request(docker: &mut Docker, method: &str, api_end: &str, body: &str) -> Self {
        let req = format!(
            "{} /v{}{} HTTP/1.1\r\nHost: localhost\r\nAccept: */*\r\n\r\n",
            method, docker.api_version, api_end
        );

        docker.stream.write_all(req.as_bytes()).unwrap();

        if body.len() != 0 {
            docker.stream.write_all(body.as_bytes()).unwrap();
        }

        let (headers, res) = DockerResult::parse(&mut docker.stream);

        Self {
            headers: headers,
            data: serde_json::from_str(&res),
        }
    }

    pub fn status_code(&self) -> i32 {
        self.headers["status_code"].parse::<i32>().unwrap()
    }
}

impl Docker {
    pub fn new() -> Result<Self, io::Error> {
        let stream = UnixStream::connect("/var/run/docker.sock")?;
        let mut docker = Self {
            version: String::new(),
            api_version: String::from("1.24"),
            os_version: String::new(),
            stream: stream,
        };

        let dr = docker.get("/version");
        let data = dr.data?;
        docker.version = data["Version"].to_string().replace("\"", "");
        docker.api_version = data["ApiVersion"].to_string().replace("\"", "");
        docker.os_version = data["Os"].to_string().replace("\"", "");

        Ok(docker)
    }

    pub fn head(&mut self, api_end: &str) -> DockerResult {
        DockerResult::request(self, "HEAD", api_end, "")
    }
    pub fn get(&mut self, api_end: &str) -> DockerResult {
        DockerResult::request(self, "GET", api_end, "")
    }
    pub fn post(&mut self, api_end: &str, body: &str) -> DockerResult {
        DockerResult::request(self, "POST", api_end, body)
    }
    pub fn put(&mut self, api_end: &str, body: &str) -> DockerResult {
        DockerResult::request(self, "PUT", api_end, body)
    }
    pub fn delete(&mut self, api_end: &str, body: &str) -> DockerResult {
        DockerResult::request(self, "DELETE", api_end, body)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_new() {
        let mut d = Docker::new().unwrap();
        let p = d.head("/_ping");
        println!("{}", p.status_code());
    }
}
