#![feature(macro_rules)]

extern crate git;

use std::default::Default;
use std::os::args;

use git::Error;

macro_rules! ok(
    ($result:expr) => (
        match $result {
            Ok(_) => {},
            Err(e) => {
                error(e);
                return;
            },
        }
    );
)

fn main() {
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
                $sep = true;
                print($title, &$paths);
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

fn check(path: &Path) -> Result<(Vec<Path>, Vec<Path>, Vec<Path>), Error> {
    use git::status;
    use git::status::Flags;

    const NEW: Flags = Flags(status::IndexNew as u32 | status::WorkDirNew as u32);
    const UPDATED: Flags = Flags(status::IndexModified as u32 | status::WorkDirModified as u32);
    const REMOVED: Flags = Flags(status::IndexDeleted as u32 | status::WorkDirDeleted as u32);

    let repo = try!(git::Repository::open(path));
    let list = try!(repo.status(&Default::default()));

    let mut new = vec![];
    let mut updated = vec![];
    let mut removed = vec![];

    for entry in list.iter() {
        let (path, status) = (entry.new_path(), entry.status());

        if status.any(NEW) {
            new.push(path);
        } else if status.any(UPDATED) {
            updated.push(path);
        } else if status.any(REMOVED) {
            removed.push(path);
        }
    }

    Ok((new, updated, removed))
}

fn print(title: &str, paths: &Vec<Path>) {
    println!("{}:", title);
    for path in paths.iter() {
        println!("* {}", format(path));
    }
}

fn format(path: &Path) -> String {
    match path.with_extension("").filename_str() {
        Some(s) => String::from_str(s),
        None => "<unrecognized>".to_string(),
    }
}
