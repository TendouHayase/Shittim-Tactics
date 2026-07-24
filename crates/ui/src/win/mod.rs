pub mod config;
pub mod result;

use config::ConfigPanel;
use result::ResultPanel;

const APP_TITLE: &str = "Shittim-Tactics";

pub struct ShittimTacticsApp {
    config: ConfigPanel,
    result: ResultPanel,
    selected_tab: Tab,
}

#[derive(Default, PartialEq, Eq)]
enum Tab {
    #[default]
    Configuration,
    Results,
}

impl ShittimTacticsApp {
    pub fn new() -> Self {
        Self {
            config: ConfigPanel::default(),
            result: ResultPanel::default(),
            selected_tab: Tab::default(),
        }
    }
}

impl eframe::App for ShittimTacticsApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::TopBottomPanel::top("menu_bar").show(ctx, |ui| {
            ui.horizontal(|ui| {
                ui.heading(APP_TITLE);
                ui.separator();
                ui.selectable_value(&mut self.selected_tab, Tab::Configuration, "Configuration");
                ui.selectable_value(&mut self.selected_tab, Tab::Results, "Results");
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    ui.label("Blue Archive Raid Optimizer");
                });
            });
        });

        egui::CentralPanel::default().show(ctx, |ui| {
            match self.selected_tab {
                Tab::Configuration => {
                    egui::ScrollArea::vertical().show(ui, |ui| {
                        self.config.show(ui);
                    });
                }
                Tab::Results => {
                    self.result.show(ui);
                }
            }
        });
    }
}
