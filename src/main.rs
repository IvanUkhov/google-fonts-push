#![feature(if_let, macro_rules)]

extern crate curl;
extern crate git;
extern crate serialize;
extern crate time;

use git::Error;
use std::default::Default;
use std::os::args;

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
    )

    let args = args();

    if args.len() != 3 {
        usage();
        return;
    }

    let (command, path) = (args[1].clone(), Path::new(args[2].clone()));

    match command.as_slice() {
        "status" => ok!(status(&mut std::io::stdio::stdout(), &path)),
        "push" => ok!(push(&path)),
        _ => usage(),
    }
}

fn status<T: Writer>(writer: &mut T, path: &Path) -> Result<(), Error> {
    #![allow(unused_assignments)]

    use std::io::MemWriter;

    macro_rules! ok(
        ($result:expr) => (
            if let Err(_) = $result {
                panic!("cannot write to a buffer");
            }
        );
    )

    macro_rules! display(
        ($writer:expr, $title:expr, $paths:expr, $sep:expr) => {
            if !$paths.is_empty() {
                if $sep {
                    ok!(writeln!($writer, ""));
                }
                $sep = display($writer, $title, &$paths);
            }
        };
    )

    let (new, updated, removed) = try!(check(path));
    let mut sep = false;

    let mut buffer = MemWriter::new();

    display!(&mut buffer, "New", new, sep);
    display!(&mut buffer, "Updated", updated, sep);
    display!(&mut buffer, "Deleted", removed, sep);

    let data = buffer.unwrap();

    if data.len() > 0 {
        ok!(writeln!(writer, "### {}", timestamp()));
        ok!(writeln!(writer, ""));
        ok!(writer.write(data.as_slice()));
    }

    Ok(())
}

fn push(path: &Path) -> Result<(), Error> {
    let mut git = try!(git::open(path));

    try!(git.add_all());
    try!(git.commit("Synchronized with the official repository"));
    try!(git.push());

    Ok(())
}

fn error(e: Error) {
    println!("{}", e);
}

fn usage() {
    println!("Usage: {} (status|push) <path>", args()[0]);
}

fn check(dir: &Path) -> Result<(Vec<Path>, Vec<Path>, Vec<Path>), Error> {
    use git::status;
    use git::status::Flags;

    macro_rules! equal(
        ($one:expr, $two:expr) => ($one.dir_path() == $two.dir_path());
    )

    const NEW: Flags = Flags(status::IndexNew as u32 | status::WorkDirNew as u32);
    const UPDATED: Flags = Flags(status::IndexModified as u32 | status::WorkDirModified as u32);
    const REMOVED: Flags = Flags(status::IndexDeleted as u32 | status::WorkDirDeleted as u32);

    let repo = try!(git::Repository::open(dir));
    let list = try!(repo.status(&Default::default()));

    let mut new = vec![];
    let mut updated = vec![];
    let mut removed = vec![];

    fn push(vec: &mut Vec<Path>, path: &Path) {
        if vec.iter().find(|&p| equal!(p, path)).is_none() {
            vec.push(path.clone());
        }
    }

    for entry in list.iter() {
        let status = entry.status();
        let path = dir.join(entry.new_path());

        if status.any(NEW) {
            push(&mut new, &path);
        } else if status.any(UPDATED) {
            push(&mut updated, &path);
        } else if status.any(REMOVED) {
            push(&mut removed, &path);
        }
    }

    let new = new.into_iter().filter(|path| {
        if removed.iter().find(|&p| equal!(p, path)).is_some() {
            push(&mut updated, path);
            false
        } else {
            true
        }
    }).collect::<Vec<_>>();

    let removed = removed.into_iter().filter(|path| {
        updated.iter().find(|&p| equal!(p, path)).is_none()
    }).collect::<Vec<_>>();

    Ok((new, updated, removed))
}

fn display<T: Writer>(writer: &mut T, title: &str, paths: &Vec<Path>) -> bool {
    #![allow(unused_must_use)]

    macro_rules! ok(
        ($result:expr) => (
            if let Err(_) = $result {
                panic!("cannot write to a buffer");
            }
        );
    )

    let lines = paths.iter().by_ref()
                     .map(|path| format(path))
                     .filter(|line| line.is_some())
                     .map(|line| line.unwrap())
                     .collect::<Vec<_>>();

    let len = lines.len();
    if len == 0 {
        return false;
    }

    ok!(writeln!(writer, "{}:", title));

    for (i, line) in lines.iter().enumerate() {
        if i + 1 == len {
            ok!(writeln!(writer, " * {}.", line));
        } else if i + 2 == len {
            if len == 2 {
                ok!(writeln!(writer, " * {} and", line));
            } else {
                ok!(writeln!(writer, " * {}, and", line));
            }
        } else {
            ok!(writeln!(writer, " * {},", line));
        }
    }

    true
}

fn format(path: &Path) -> Option<String> {
    let mut line = String::new();

    let desc = Description::load(path);

    match desc.name {
        Some(ref name) => line.push_str(name.as_slice()),
        None => return None,
    }

    match desc.url {
        Some(ref url) => {
            line.insert(0, '[');
            line.push_str("](");
            line.push_str(url.as_slice());
            line.push_str(")");
        },
        None => {},
    }

    match desc.designer {
        Some(ref designer) => {
            line.push_str(" by ");
            line.push_str(designer.as_slice());
        },
        None => {},
    }

    Some(line)
}

fn timestamp() -> String {
    let time = time::now();

    const MONTHS: [&'static str, ..12] = ["January", "February", "March", "April",
                                          "May", "June", "July", "August", "September",
                                          "October", "November", "December"];

    format!("{} {}, {}", MONTHS[time.tm_mon as uint], time.tm_mday, 1900 + time.tm_year)
}
