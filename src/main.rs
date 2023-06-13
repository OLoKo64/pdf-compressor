use eframe::egui;
use std::process::Command;

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

#[derive(PartialEq)]
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
    compression_complete: bool,
}

impl Default for MyApp {
    fn default() -> Self {
        Self {
            picked_path: None,
            output_path: None,
            pdf_settings: PdfSettings::Screen,
            image_dpi: 150,
            compression_complete: false,
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

                    self.compression_complete = false;
                }
            }

            ui.separator();

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

                if ui.button("Compress PDF").clicked() {
                    run(
                        picked_path,
                        self.output_path.as_deref().unwrap(),
                        self.image_dpi,
                        &self.pdf_settings,
                        &mut self.compression_complete,
                    );
                }

                ui.add_space(10.0);

                if self.compression_complete {
                    ui.heading("Compression complete!");
                }
            }
        });
    }
}

fn run(
    file_path: &str,
    output_path: &str,
    image_resolution: u16,
    pdf_settings: &PdfSettings,
    compression_complete: &mut bool,
) {
    // Some options: https://gist.github.com/ahmed-musallam/27de7d7c5ac68ecbd1ed65b6b48416f9

    #[cfg(target_os = "linux")]
    let command = "gs";

    #[cfg(target_os = "windows")]
    let command = "gswin64";

    let child = Command::new(command)
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

    *compression_complete = true;
}
