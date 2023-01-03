use eframe::egui::{self};
use egui::plot::{Legend, Plot};
use ussal_shared::{Bench, BenchArchive};

pub struct App {
    archive: BenchArchive,
}

impl App {
    pub fn new(_cc: &eframe::CreationContext<'_>, archive: BenchArchive) -> Self {
        Self { archive }
    }
}

impl eframe::App for App {
    /// Called each time the UI needs repainting, which may be many times per second.
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::TopBottomPanel::top("top_panel").show(ctx, |ui| {
            // The top panel is often a good place for a menu bar:
            egui::menu::bar(ui, |ui| {
                ui.menu_button("File", |ui| {
                    if ui.button("Do Something").clicked() {
                        todo!("Just leaving this here in case I want to add menu options later")
                    }
                });
            });
        });

        egui::SidePanel::left("side_panel").show(ctx, |ui| {
            ui.heading("something");
        });

        egui::SidePanel::right("right_panel").show(ctx, |ui| {
            ui.heading("Right Panel");

            if ui.button("do something").clicked() {
                todo!();
            }
        });

        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading(&self.archive.name);
            egui::ScrollArea::vertical().show(ui, |ui| {
                ui.horizontal_wrapped(|ui| {
                    for (i, result) in self.archive.benches.iter().enumerate() {
                        plot_bench(ui, result, i as i32);
                    }
                })
            });

            ui.with_layout(egui::Layout::bottom_up(egui::Align::LEFT), |ui| {
                ui.horizontal(|ui| {
                    ui.spacing_mut().item_spacing.x = 0.0;
                    ui.label("powered by ");
                    ui.hyperlink_to("ussal-bench", "https://github.com/rukai/ussal-bench");
                });
            });
        });
    }
}

fn plot_bench(ui: &mut egui::Ui, bench: &Bench, id: i32) {
    // TODO: couldnt get this playing nicely, lets just directly implement the desired legend header directly into egui
    //ui.label(&result.name);

    let plot = Plot::new(id)
        .legend(Legend::default())
        .width(500.0)
        .height(250.0)
        .allow_scroll(false);

    plot.show(ui, |plot_ui| {
        let line = egui::plot::PlotPoints::new(
            bench
                .measurements
                .iter()
                .enumerate()
                .map(|(i, x)| [i as f64, x.value as f64])
                .collect(),
        );
        plot_ui.line(
            egui::plot::Line::new(line)
                .color(egui::Color32::from_rgb(100, 200, 100))
                .name(bench.keys.get("type").unwrap()),
        );
    });
}
