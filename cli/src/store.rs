use binc::journal::Journal;
use std::fs::OpenOptions;
use std::io::Write;
use std::{fs, io};

pub struct Store {
    root_dir: String,
}

impl Store {
    pub fn new(root: &str) -> Store {
        Store {
            root_dir: root.to_string(),
        }
    }

    fn translate_path(&self, path: &str) -> String {
        self.root_dir.clone() + "/" + &path
    }

    pub fn list_files(&self, path: String) -> io::Result<Vec<String>> {
        let entries = fs::read_dir(self.translate_path(&path))?;

        let filenames: Vec<String> = entries
            .filter_map(|entry| entry.ok().and_then(|e| e.file_name().into_string().ok()))
            .collect();

        Ok(filenames)
    }

    pub fn create_file(&self, path: String) -> io::Result<()> {
        let path = self.translate_path(&path);

        match OpenOptions::new().create_new(true).write(true).open(path) {
            Ok(mut f) => Journal::new().write(&mut f),
            Err(e) => Err(e),
        }
    }

    pub fn get_file_data(&self, from: u64, path: String) -> io::Result<(u64, u64, Vec<u8>)> {
        let repo = Journal::read(&mut fs::File::open(self.translate_path(&path))?)?;
        let to = repo.operations.len() as u64;

        if from > to {
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                "Revision out of range",
            ));
        }

        let mut data = vec![0; 0];

        let mut index = 0;
        for change in &repo.operations {
            if index >= from {
                change.write(&mut data)?;
            }
            index += 1;
        }

        Ok((from, to, data))
    }

    pub(crate) fn append_file(
        &self,
        from: u64,
        to: u64,
        path: &str,
        data: Vec<u8>,
    ) -> io::Result<()> {
        if from >= to {
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                format!("No changes to append. {}..{}", from, to),
            ));
        }

        let fs_path = self.translate_path(path);
        let repo = Journal::read(&mut fs::File::open(fs_path.clone())?)?;
        if repo.operations.len() as u64 != from {
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                "Revision mismatch",
            ));
        }

        let mut file = OpenOptions::new().append(true).open(fs_path)?;
        file.write(&data)?;

        Ok(())
    }
}
