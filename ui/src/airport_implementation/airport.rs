use egui::{vec2, Align2, Color32, FontId, Painter, Pos2, Response, Vec2};
use std::sync::{Arc, Mutex};
use walkers::{Position, Projector};

#[derive(Clone, PartialEq)]
pub struct Airport {
    pub name: String,
    pub code: String,
    pub position: (f64, f64),
}

// Comunicacion con Cassandra o informacion de aeropuerto
impl Airport {
    pub fn new(name: String, code: String, position: (f64, f64)) -> Self {
        Airport {
            name,
            code,
            position,
        }
    }

    pub fn list_information(&self, ui: &mut egui::Ui) {
        ui.label(format!("{} ({})", self.name, self.code));
    }

    pub fn get_airport_pos2(&self, projector: &Projector) -> Pos2 {
        self.get_flight_vec2(projector).to_pos2()
    }

    pub fn get_flight_vec2(&self, projector: &Projector) -> Vec2 {
        let airport_coordinates = self.position;
        let airport_position = Position::from_lon_lat(airport_coordinates.0, airport_coordinates.1);
        projector.project(airport_position)
    }
}

// Interfaz grafica
impl Airport {
    pub fn draw(
        &self,
        response: &Response,
        painter: Painter,
        projector: &Projector,
        selected_airport_code: &Arc<Mutex<Option<String>>>,
    ) {
        self.draw_icon_airport(painter.clone(), projector);
        self.clickeable_airport(response, projector, selected_airport_code);
        self.holdeable_airport(response, painter, projector);
    }

    fn holdeable_airport(&self, response: &Response, painter: Painter, projector: &Projector) {
        let screen_flight_position = self.get_airport_pos2(projector);
        if self.is_hovering_on_airport(response, screen_flight_position) {
            self.draw_description(painter, projector);
        }
    }

    fn draw_description(&self, painter: Painter, projector: &Projector) {
        let screen_flight_position = self.get_flight_vec2(projector);

        let label = painter.layout_no_wrap(
            self.code.to_string(),
            FontId::proportional(12.),
            Color32::from_gray(200),
        );

        let offset = vec2(8., 8.);

        painter.rect_filled(
            label
                .rect
                .translate(screen_flight_position)
                .translate(offset)
                .expand(5.),
            10.,
            Color32::BLACK.gamma_multiply(0.8),
        );

        painter.galley(
            (screen_flight_position + offset).to_pos2(),
            label,
            egui::Color32::BLACK,
        );
    }

    // Dibuja el icono del avion en su posicion
    pub fn draw_icon_airport(&self, painter: Painter, projector: &Projector) {
        let screen_airport_position = self.get_airport_pos2(projector);
        painter.text(
            screen_airport_position,
            Align2::CENTER_CENTER,
            // 📍, 🏫, 📌
            '🏫'.to_string(),
            FontId::proportional(20.0),
            Color32::BLACK,
        );
    }

    // Si lo clikea cambia el aeropuerto seleccionado
    fn clickeable_airport(
        &self,
        response: &Response,
        projector: &Projector,
        selected_airport_code: &Arc<Mutex<Option<String>>>,
    ) {
        let screen_airport_position = self.get_airport_pos2(projector);
        if self.is_hovering_on_airport(response, screen_airport_position)
            && response.clicked_by(egui::PointerButton::Primary)
        {
            let mut selected_airport = match selected_airport_code.lock() {
                Ok(lock) => lock,
                Err(_) => return,
            };
            match &*selected_airport {
                Some(airport) if airport == &self.code => *selected_airport = None,
                Some(_) | None => *selected_airport = Some(self.code.to_string()),
            }
        }
    }

    fn is_hovering_on_airport(&self, response: &Response, screen_airport_position: Pos2) -> bool {
        response.hover_pos().map_or(false, |pos| {
            let airplane_size = egui::Vec2::new(30.0, 30.0);
            let airplane_rect =
                egui::Rect::from_center_size(screen_airport_position, airplane_size);
            airplane_rect.contains(pos)
        })
    }
}
