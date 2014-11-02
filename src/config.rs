use std::io::{IoError, IoResult};
use serialize::json;

#[deriving(Decodable)]
pub struct Config  {
    pub path: String,
}

impl Config {
    pub fn load(path: &Path) -> IoResult<Config> {
        use std::io::{File, OtherIoError};
        match json::decode(try!(try!(File::open(path)).read_to_string()).as_slice()) {
            Ok(config) => Ok(config),
            Err(error) => Err(IoError {
                kind: OtherIoError,
                desc: "cannot parse the configuration file",
                detail: Some(error.to_string()),
            }),
        }
    }
}

#[cfg(test)]
mod test {
    use super::Config;

    #[test]
    fn load() {
        let config = Config::load(&find_fixture("config.json")).unwrap();
    }

    fn find_fixture(name: &str) -> Path {
        use std::io::fs::PathExtensions;
        let path = Path::new("tests").join_many(["fixtures", name]);
        assert!(path.exists());
        path
    }
}
