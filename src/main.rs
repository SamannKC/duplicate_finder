use walkdir::{DirEntry, WalkDir};
use sha2::{Digest, Sha256};
use std::fs::File;
use std::io;
use std::path::Path;

#[derive(Debug)]
struct FileInfo {
    size: u64,
    path: std::path::PathBuf,
}

#[derive(Debug)]
struct SimilarFiles {
    size: u64,
    paths: Vec<std::path::PathBuf>,
}

#[derive(Debug)]
struct HashedFile<'a>{
    path: &'a std::path::PathBuf,
    hash: String,
}


fn main() -> Result<(), Box<dyn std::error::Error>>{
    let mut files: Vec<FileInfo> = Vec::new();
    let directory = "/home/ado/Documents/";
    let walker = WalkDir::new(directory).into_iter();
    for entry in walker.filter_entry(|e| !is_hidden(e)){
        let entry = entry?;
        if entry.file_type().is_file(){
            let metadata = entry.metadata()?;
            let size = metadata.len();
            let file_info = FileInfo {
                size: size,
                path: entry.path().to_path_buf(),
            };
            files.push(file_info);
        }
    }
    let mut similar: Vec<SimilarFiles> = Vec::new();
    for file in &files{
        match similar.iter_mut().find(|g| g.size == file.size){
            Some(group) => {
                group.paths.push(file.path.clone());
            }
            None => {
                similar.push(SimilarFiles {
                    size: file.size,
                    paths: vec![file.path.clone()],
                });
            }
        }
    }
    for groups in &similar{
        if groups.paths.len() > 1{
            let mut hashed_files: Vec<HashedFile> = Vec::new();
            
            for path in &groups.paths{
                match to_sha256(path){
                    Ok(hash) => hashed_files.push(HashedFile{
                        path,
                        hash,
                    }),
                    Err(e) => eprintln!("Error calculating hash: {}", e),
                }
            }

            let mut used = vec![false; hashed_files.len()];

            for i in 0..hashed_files.len() {
                if used[i] {
                    continue;
                }

                let mut group = vec![hashed_files[i].path];

                for j in (i + 1)..hashed_files.len() {
                    if !used[j] && hashed_files[i].hash == hashed_files[j].hash {
                        group.push(hashed_files[j].path);
                        used[j] = true;
                    }
                }

                if group.len() > 1 {
                    println!("\nDuplicate group:");
                    for p in group {
                        println!("{}", p.display());
                    }
                }
            }
        }
    }
    Ok(())
}

fn is_hidden(entry: &DirEntry) -> bool {
    entry.file_name()
        .to_str()
        .map(|s| s.starts_with("."))
        .unwrap_or(false)
}

fn to_sha256<P: AsRef<Path>>(path: P) -> io::Result<String>{
    let mut file = File::open(path)?;

    let mut hasher = Sha256::new();

    io::copy(&mut file, &mut hasher)?;

    let hash_bytes = hasher.finalize();

    Ok(hex::encode(hash_bytes))
}
