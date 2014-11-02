#![feature(macro_rules)]

extern crate git;

use std::default::Default;
use std::os::args;

use git::Error;
use git::status;
use git::status::Flags;

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

fn error(e: Error) {
    println!("{}", e);
}

fn usage() {
    println!("Usage: {} (status|push) <path>", args()[0]);
}

const NEW_FLAGS: Flags = Flags(status::IndexNew as u32 |
                               status::WorkDirNew as u32);

const UPDATED_FLAGS: Flags = Flags(status::IndexModified as u32 |
                                   status::WorkDirModified as u32);

const REMOVED_FLAGS: Flags = Flags(status::IndexDeleted as u32 |
                                   status::WorkDirDeleted as u32);

fn status(path: &Path) -> Result<(), Error> {
    let repo = try!(git::Repository::open(path));
    let list = try!(repo.status(&Default::default()));

    let mut new = vec![];
    let mut updated = vec![];
    let mut removed = vec![];

    for entry in list.iter() {
        let (path, status) = (entry.new_path(), entry.status());

        if status.any(NEW_FLAGS) {
            new.push(path);
        } else if status.any(UPDATED_FLAGS) {
            updated.push(path);
        } else if status.any(REMOVED_FLAGS) {
            removed.push(path);
        }
    }

    print("New", &new);
    print("Updated", &updated);
    print("Deleted", &removed);

    Ok(())
}

fn push(_path: &Path) -> Result<(), Error> {
    Ok(())
}

fn print(title: &str, paths: &Vec<Path>) {
    if paths.is_empty() {
        return;
    }

    println!("{}:", title);

    for path in paths.iter() {
        println!("* {}", path.display());
    }

    println!("");
}
