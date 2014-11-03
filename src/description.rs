use serialize::json;

pub struct Description {
    pub name: Option<String>,
    pub designer: Option<String>,
    pub url: Option<String>,
}

#[deriving(Decodable)]
struct MetaData {
    name: String,
    designer: String,
}

impl Description {
    #[inline]
    fn new() -> Description {
        Description {
            name: None,
            designer: None,
            url: None,
        }
    }

    pub fn load(path: &Path) -> Description {
        let mut desc = Description::new();

        Description::populate_by_metadata(&mut desc, path);
        Description::populate_by_inference(&mut desc, path);
        Description::populate_by_guessing(&mut desc, path);

        desc
    }

    fn populate_by_metadata(desc: &mut Description, path: &Path) {
        use std::io::fs;
        use std::io::fs::PathExtensions;

        macro_rules! ok(
            ($result:expr) => {
                match $result {
                    Ok(result) => result,
                    Err(_) => return,
                }
            };
        )

        let path = path.dir_path().join("METADATA.json");

        if !path.exists() {
            return;
        }

        let content = ok!(ok!(fs::File::open(&path)).read_to_string());

        match json::decode::<MetaData>(content.as_slice()) {
            Ok(metadata) => {
                if !metadata.name.is_empty() {
                    desc.name = Some(metadata.name);
                }
                if !metadata.designer.is_empty() {
                    desc.designer = Some(metadata.designer);
                }
            },
            Err(_) => {},
        }
    }

    fn populate_by_inference(desc: &mut Description, path: &Path) {
        if desc.name.is_some() {
            return;
        }

        match path.extension() {
            Some(b"ttf") | Some(b"TTF") => {},
            _ => return,
        }

        let path = path.with_extension("");
        let blob = match path.filename_str() {
            Some(blob) => blob,
            _ => return,
        };

        let blob = match blob.split('-').next() {
            Some(blob) => blob,
            _ => return,
        };

        let mut name = String::new();
        for (i, c) in blob.char_indices() {
            match c {
                'A'...'Z' if i > 0 => {
                    name.push(' ');
                    name.push(c);
                },
                _ => {
                    name.push(c);
                },
            }
        }

        if !name.is_empty() {
            desc.name = Some(name);
        }
    }

    fn populate_by_guessing(desc: &mut Description, _: &Path) {
        let chunks = match desc.name {
            Some(ref name) => name.as_slice().split(' ').collect::<Vec<_>>(),
            None => return,
        };

        for count in range(1, chunks.len() + 1).rev() {
            let mut name = String::new();
            for &chunk in chunks.iter().take(count) {
                if !name.is_empty() {
                    name.push('+');
                }
                name.push_str(chunk);
            }

            let url = format!("https://www.google.com/fonts/specimen/{}", name);
            if ping(url.as_slice()) {
                desc.url = Some(url);
                break;
            }
        }
    }
}

fn ping(url: &str) -> bool {
    use curl::http;
    match http::handle().head(url).exec() {
        Ok(response) => response.get_code() == 200,
        Err(_) => false,
    }
}
