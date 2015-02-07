#![feature(core, env, io, os, path)]

extern crate curl;
extern crate git;
extern crate "rustc-serialize" as rustc_serialize;
extern crate time;

use git::Result as GitResult;
use std::default::Default;
use std::old_io::IoResult;
use std::env::args;

use description::Description;

mod description;

fn main() {
    macro_rules! ok(
        ($result:expr) => (
            if let Err(e) = $result {
                error(e);
                std::os::set_exit_status(1);
                return;
            }
        );
    );

    let mut args = args();

    args.next();

    let command = match args.next() {
        Some(command) => command.into_string().ok().unwrap(),
        _ => {
            usage();
            return;
        }
    };

    let path = match args.next() {
        Some(path) => Path::new(path.into_string().ok().unwrap()),
        _ => {
            usage();
            return;
        }
    };

    match &command[] {
        "status" => ok!(status(&mut std::old_io::stdio::stdout(), &path)),
        "push" => ok!(push(&path)),
        _ => usage(),
    }
}

fn error<T: std::fmt::Debug>(e: T) {
    println!("{:?}", e);
}

fn usage() {
    println!("Usage: {} (status|push) <path>",
        args().next().unwrap().into_string().ok().unwrap());
}

fn status<T: Writer>(writer: &mut T, path: &Path) -> IoResult<()> {
    use std::old_io::MemWriter;
    use std::old_io::{IoError, OtherIoError};

    macro_rules! display(
        ($writer:expr, $title:expr, $paths:expr) => {
            if !$paths.is_empty() {
                try!(display($writer, $title, &$paths));
            }
        };
    );

    let (new, updated, removed) = match summarize(path) {
        Ok(result) => result,
        Err(error) => return Err(IoError {
            kind: OtherIoError,
            desc: "cannot check the status of the repository",
            detail: Some(format!("{:?}", error)),
        }),
    };

    let mut buffer = MemWriter::new();

    display!(&mut buffer, "New", new);
    display!(&mut buffer, "Updated", updated);
    display!(&mut buffer, "Deleted", removed);

    let data = buffer.into_inner();

    if data.len() > 0 {
        try!(writeln!(writer, "### {}", timestamp()));
        try!(writeln!(writer, ""));
        try!(writer.write_all(&data[]));
    }

    Ok(())
}

fn push(path: &Path) -> GitResult<()> {
    let mut git = try!(git::open(path));

    if try!(git.status()).len() > 0 {
        try!(git.add_all());
        try!(git.commit("Synchronized with the official repository"));
    }

    try!(git.push());

    Ok(())
}

fn summarize(dir: &Path) -> GitResult<(Vec<Path>, Vec<Path>, Vec<Path>)> {
    use git::status::{Flag, Flags};

    macro_rules! equal(
        ($one:expr, $two:expr) => ($one.dir_path() == $two.dir_path());
    );

    macro_rules! find(
        ($vector:expr, $element:expr) => (
            $vector.iter().find(|p| equal!(p, $element))
        );
    );

    const NEW: Flags = Flags(Flag::IndexNew as u32 | Flag::WorkDirNew as u32);
    const UPDATED: Flags = Flags(Flag::IndexModified as u32 | Flag::WorkDirModified as u32);
    const REMOVED: Flags = Flags(Flag::IndexDeleted as u32 | Flag::WorkDirDeleted as u32);

    let repo = try!(git::Repository::open(dir));
    let list = try!(repo.status(&Default::default()));

    let mut new = vec![];
    let mut updated = vec![];
    let mut removed = vec![];

    fn push(vector: &mut Vec<Path>, path: &Path) {
        if find!(vector, path).is_none() {
            vector.push(path.clone());
        }
    }

    for entry in list.iter() {
        let status = entry.status();
        let path = dir.join(entry.new_path());

        if status.has_any(NEW) {
            push(&mut new, &path);
        } else if status.has_any(UPDATED) {
            push(&mut updated, &path);
        } else if status.has_any(REMOVED) {
            push(&mut removed, &path);
        }
    }

    let new = new.into_iter().filter(|path| {
        if find!(removed, path).is_some() {
            push(&mut updated, path);
            false
        } else {
            true
        }
    }).collect::<Vec<_>>();

    let removed = removed.into_iter().filter(|path| {
        (find!(updated, path).is_none())
    }).collect::<Vec<_>>();

    Ok((new, updated, removed))
}

fn display<T: Writer>(writer: &mut T, title: &str, paths: &Vec<Path>) -> IoResult<()> {
    let lines = paths.iter().by_ref()
                     .map(|path| format(path))
                     .filter(|line| line.is_some())
                     .map(|line| line.unwrap())
                     .collect::<Vec<_>>();

    let len = lines.len();
    if len == 0 {
        return Ok(());
    }

    try!(writeln!(writer, "{}:", title));
    try!(writeln!(writer, ""));
    for (i, line) in lines.iter().enumerate() {
        if i + 1 == len {
            try!(writeln!(writer, "* {}.", line));
        } else if i + 2 == len {
            if len == 2 {
                try!(writeln!(writer, "* {} and", line));
            } else {
                try!(writeln!(writer, "* {}, and", line));
            }
        } else {
            try!(writeln!(writer, "* {},", line));
        }
    }
    try!(writeln!(writer, ""));

    Ok(())
}

fn format(path: &Path) -> Option<String> {
    let mut line = String::new();

    let desc = Description::load(path);

    match desc.name {
        Some(ref name) => line.push_str(&name[]),
        None => return None,
    }

    match desc.url {
        Some(ref url) => {
            line.insert(0, '[');
            line.push_str("](");
            line.push_str(&url[]);
            line.push_str(")");
        },
        None => {},
    }

    match desc.designer {
        Some(ref designer) => {
            line.push_str(" by ");
            line.push_str(&designer[]);
        },
        None => {},
    }

    Some(line)
}

fn timestamp() -> String {
    let time = time::now();

    const MONTHS: [&'static str; 12] = ["January", "February", "March", "April",
                                        "May", "June", "July", "August", "September",
                                        "October", "November", "December"];

    format!("{} {}, {}", MONTHS[time.tm_mon as usize], time.tm_mday, 1900 + time.tm_year)
}
