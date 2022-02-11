use std::process::Command;
use regex::Regex;
use thiserror::Error;

#[derive(Debug, Clone)]
pub enum WgEntry {
    Client(ClientData),
    Server(ServerData)
}

#[derive(Debug, Clone)]
pub struct ClientData {
    pub interface : String,
    pub public_key : String,
    pub preshared_key : Option<String>,
    pub endpoint : Option<String>,
    pub allowed_ips : String,
    pub latest_handshake : u64,
    pub transfer_rx : i64,
    pub transfer_tx : i64,
    pub persistent_keepalive : u64,
}

#[derive(Debug, Clone)]
pub struct ServerData {
    pub interface : String,
    pub private_key : String,
    pub public_key : String,
    pub listen_port : String,
    pub fwmark : String,
}

pub fn get_dump() -> Vec<WgEntry> {
    let cmd = Command::new("wg")
        .arg("show")
        .arg("all")
        .arg("dump")
        .output().unwrap();

    let output = String::from_utf8(cmd.stdout).unwrap();

    parse_dump(output)
}

pub fn parse_dump(data : String) -> Vec<WgEntry> {
    let mut payload : Vec<WgEntry> = Vec::new();
    for (_, line) in data.lines().enumerate() {
        let re = Regex::new(r"\s+").unwrap();
        let replaced_line = re.replace_all(line, " ").to_string();
        let splits : Vec<String> = replaced_line.split_whitespace().map(|s| s.to_owned()).collect();
        match splits.len() {
            // Server
            5 => {
                let entry = WgEntry::Server(ServerData {
                    interface: splits[0].clone(),
                    private_key: splits[1].clone(),
                    public_key: splits[2].clone(),
                    listen_port: splits[3].clone(),
                    fwmark: splits[4].clone()
                });
                payload.push(entry);
            },
            // Client
            9 => {
                let preshared_key = {
                    if splits[2].eq("(none)") {
                        None
                    } else {
                        Some(splits[2].to_owned())
                    }
                };

                let endpoint = {
                    if splits[3].eq("(none)") {
                        None
                    } else {
                        Some(splits[3].to_owned())
                    }
                };

                let entry = WgEntry::Client(ClientData {
                    interface: splits[0].clone(),
                    public_key: splits[1].clone(),
                    preshared_key,
                    endpoint,
                    allowed_ips: splits[4].clone(),
                    latest_handshake: splits[5].parse::<u64>().unwrap(),
                    transfer_rx: splits[6].parse::<i64>().unwrap(),
                    transfer_tx: splits[7].parse::<i64>().unwrap(),
                    persistent_keepalive: splits[8].parse::<u64>().unwrap()
                });
                payload.push(entry);
            },
            _ => {}
        }
    }

    payload
}


#[derive(Error, Debug)]
pub enum WgError {
    #[error("context `{0}` could not be found")]
    ContextNotFound(String),
    #[error("config error: {0:?}")]
    CustomError(Box<dyn std::error::Error>),
    #[error("config error: {0}")]
    Message(String)
}

#[cfg(test)]
mod tests {
    static WG_SHOW_ALL_DUMP_EXAMPLE : &str = r#"
wg0	bGFib3JlIHN1bnQgb21uaXMgcXVvcyBvZmZpY2lpcw==	cGFyaWF0dXIuIFByb3ZpZGVudCBldCB0ZW1wb3JhIHF1b3M=	31194	off
wg0	QXNodG9uIFNoZXJ5bCBNb3JzZQ==	(none)	10.2.2.68:62299	10.2.98.3/32	1643795801	1204	1900	25
wg0	RXhlcmNpdGF0aW9uZW0gbmVxdWUgZGVzZXJ1bnQ=	(none)	(none)	10.2.98.8/32	0	0	0	25
wg0	dW5kZSBleC4gUXVhcw==	(none)	(none)	10.2.98.6/32	0	0	0	25
wg0	dXQgY29uc2VjdGV0dXIgZXQgYXNwZXJpb3JlcyB1dCBub2Jpcw==	(none)	(none)	10.2.98.7/32	0	0	0	25
    "#;


    #[test]
    fn test_dump_parsing() {
        let entries = super::parse_dump(WG_SHOW_ALL_DUMP_EXAMPLE.to_owned());

        for entry in &entries {
            println!("{:?}", entry);
        }
    }
}