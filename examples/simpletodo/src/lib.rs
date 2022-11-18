use edom;
use wasm_bindgen::prelude::wasm_bindgen;
use serde::{Serialize, Deserialize};
use std::collections::HashSet;
use chrono;
use chrono::{TimeZone, Utc};
use chrono::serde::ts_seconds;

#[derive(Serialize, Deserialize)]
struct TodoItem {
    text: String,
    id: u64,
    labels: HashSet<String>,
    priority: u8,
    date: Option<chrono::NaiveDate>,
    time: Option<chrono::NaiveTime>,
}

#[wasm_bindgen(start)]
pub fn simpletodo() {
    let window = web_sys::window().unwrap();
    let local_storage=window.local_storage().unwrap().unwrap();
    let mut n:u64=0;
    let mut todolist:Vec<TodoItem>=Vec::new();
    let mut new_text=String::new();
    let mut current_label: Option<String>=None;
    let current_date : Option<chrono::NaiveDate>=None;
    let mut all_labels: Vec<String>=Vec::new();
    let mut new_label=String::new();

    if let Some(data)=local_storage.get_item("simpletodos-edom").unwrap() {
        todolist=serde_json::from_str(data.as_str()).unwrap();
    }

    edom::wasm::render(move |mut root| {
        root.h1().text("Todo list");
        root.form(|form| {
            form.text_input(&mut new_text);
            if form.on_submit() {
                let mut labels=HashSet::new();
                if !current_label.is_none() {
                    labels.insert(current_label.as_ref().unwrap().clone());
                }
                todolist.push(TodoItem {text: new_text.clone(), id: n, labels,
                    date: current_date, time: None, priority: 4});
                new_text.clear();
                n+=1;
            }
        });
        root.h1().text("Labels");
        root.ul(|ul| {
            ul.for_each(all_labels.iter_mut(), |label| (*label).clone(), "li", |label,li|{
                li.render_element_if(Some(label.clone())==current_label, "span",
                        |el| el.text(label.as_str()));
                li.render_element_if(Some(label.clone())!=current_label, "span", |el| {
                    if el.a("-", label).clicked() {
                        current_label=Some(label.clone());
                    }
                });
            });
        });

        root.form(|form| {
            form.text_input(&mut new_label);
            if form.on_submit() {
                all_labels.push(new_label.clone());
                all_labels.sort();
                new_label.clear();
            }
        });
        root.h1().text("Todos");
        root.ul(|ul| {
            ul.for_each(todolist.iter_mut(), |item| item.id, "li", |item,li|{
                li.text(item.text.as_str());
            });
        });
    });
}