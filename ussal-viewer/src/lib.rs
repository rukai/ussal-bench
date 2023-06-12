use eframe::{
    egui::{
        self,
        plot::{PlotPoint, Text},
        RichText,
    },
    emath::Align2,
    epaint::Color32,
};
use egui::plot::{Legend, Plot};
use ussal_archive::{Bench, BenchArchive};

struct FilterValue {
    name: String,
    show: bool,
}

struct FilterKey {
    name: String,
    values: Vec<FilterValue>,
}

pub struct App {
    archive: BenchArchive,
    filter_keys: Vec<FilterKey>,
    bench_names: Vec<String>,
}

impl App {
    pub fn new(_cc: &eframe::CreationContext<'_>, archive: BenchArchive) -> Self {
        let mut filter_keys: Vec<FilterKey> = vec![];
        for bench in &archive.benches {
            for (key, value) in &bench.keys {
                if let Some(filter_key) = filter_keys.iter_mut().find(|x| &x.name == key) {
                    if !filter_key.values.iter().any(|x| &x.name == value) {
                        filter_key.values.push(FilterValue {
                            name: value.to_owned(),
                            show: true,
                        })
                    }
                } else {
                    filter_keys.push(FilterKey {
                        name: key.to_owned(),
                        values: vec![FilterValue {
                            name: value.to_owned(),
                            show: true,
                        }],
                    })
                }
            }
        }
        let mut bench_names = vec![];
        for bench in &archive.benches {
            if !bench_names.contains(&bench.name) {
                bench_names.push(bench.name.clone());
            }
        }
        bench_names.sort();

        Self {
            archive,
            filter_keys,
            bench_names,
        }
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
            for filter_key in &mut self.filter_keys {
                ui.heading(&filter_key.name);
                for value in &mut filter_key.values {
                    ui.toggle_value(&mut value.show, &value.name);
                }
            }
        });

        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading(&self.archive.title);
            egui::ScrollArea::vertical().show(ui, |ui| {
                ui.horizontal_wrapped(|ui| {
                    for (i, name) in self.bench_names.iter().enumerate() {
                        plot_bench(ui, i as i32, name, &self.archive.benches, &self.filter_keys);
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

fn plot_bench(
    ui: &mut egui::Ui,
    id: i32,
    name: &str,
    benches: &[Bench],
    filter_keys: &[FilterKey],
) {
    //ui.vertical(|ui| {
    //ui.label(&bench.name);

    let plot = Plot::new(id)
        .legend(Legend::default())
        .width(500.0)
        .height(250.0)
        .allow_scroll(false);

    if benches.iter().any(|bench| show_bench(filter_keys, bench)) {
        plot.show(ui, |plot_ui| {
            for bench in benches.iter() {
                if bench.name == name && show_bench(filter_keys, bench) {
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
                            .name(format!(
                                "{}-{}",
                                bench.keys.get("machine").unwrap(),
                                bench.keys.get("type").unwrap(),
                            )),
                    );
                }
            }
            plot_ui.text(
                Text::new(
                    PlotPoint::new(0.0, 0.0),
                    RichText::new(format!(" {}", name)).size(17.0),
                )
                .anchor(Align2::LEFT_BOTTOM)
                .color(Color32::WHITE),
            );
        });
    }
    // });
}

fn show_bench(filter_keys: &[FilterKey], bench: &Bench) -> bool {
    for (key, value) in &bench.keys {
        let filter_key = filter_keys.iter().find(|x| &x.name == key).unwrap();
        let filter_value = filter_key.values.iter().find(|x| &x.name == value).unwrap();
        if !filter_value.show {
            return false;
        }
    }
    true
}
