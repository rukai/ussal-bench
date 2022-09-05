use crate::app::egui::plot::{Legend, Plot};
use eframe::egui::{self};
use ussal_shared::{BenchResult, BenchRun};

pub struct App {
    bench_run: BenchRun,
}

impl App {
    pub fn new(_cc: &eframe::CreationContext<'_>, bench_run: BenchRun) -> Self {
        Self { bench_run }
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
            ui.heading(&self.bench_run.name);
            egui::ScrollArea::vertical().show(ui, |ui| {
                ui.horizontal_wrapped(|ui| {
                    for (i, result) in self.bench_run.results.iter().enumerate() {
                        plot_result(ui, result, i as i32);
                    }
                    plot(ui, "circle_plot");
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

fn plot_result(ui: &mut egui::Ui, result: &BenchResult, id: i32) {
    // TODO: couldnt get this playing nicely, lets just directly implement the desired legend header directly into egui
    //ui.label(&result.name);

    let plot = Plot::new(id)
        .legend(Legend::default())
        .width(500.0)
        .height(250.0)
        .allow_scroll(false);

    plot.show(ui, |plot_ui| {
        for measurement in &result.measurements {
            let line = egui::plot::PlotPoints::new(vec![[0.0, measurement.value as f64]]);
            plot_ui.line(
                egui::plot::Line::new(line)
                    .color(egui::Color32::from_rgb(100, 200, 100))
                    .name(&measurement.name),
            )
        }
    });
}
