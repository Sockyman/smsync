use crate::error::Error;
use eframe::egui;
use std::sync::mpsc;


pub fn show_error(error: &Error) {
    log::error!("{}", error);
    show_message(
        format!("{}: Error", crate::PROGRAM_NAME),
        error.to_string(),
        "exit".into()
    );
}

fn base_gui_options() -> eframe::NativeOptions {
    eframe::NativeOptions {
        initial_window_size: Some(egui::vec2(640., 480.)),
        ..Default::default()
    }
}

pub fn show_message(title: String, message: String, button_label: String) {
    eframe::run_simple_native(
        &title.clone(),
        base_gui_options(),
        move |ctx, frame| {
            egui::CentralPanel::default().show(ctx, |ui| {
                ctx.set_pixels_per_point(1.);
                //ui.heading(&title);

                ui.label(&title);

                ui.separator();
                //ui.label(format!("{}", message));

                ui.label(egui::RichText::new(&message).monospace());
                
                ui.separator();

                if ui.button(&button_label).clicked() {
                    frame.close();
                }
            });
        }
    ).unwrap();
}


pub fn show_question<T>(
    title: String,
    message: String,
    choices: impl IntoIterator<Item = (String, T)>, start: T
) -> T
    where 
        T: Clone + PartialEq + 'static
{
    let (tx, rx) = mpsc::channel();

    let choices: Vec<_> = choices.into_iter().collect();
    let mut value = start;

    eframe::run_simple_native(
        &title.clone(),
        base_gui_options(),
        move |ctx, frame| {
            egui::CentralPanel::default().show(ctx, |ui| {
                ui.heading(&title);
                ui.separator();
                ui.label(&message);
                ui.separator();

                for choice in choices.iter() {
                    ui.radio_value(&mut value, choice.1.clone(), &choice.0);
                }
                ui.separator();

                if ui.button("select").clicked() {
                    tx.send(value.clone()).unwrap();
                    frame.close();
                }
            });
        }
    ).unwrap();

    rx.recv().unwrap()
}

