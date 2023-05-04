mod report;

use crate::model::{Model, Report};
use crate::view::app::AppInterface;
use std::sync::{Arc, Mutex};
use tokio::sync::mpsc;

use self::report::save_report;

pub struct CommandInterface {
    model: Arc<Mutex<Model>>,
    channel: mpsc::Sender<()>,
}

impl AppInterface for CommandInterface {
    fn model(self: &Self) -> Model {
        self.model.lock().unwrap().clone()
    }

    fn model_op<O>(self: &Self, op: O)
    where
        O: FnOnce(Model) -> Model,
    {
        let mut model = self.model.lock().unwrap();
        *model = op(model.clone());
    }
}

impl Default for CommandInterface {
    fn default() -> Self {
        let report = Report::default();
        save_report(&report);

        let model = Arc::new(Mutex::new(Model::default()));
        let (channel, _) = mpsc::channel(32);
        return Self { model, channel };
    }
}
