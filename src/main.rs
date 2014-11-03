#![feature(macro_rules)]

extern crate git;
extern crate hyper;
extern crate serialize;

use git::Error;
use std::default::Default;
use std::os::args;

use description::Description;

mod description;

fn main() {
    macro_rules! ok(
        ($result:expr) => (
            match $result {
                Err(e) => {
                    error(e);
                    return;
                },
                _ => {},
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
        "status" => ok!(status(&path)),
        "push" => ok!(push(&path)),
        _ => usage(),
    }
}

fn status(path: &Path) -> Result<(), Error> {
    #![allow(unused_assignments)]

    macro_rules! print(
        ($title:expr, $paths:expr, $sep:expr) => {
            if !$paths.is_empty() {
                if $sep {
                    println!("");
                }
                $sep = print($title, &$paths);
            }
        };
    )

    let (new, updated, removed) = try!(check(path));
    let mut sep = false;

    print!("New", new, sep);
    print!("Updated", updated, sep);
    print!("Deleted", removed, sep);

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

    const NEW: Flags = Flags(status::IndexNew as u32 | status::WorkDirNew as u32);
    const UPDATED: Flags = Flags(status::IndexModified as u32 | status::WorkDirModified as u32);
    const REMOVED: Flags = Flags(status::IndexDeleted as u32 | status::WorkDirDeleted as u32);

    let repo = try!(git::Repository::open(dir));
    let list = try!(repo.status(&Default::default()));

    let mut new = vec![];
    let mut updated = vec![];
    let mut removed = vec![];

    fn push(vec: &mut Vec<Path>, path: &Path) {
        if vec.iter().find(|&p| p == path).is_none() {
            vec.push(path.clone());
        }
    }

    for entry in list.iter() {
        let status = entry.status();
        let path = dir.join(entry.new_path()).dir_path();

        if status.any(NEW) {
            push(&mut new, &path);
        } else if status.any(UPDATED) {
            push(&mut updated, &path);
        } else if status.any(REMOVED) {
            push(&mut removed, &path);
        }
    }

    let new = new.into_iter().filter(|path| {
        if removed.iter().find(|&p| p == path).is_some() {
            push(&mut updated, path);
            false
        } else {
            true
        }
    }).collect::<Vec<_>>();

    let removed = removed.into_iter().filter(|path| {
        updated.iter().find(|&p| p == path).is_none()
    }).collect::<Vec<_>>();

    Ok((new, updated, removed))
}

fn print(title: &str, paths: &Vec<Path>) -> bool {
    let lines = paths.iter().by_ref()
                     .map(|path| format(path))
                     .filter(|line| line.is_some())
                     .map(|line| line.unwrap())
                     .collect::<Vec<_>>();

    let len = paths.len();
    if len == 0 {
        return false;
    }

    println!("{}:", title);

    for (i, line) in lines.iter().enumerate() {
        if i + 1 == len {
            println!(" * {}.", line);
        } else if i + 2 == len {
            println!(" * {} and", line);
        } else {
            println!(" * {},", line);
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
