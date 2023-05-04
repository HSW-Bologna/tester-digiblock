use crate::model::Model;
use std::ops::FnOnce;

pub trait AppInterface {
    fn model(self: &Self) -> Model;
    fn model_op<O>(self: &Self, op: O)
    where
        O: FnOnce(Model) -> Model;
}

pub struct App<I: AppInterface> {
    interface: I,
}

impl<I> App<I>
where
    I: AppInterface,
{
    pub fn new(interface: I) -> Self {
        Self { interface }
    }
}

impl<I> eframe::App for App<I>
where
    I: AppInterface,
{
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("Collaudo Arag");

            if ui.button("+").clicked() {
                self.interface.model_op(Model::increase);
            }

            ui.label(format!("count {}", self.interface.model().count));

            if ui.button("-").clicked() {
                self.interface.model_op(Model::decrease);
            }
        });
    }
}
