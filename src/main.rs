use eframe::egui;
use std::{
    process::Command,
    sync::{Arc, RwLock},
};

#[cfg(target_os = "linux")]
const COMMAND: &str = "gs";

#[cfg(target_os = "windows")]
const COMMAND: &str = "gswin64";

fn main() -> Result<(), eframe::Error> {
    env_logger::init(); // Log to stderr (if you run with `RUST_LOG=debug`).
    let options = eframe::NativeOptions {
        initial_window_size: Some(egui::vec2(800.0, 440.0)),
        ..Default::default()
    };
    eframe::run_native(
        "PDF compressor",
        options,
        Box::new(|_cc| Box::<MyApp>::default()),
    )
}

#[derive(PartialEq, Clone)]
enum PdfSettings {
    Default,
    Screen,
    Ebook,
    Printer,
    Prepress,
}

impl std::fmt::Display for PdfSettings {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            PdfSettings::Default => write!(f, "default"),
            PdfSettings::Screen => write!(f, "screen"),
            PdfSettings::Ebook => write!(f, "ebook"),
            PdfSettings::Printer => write!(f, "printer"),
            PdfSettings::Prepress => write!(f, "prepress"),
        }
    }
}

struct MyApp {
    picked_path: Option<String>,
    output_path: Option<String>,
    pdf_settings: PdfSettings,
    image_dpi: u16,
    is_processing: Arc<RwLock<bool>>,
    compression_complete: Arc<RwLock<bool>>,
}

impl Default for MyApp {
    fn default() -> Self {
        Self {
            picked_path: None,
            output_path: None,
            pdf_settings: PdfSettings::Ebook,
            image_dpi: 150,
            is_processing: Arc::new(RwLock::new(false)),
            compression_complete: Arc::new(RwLock::new(false)),
        }
    }
}

impl eframe::App for MyApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("PDF compressor");
            ui.add_space(10.0);

            ui.label("Image DPI:");
            ui.add(egui::Slider::new(&mut self.image_dpi, 10..=300).step_by(10.0));

            ui.add_space(10.0);

            ui.label("Select a mode to compress:");
            ui.horizontal(|ui| {
                ui.selectable_value(&mut self.pdf_settings, PdfSettings::Default, "default");
                ui.selectable_value(&mut self.pdf_settings, PdfSettings::Screen, "screen");
                ui.selectable_value(&mut self.pdf_settings, PdfSettings::Ebook, "ebook");
                ui.selectable_value(&mut self.pdf_settings, PdfSettings::Printer, "printer");
                ui.selectable_value(&mut self.pdf_settings, PdfSettings::Prepress, "prepress");
            });

            ui.add_space(10.0);

            if ui.button("Open file…").clicked() {
                if let Some(path) = rfd::FileDialog::new()
                    .add_filter("pdf", &["pdf"])
                    .set_directory(dirs::document_dir().unwrap())
                    .pick_file()
                {
                    let file_name_without_extension = path.file_stem().unwrap().to_str().unwrap();
                    let output_path = path
                        .with_file_name(format!("{file_name_without_extension}_compressed.pdf"));

                    self.picked_path = Some(path.display().to_string());
                    self.output_path = Some(output_path.display().to_string());

                    let mut lock = self.compression_complete.write().unwrap();
                    *lock = false;
                }
            }

            ui.add_space(5.0);
            ui.separator();
            ui.add_space(5.0);

            if let Some(picked_path) = &self.picked_path {
                ui.horizontal(|ui| {
                    ui.label("Picked file:");
                    ui.monospace(picked_path);
                });

                ui.horizontal(|ui| {
                    ui.label("Compressed file output:");
                    ui.monospace(self.output_path.as_ref().unwrap());
                });

                ui.add_space(10.0);

                if *self.is_processing.read().unwrap() {
                    ui.horizontal(|ui| {
                        ui.label("Compressing…");
                        ui.spinner();
                    });
                } else if ui.button("Compress PDF").clicked() {
                    let mut is_processing = self.is_processing.write().unwrap();
                    *is_processing = true;
                    let mut compression_complete = self.compression_complete.write().unwrap();
                    *compression_complete = false;
                    drop(is_processing);
                    drop(compression_complete);

                    run(
                        picked_path.to_string(),
                        self.output_path.as_ref().unwrap().clone(),
                        self.image_dpi,
                        self.pdf_settings.clone(),
                        Arc::clone(&self.is_processing),
                        Arc::clone(&self.compression_complete),
                    );
                }

                ui.add_space(10.0);

                if *self.compression_complete.read().unwrap() {
                    ui.heading("Compression complete!");
                }
            }
        });
    }
}

fn run(
    file_path: String,
    output_path: String,
    image_resolution: u16,
    pdf_settings: PdfSettings,
    is_processing: Arc<RwLock<bool>>,
    compression_complete: Arc<RwLock<bool>>,
) {
    // Some options: https://gist.github.com/ahmed-musallam/27de7d7c5ac68ecbd1ed65b6b48416f9

    std::thread::spawn(move || {
        let child = Command::new(COMMAND)
            .arg("-dBATCH")
            .arg("-dNOPAUSE")
            // .arg("-q")
            .arg("-dCompatibilityLevel=1.4")
            .arg(format!("-dPDFSETTINGS=/{pdf_settings}"))
            .arg("-dCompressFonts=true")
            .arg("-dEmbedAllFonts=true")
            .arg("-dSubsetFonts=true")
            // .arg("-dDetectDuplicateImages=true")
            // .arg("-dDownsampleColorImages=true")
            // .arg("-dDownsampleGrayImages=true")
            // .arg("-dDownsampleMonoImages=true")
            // .arg("-dColorImageDownsampleType=/Bicubic")
            // .arg("-dGrayImageDownsampleType=/Bicubic")
            // .arg("-dMonoImageDownsampleType=/Bicubic")
            .arg(format!("-dColorImageResolution={image_resolution}"))
            .arg(format!("-dGrayImageResolution={image_resolution}"))
            .arg(format!("-dMonoImageResolution={image_resolution}"))
            .arg(format!("-r{image_resolution}"))
            .arg("-sDEVICE=pdfwrite")
            .arg(format!("-sOutputFile={output_path}"))
            .arg(file_path)
            .spawn();

        child.unwrap().wait().unwrap();

        let mut compression_complete = compression_complete.write().unwrap();
        *compression_complete = true;

        let mut is_processing = is_processing.write().unwrap();
        *is_processing = false;
    });
}
