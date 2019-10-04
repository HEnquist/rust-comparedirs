use walkdir::WalkDir;
use filetime::FileTime;
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::mpsc::Sender;
use std::sync::{Arc, Mutex, RwLock};
use std::thread;
use std::time::{Duration, Instant};
use std::error::Error;


struct PathData {
    mtime: i64,
}

fn map_dir(basepath: &PathBuf) -> Result<HashMap<PathBuf, PathData>,  Box<Error>> {
    let mut paths = HashMap::new();
    let depth = usize::max_value();
    for entry in WalkDir::new(basepath.clone())
            .follow_links(false)
            .max_depth(depth)
            .into_iter()
            .filter_map(|e| e.ok())
        {
            let path = entry.path();
            match entry.metadata() {
                Err(e) => {
                    //self.event_tx.send(RawEvent {
                    //    path: Some(path.to_path_buf()),
                    //    op: Err(Error::Io(e.into())),
                    //    cookie: None,
                    //});
                }
                Ok(m) => {
                    let mtime = FileTime::from_last_modification_time(&m).seconds();
                    let relpath = path.strip_prefix(basepath.to_str().unwrap()).unwrap().to_path_buf();
                    println!("insert {}",relpath.to_path_buf().display());
                    paths.insert(
                        relpath,
                        PathData {
                            mtime: mtime,
                        },
                    );
                }
            }
        }
    Ok(paths)
}

fn compare_dirs(dirA: HashMap<PathBuf, PathData>, dirB: HashMap<PathBuf, PathData>) {
    for (path, pathdataA) in &dirA {
        match dirB.get(path) {
            Some(pathdataB) => {
                println!("{} found.", path.display());
                if pathdataA.mtime > pathdataB.mtime {
                    println!("A is newer");
                }
                else if pathdataA.mtime < pathdataB.mtime {
                    println!("B is newer");
                }
                else {
                    println!("same age");
                }
            }
            None => println!("{} is missing.", path.display())
        }
    }
}

fn main() {
    let current_time = Instant::now();
    let watch_a = PathBuf::from("/home/henrik/comparedirs/testdir/A");
    let watch_b = PathBuf::from("/home/henrik/comparedirs/testdir/B");


    let paths_a = map_dir(&watch_a).unwrap();
    let paths_b = map_dir(&watch_b).unwrap();
    compare_dirs(paths_a, paths_b);
}
