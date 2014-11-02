#![feature(macro_rules)]

extern crate git;

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

fn error(e: Error) {
    println!("{}", e);
}

fn usage() {
    println!("Usage: {} (status|push) <path>", args()[0]);
}

fn status(_path: &Path) -> Result<(), Error> {
    Ok(())
}

fn push(_path: &Path) -> Result<(), Error> {
    Ok(())
}
