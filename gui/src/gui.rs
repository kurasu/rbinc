use std::fs::File;
use std::io;
use eframe::egui::Ui;
use rfd::MessageLevel::Error;
use binc::document::Document;
use binc::repository::Repository;
use uuid::Uuid;

pub struct SimpleApplication {
    pub document: Box<Document>,
    pub roots: Vec<Uuid>,
}

impl SimpleApplication {
    pub fn new() -> SimpleApplication {
        SimpleApplication {
            document: Box::from(new_document()),
            roots: Vec::new(),
        }
    }

    pub fn set_document(&mut self, document: Document) {
        self.document = Box::from(document);
        self.roots = self.document.find_roots();
    }
}

pub fn create_toolbar(app: &mut SimpleApplication, ui: &mut Ui) {
    ui.horizontal(|ui| {
        if ui.button("New").clicked() {
            app.set_document(new_document());
        }
        if ui.button("Open").clicked() {
            let result = open_document();
            if let Ok(Some(result)) = result {
                app.set_document(result);
            } else { show_error(result, "Failed to open document"); }
        }
        if ui.button("Save").clicked() {
            save_document(&app.document);
        }
    });
}

pub fn show_error<T>(result: io::Result<T>, description: &str) {
    if let Err(error) = result {
        let text = format!("{}\n\n{}", description.to_string(), error.to_string());
        rfd::MessageDialog::new().set_level(Error).set_title("Error").set_description(text).show();
    }
}

pub fn open_document() -> io::Result<Option<Document>> {
    let path = rfd::FileDialog::new().add_filter("BINC files", &["binc"]).pick_file();

    if let Some(path) = path {
        let mut file = File::open(path)?;
        let document = Document::read(&mut file)?;
        return Ok(Some(document));
    }

    Ok(None)
}

pub fn save_document(document: &Document) -> io::Result<bool> {
    let path = rfd::FileDialog::new().add_filter("BINC files", &["binc"]).save_file();

    if let Some(path) = path {
        let mut file = File::create(path)?;
        document.write(&mut file)?;
        return Ok(true);
    }
    Ok(false)
}

pub fn new_document() -> Document {
    Document::new(Repository::new())
}