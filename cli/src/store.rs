use std::{fs, io};
use std::fs::OpenOptions;
use binc::repository::Repository;

pub struct Store {
    root_dir: String,
}

impl Store {
    pub fn new(root: &str) -> Store {
        Store { root_dir: root.to_string() }
    }

    fn translate_path(&self, path: &str) -> String {
        self.root_dir.clone() + "/" + &path
    }

    pub fn list_files(&self, path: String) -> io::Result<Vec<String>>{
        let entries = fs::read_dir(self.translate_path(&path))?;

        let filenames: Vec<String> = entries
            .filter_map(|entry| {
                entry.ok().and_then(|e| e.file_name().into_string().ok())
            })
            .collect();

        Ok(filenames)
    }

    pub fn create_file(&self, path: String) -> io::Result<()> {
        let path = self.translate_path(&path);

        match OpenOptions::new().create_new(true).write(true).open(path) {
            Ok(mut f) =>
                {
                    Repository::new().write(&mut f)
                },
            Err(e) => Err(e)
        }
    }

    pub fn get_file_data(&self, from_revision: u64, path: String) -> io::Result<(u64, u64, Vec<u8>)> {
        let repo = Repository::read(&mut fs::File::open(self.translate_path(&path))?)?;
        let to_revision = repo.revisions.len() as u64;

        if from_revision > to_revision {
            return Err(io::Error::new(io::ErrorKind::InvalidInput, "Revision out of range"));
        }

        let mut data = vec![0; 0];

        let mut index = 0;
        for rev in &repo.revisions {
            if index >= from_revision {
                rev.write(&mut data)?;
            }
            index += 1;
        }

        Ok((from_revision, to_revision, data))
    }
}