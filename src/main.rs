use walkdir::WalkDir;
use filetime::FileTime;
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;
use std::time::{Duration, SystemTime};
use std::error::Error;
use std::os::unix::fs::PermissionsExt;

use serde::{Serialize, Deserialize};

#[derive(Clone, Serialize, Deserialize, PartialEq)]
enum FileType {
    File,
    Dir,
    Link,
}

#[derive(Clone, Serialize, Deserialize)]
struct PathData {
    mtime: i64,
    perms: u32,
    size: u64,
    ftype: FileType,
}

#[derive(Clone, Serialize, Deserialize)]
struct DirIndex {
    scantime: u64,
    root: PathBuf,
    contents: HashMap<PathBuf, PathData>,
}

impl PartialEq for PathData {
    fn eq(&self, other: &PathData) -> bool {
        self.mtime == other.mtime && self.perms == other.perms && self.size == other.size && self.ftype == other.ftype
    }
}

impl Eq for PathData {}

fn map_dir(basepath: &PathBuf) -> Result<DirIndex,  Box<Error>> {
    let current_time = SystemTime::now().duration_since(SystemTime::UNIX_EPOCH)?.as_secs();
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
                Err(e) => {}
                Ok(m) => {
                    let mtime = FileTime::from_last_modification_time(&m).seconds();
                    let relpath = path.strip_prefix(basepath.to_str().unwrap_or(""))?.to_path_buf();
                    let ftype = if m.file_type().is_dir() {
                        FileType::Dir
                    }
                    else if m.file_type().is_symlink() {
                        FileType::Link
                    }
                    else {
                        FileType::File
                    };

                    println!("insert {}",relpath.to_path_buf().display());
                    paths.insert(
                        relpath,
                        PathData {
                            mtime: mtime,
                            perms: m.permissions().mode(),
                            size: m.len(),
                            ftype: ftype,
                        },
                    );
                }
            }
        }
    Ok(DirIndex {
        scantime: current_time,
        root: basepath.to_path_buf(),
        contents: paths,
    })
}

fn compare_dirs(dir_a: &mut HashMap<PathBuf, PathData>, dir_b: &mut HashMap<PathBuf, PathData>) {
    let mut dir_b_copy = dir_b.clone();
    for (path, pathdata_a) in dir_a.iter() {
        match dir_b.get(path) {
            Some(pathdata_b) => {
                if pathdata_a == pathdata_b {
                    println!("{} found, identical", path.display());
                }
                else if pathdata_a.mtime > pathdata_b.mtime {
                    println!("{} found, A is newer", path.display());
                }
                else if pathdata_a.mtime < pathdata_b.mtime {
                    println!("{} found, B is newer", path.display());
                }
                else {
                    println!("{} found, different", path.display());
                }
                dir_b_copy.remove(path);
            }
            None => println!("{} is missing from B.", path.display())
        }
    }
    for (path, pathdata_b) in dir_b_copy.iter() {
        match dir_a.get(path) {
            Some(pathdata_a) => {
                println!("{} found in both, strange..", path.display());
            }
            None => println!("{} is missing from A.", path.display())
        }
    }
}

fn main() {
    //let current_time = Instant::now();
    let watch_a = PathBuf::from("/home/henrik/comparedirs/testdir/A");
    let watch_b = PathBuf::from("/home/henrik/comparedirs/testdir/B");


    let mut paths_a = map_dir(&watch_a).unwrap();
    let mut paths_b = map_dir(&watch_b).unwrap();
    let serialized = serde_json::to_string(&paths_a).unwrap();
    println!("serialized = {}", serialized);
    //compare_dirs(&mut paths_a, &mut paths_b);
}
