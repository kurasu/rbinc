use std::fs::File;
use std::io;
use eframe::egui::Ui;
use rfd::MessageLevel::Error;
use binc::document::Document;
use binc::repository::Repository;

pub(crate) struct SimpleApplication {
    pub document: Box<Document>,
    pub view: fn(&mut Ui, &mut SimpleApplication) -> ()
}

pub(crate) fn create_toolbar(app: &mut SimpleApplication, ui: &mut Ui) {
    ui.horizontal(|ui| {
        if ui.button("New").clicked() {
            app.document = Box::from(new_document());
        }
        if ui.button("Open").clicked() {
            let result = open_document();
            if let Ok(Some(result)) = result {
                app.document = Box::from(result);
            } else { show_error(result, "Failed to open document"); }
        }
        if ui.button("Save").clicked() {
            save_document(&app.document);
        }
    });
}

pub (crate) fn show_error<T>(result: io::Result<T>, description: &str) {
    if let Err(error) = result {
        let text = format!("{}\n\n{}", description.to_string(), error.to_string());
        rfd::MessageDialog::new().set_level(Error).set_title("Error").set_description(text).show();
    }
}

pub (crate) fn open_document() -> io::Result<Option<Document>> {
    let path = rfd::FileDialog::new().add_filter("BINC files", &["binc"]).pick_file();

    if let Some(path) = path {
        let mut file = File::open(path)?;
        let document = Document::read(&mut file)?;
        return Ok(Some(document));
    }

    Ok(None)
}

pub (crate) fn save_document(document: &Document) -> io::Result<bool> {
    let path = rfd::FileDialog::new().add_filter("BINC files", &["binc"]).save_file();

    if let Some(path) = path {
        let mut file = File::create(path)?;
        document.write(&mut file)?;
        return Ok(true);
    }
    Ok(false)
}

pub (crate) fn new_document() -> Document {
    Document::new(Repository::new())
}