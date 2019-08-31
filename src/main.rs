extern crate boss;

use boss::CSPStreamWorkerPool;
use std::io;
use std::io::BufRead;
use std::process::Command;
use std::str;
use std::thread;

fn gitclone(url: String) -> String {
    let output = Command::new("git").arg("clone").arg(&url).output();
    match output {
        Ok(outres) => {
            if outres.status.success() {
                // TODO path to the filesystem for quick rightclick
                format!("Cloned repo {}", url)
            } else {
                format!("Failed {:?}", str::from_utf8(&outres.stderr).unwrap())
            }
        }
        Err(err) => format!("Failed {}", err.into_inner().unwrap()),
    }
}

fn main() -> io::Result<()> {
    let boss = CSPStreamWorkerPool::new(None, Some(4), gitclone);
    let rv = boss.clone();
    let stdin = io::stdin();
    let handle = stdin.lock();
    let urls: Vec<String> = handle.lines().map(|r| r.unwrap()).collect();

    thread::spawn(move || {
        for url in urls {
            boss.send_data(url);
        }
        boss.finish();
    });
    for r in rv {
        println!("{}", r)
    }
    Ok(())
}
