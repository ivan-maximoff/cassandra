use std::sync::{Arc, Mutex};

use egui::{Painter, Response, ScrollArea};
use walkers::{Plugin, Projector};

use crate::flight::Flight;

#[derive(Clone)]
pub struct Flights {
    pub flights: Vec<Flight>,
    pub on_flight_selected: Arc<Mutex<Option<Flight>>>
}

impl Flights {
    pub fn new(flights: Vec<Flight>, on_flight_selected: Arc<Mutex<Option<Flight>>>) -> Self {
        Self {
            flights,
            on_flight_selected
        }
    }

    pub fn list_flight_codes(&self, ui: &mut egui::Ui) {
        ScrollArea::vertical().show(ui, |ui| {
            ui.set_width(ui.available_width());
            for flight in &self.flights {
                ui.label(format!("Vuelo: {}", flight.code));
            }
        });
    }
}

impl Plugin for Flights {
    fn run(&mut self, response: &Response, painter: Painter, projector: &Projector) {
        for flight in &self.flights {
            flight.draw(response, painter.clone(), projector, &mut self.on_flight_selected.lock().unwrap());
        }
    }
}