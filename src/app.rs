/* This Source Code Form is subject to the terms of the Mozilla Public
License, v. 2.0. If a copy of the MPL was not distributed with this
file, You can obtain one at https://mozilla.org/MPL/2.0/.
Copyright 2021 Peter Dunne */

use eframe::{
    egui::{self, Widget},
    epi,
};
use libcaliph::routines;

#[cfg_attr(feature = "persistence", derive(serde::Deserialize, serde::Serialize))]
#[cfg_attr(feature = "persistence", serde(default))]
pub struct TemplateApp {
    ph4: f64,
    ph10: f64,
    temperature: f64,
    slope: f64,
    offset: f64,
    ph_measured: f64,
    calibrated_ph: f64,
}

impl Default for TemplateApp {
    fn default() -> Self {
        Self {
            // Example stuff:
            ph4: 4.01,
            ph10: 10.01,
            temperature: 25.0,
            slope: 1.0,
            offset: 0.0,
            ph_measured: 7.0,
            calibrated_ph: 7.0,
        }
    }
}

impl epi::App for TemplateApp {
    fn name(&self) -> &str {
        "Caliphui"
    }

    /// Called once before the first frame.
    fn setup(
        &mut self,
        _ctx: &egui::CtxRef,
        _frame: &mut epi::Frame<'_>,
        _storage: Option<&dyn epi::Storage>,
    ) {
        // let mut spacing_mut = egui::style::Spacing::default();
        // //
        // spacing_mut.item_spacing = egui::Vec2::new(8.0, 12.0);
        // spacing_mut.interact_size = egui::Vec2::new(40.0, 18.0);

        let _spacing_mut = egui::style::Spacing {
            item_spacing: egui::Vec2::new(8.0, 12.0),
            window_padding: egui::Vec2::splat(6.0),
            button_padding: egui::Vec2::new(4.0, 1.0),
            indent: 18.0, // match checkbox/radio-button with `button_padding.x + icon_width + icon_spacing`
            interact_size: egui::Vec2::new(40.0, 18.0),
            slider_width: 100.0,
            text_edit_width: 280.0,
            icon_width: 14.0,
            icon_spacing: 0.0,
            tooltip_width: 600.0,
            combo_height: 200.0,
            scroll_bar_width: 8.0,
            indent_ends_with_horizontal_line: false,
        };

        let mut fonts = egui::FontDefinitions::default();

        fonts.family_and_size.insert(
            egui::TextStyle::Heading,
            (egui::FontFamily::Proportional, 26.0),
        );
        fonts.family_and_size.insert(
            egui::TextStyle::Body,
            (egui::FontFamily::Proportional, 22.0),
        );

        fonts.family_and_size.insert(
            egui::TextStyle::Button,
            (egui::FontFamily::Proportional, 22.0),
        );
        fonts.family_and_size.insert(
            egui::TextStyle::Monospace,
            (egui::FontFamily::Monospace, 22.0),
        );

        _ctx.set_fonts(fonts);
        // Load previous app state (if any).
        // Note that you must enable the `persistence` feature for this to work.
        #[cfg(feature = "persistence")]
        if let Some(storage) = _storage {
            *self = epi::get_value(storage, epi::APP_KEY).unwrap_or_default()
        }

        self.calibrated_ph =
            update_conversion(&self.ph_measured, &mut self.slope, &mut self.offset);
    }

    /// Called by the frame work to save state before shutdown.
    #[cfg(feature = "persistence")]
    fn save(&mut self, storage: &mut dyn epi::Storage) {
        epi::set_value(storage, epi::APP_KEY, self);
    }

    /// Called each time the UI needs repainting, which may be many times per second.
    fn update(&mut self, ctx: &egui::CtxRef, _frame: &mut epi::Frame<'_>) {
        let Self {
            ph4,
            ph10,
            temperature,
            slope,
            offset,
            ph_measured,
            calibrated_ph,
        } = self;

        egui::TopBottomPanel::top("top_panel").show(ctx, |ui| {
            egui::widgets::global_dark_light_mode_switch(ui);
        });

        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("Calibrate");
            let ph4_slider = egui::Slider::new(ph4, 0.0..=14.0)
                .text("pH 4")
                .fixed_decimals(2);
            let ph4_response = ph4_slider.ui(ui);
            if ph4_response.changed() {
                update_calibration(ph4, ph10, temperature, slope, offset);
                *calibrated_ph = update_conversion(ph_measured, slope, offset);
            }

            let ph10_slider = egui::Slider::new(ph10, 0.0..=14.0)
                .text("pH 10")
                .fixed_decimals(2);
            let ph10_response = ph10_slider.ui(ui);
            if ph10_response.changed() {
                update_calibration(ph4, ph10, temperature, slope, offset);
                *calibrated_ph = update_conversion(ph_measured, slope, offset);
            }

            let temperature_slider = egui::Slider::new(temperature, 0.0..=100.0)
                .suffix(" ˚C")
                .text("T")
                .fixed_decimals(1);
            let temperature_response = temperature_slider.ui(ui);
            if temperature_response.changed() {
                update_calibration(ph4, ph10, temperature, slope, offset);
                *calibrated_ph = update_conversion(ph_measured, slope, offset);
            }

            ui.add_space(5.0);

            // ================
            ui.heading("Convert");
            let ph_measured_slider = egui::Slider::new(ph_measured, 0.0..=14.0)
                .text("Input pH")
                .fixed_decimals(2);
            let ph_measured_response = ph_measured_slider.ui(ui);
            if ph_measured_response.changed() {
                *calibrated_ph = update_conversion(ph_measured, slope, offset);
            }

            ui.add_space(10.0);
            ui.heading("Calibrated pH:");
            ui.add(egui::DragValue::new(calibrated_ph).speed(1.0));

            egui::warn_if_debug_build(ui);
        });

        if false {
            egui::Window::new("Window").show(ctx, |ui| {
                ui.label("Windows can be moved by dragging them.");
                ui.label("They are automatically sized based on contents.");
                ui.label("You can turn on resizing and scrolling if you like.");
                ui.label("You would normally chose either panels OR windows.");
            });
        }
    }
}

pub fn update_calibration(
    ph4: &f64,
    ph10: &f64,
    temperature: &f64,
    slope: &mut f64,
    offset: &mut f64,
) {
    let calibration = routines::ph_calibration(&[*ph4, *ph10], temperature);
    *slope = calibration.slope;
    *offset = calibration.offset;
}

pub fn update_conversion(ph_measured: &f64, slope: &mut f64, offset: &mut f64) -> f64 {
    routines::ph_convert(ph_measured, &[*slope, *offset])
}
