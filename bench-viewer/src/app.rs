use eframe::egui;

pub struct TemplateApp {
    memory: Vec<u8>,
}

impl TemplateApp {
    pub fn new(_cc: &eframe::CreationContext<'_>, rom: Option<String>) -> Self {
        let mut memory = rom
            .map(|path| std::fs::read(path).unwrap())
            .unwrap_or_default();
        memory.resize(0x10000, 0);
        Self { memory }
    }
}

impl eframe::App for TemplateApp {
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
            egui::ScrollArea::vertical().show(ui, |ui| {
                for (i, window) in self.memory.chunks(8).enumerate() {
                    ui.horizontal(|ui| {
                        ui.monospace(format!(
                            "0x{:04x}: {:02x}{:02x}{:02x}{:02x} {:02x}{:02x}{:02x}{:02x}",
                            i * 8,
                            window[0],
                            window[1],
                            window[2],
                            window[3],
                            window[4],
                            window[5],
                            window[6],
                            window[7]
                        ));
                    });
                }
            });
        });

        egui::SidePanel::right("right_panel").show(ctx, |ui| {
            ui.heading("Right Panel");

            if ui.button("do something").clicked() {
                todo!();
            }
        });

        egui::CentralPanel::default().show(ctx, |ui| {
            // The central panel the region left after adding TopPanel's and SidePanel's

            ui.heading("eframe template");
            ui.hyperlink("https://github.com/emilk/eframe_template");
            ui.add(egui::github_link_file!(
                "https://github.com/emilk/eframe_template/blob/master/",
                "Source code."
            ));
            egui::warn_if_debug_build(ui);

            egui::ScrollArea::vertical().show(ui, |ui| {
                ui.horizontal_wrapped(|ui| {
                    plot(ui, "bench1");
                    plot(ui, "bench2");
                    plot(ui, "bench3");
                    plot(ui, "bench4");
                    plot(ui, "bench5");
                    plot(ui, "bench6");
                    plot(ui, "bench7");
                    plot(ui, "bench8");
                    plot(ui, "bench9");
                    plot(ui, "bench10");
                    plot(ui, "bench11");
                    plot(ui, "bench12");
                    plot(ui, "bench13");
                    plot(ui, "bench14");
                    plot(ui, "bench15");
                })
            });

            ui.with_layout(egui::Layout::bottom_up(egui::Align::LEFT), |ui| {
                ui.horizontal(|ui| {
                    ui.spacing_mut().item_spacing.x = 0.0;
                    ui.label("powered by ");
                    ui.hyperlink_to("bench_viewer", "https://github.com/rukai/bench_viewer");
                });
            });
        });
    }
}

fn plot(ui: &mut egui::Ui, name: &str) {
    use crate::app::egui::plot::{Legend, Plot};
    let plot = Plot::new(name)
        .legend(Legend::default())
        .width(500.0)
        .height(250.0)
        .allow_scroll(false);
    plot.show(ui, |plot_ui| {
        plot_ui.line(circle());
    });
}

fn circle() -> egui::plot::Line {
    let n = 512;
    let circle: egui::plot::PlotPoints = (0..=n)
        .map(|i| {
            let t = egui::remap(i as f64, 0.0..=(n as f64), 0.0..=std::f64::consts::TAU);
            let r = 1.0;
            [r * t.cos() as f64, r * t.sin() as f64]
        })
        .collect();
    egui::plot::Line::new(circle)
        .color(egui::Color32::from_rgb(100, 200, 100))
        .name("circle")
}
