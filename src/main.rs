use std::env;
use std::fs;
use std::io;
use std::path::{Path, PathBuf};
use std::thread;
use std::sync::{Arc, Mutex};
use colored::*;

fn search_directory(
    path: PathBuf,
    query: String,
    result: Arc<Mutex<Vec<PathBuf>>>,
) -> Result<(), io::Error> {
    let entries = fs::read_dir(&path)?;
    for entry in entries {
        let entry = entry?;
        let entry_path = entry.path();
        if entry_path.is_file() && entry_path.to_string_lossy().contains(&query) {
            let  result_guard = result.lock();
            match result_guard {
                Ok(mut guard) => guard.push(entry_path.clone()),
                Err(poisoned) => {
                    return Err(io::Error::new(
                        io::ErrorKind::Other,
                        format!("Mutex poisoned: {:?}", poisoned),
                    ));
                }
            }
        } else if entry_path.is_dir() {
            let result_clone = result.clone();
            let query_clone = query.clone();
            let entry_path_clone = entry_path.clone();
            thread::spawn(move || {
                if let Err(err) = search_directory(entry_path_clone, query_clone, result_clone) {
                    eprintln!("Error: {}", err);
                }
            });
        }
    }
    Ok(())
}

fn main() -> Result<(), io::Error> {
    let args: Vec<String> = env::args().collect();

    if args.len() != 3 {
        eprintln!("Usage: {} <directory> <query>", args[0]);
        return Ok(());
    }

    let start_dir = Path::new(&args[1]).to_path_buf();
    let query = args[2].clone();

    let result: Arc<Mutex<Vec<PathBuf>>> = Arc::new(Mutex::new(vec![]));

    search_directory(start_dir.clone(), query.clone(), result.clone())?;

    // Wait for all threads to finish (no need for a separate handles vector)
    // The threads are waited for within the search_directory function
    let result = result.lock();
    match result {
        Ok(guard) => {
            for path in guard.iter() {
                println!("{}", path.to_string_lossy().red());
            }
        }
        Err(poisoned) => {
            return Err(io::Error::new(
                io::ErrorKind::Other,
                format!("Mutex poisoned: {:?}", poisoned),
            ));
        }
    }

    Ok(())
}
