extern crate boss;

use boss::CSPStreamWorkerPool;
use std::io;
use std::io::BufRead;
use std::process::Command;
use std::str;
use std::thread;

use std::path::PathBuf;
use structopt::StructOpt;

#[derive(Debug, StructOpt)]
#[structopt(name = "pclone", about = "parallel git clone")]
struct Opt {
    /// Directory to clone to
    #[structopt(parse(from_os_str))]
    dir: PathBuf,
}

#[derive(Clone)]
struct Data {
    url: String,
    dir: PathBuf,
}

fn gitclone(d: Data) -> String {
    let fields: Vec<&str> = d.url.split('/').collect();
    let name = fields[fields.len() - 1];
    let target_name = format!("{}/{}", d.dir.to_str().unwrap(), name);
    let output = Command::new("git")
        .arg("clone")
        .arg(&d.url)
        .arg(&target_name)
        .output();
    match output {
        Ok(outres) => {
            if outres.status.success() {
                format!("{}", target_name)
            } else {
                format!("Failed {:?}", str::from_utf8(&outres.stderr).unwrap())
            }
        }
        Err(err) => format!("Failed {}", err.into_inner().unwrap()),
    }
}

fn main() -> io::Result<()> {
    let opt = Opt::from_args();
    let boss = CSPStreamWorkerPool::new(None, Some(4), gitclone);
    let rv = boss.clone();
    let stdin = io::stdin();
    let handle = stdin.lock();
    let urls: Vec<String> = handle.lines().map(|r| r.unwrap()).collect();

    thread::spawn(move || {
        for url in urls {
            let d = Data {
                url: url,
                dir: opt.dir.clone(),
            };
            boss.send_data(d);
        }
        boss.finish();
    });
    // Allows to middle click (B dir0 dir1) and load the directories into acme
    // editor. See sam(1) line :451 (B is a shell-level command that causes
    // an instance of sam running on the same terminal to load the named files.)
    print!("B");
    for r in rv {
        print!(" {}", r)
    }
    Ok(())
}
