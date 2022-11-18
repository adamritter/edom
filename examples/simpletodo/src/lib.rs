// Run with trunk `serve --open' in examples/simpletodo directory.
use edom;
use wasm_bindgen::prelude::wasm_bindgen;
use serde::{Serialize, Deserialize};
use wasm_bindgen::JsCast;

#[derive(Serialize, Deserialize)]
struct TodoItem {
    done: bool,
    text: String,
    id: u64,
}

#[wasm_bindgen(start)]
pub fn simpletodo() {
    let window = web_sys::window().unwrap();
    let local_storage=window.local_storage().unwrap().unwrap();
    let mut n:u64=0;
    let mut todolist:Vec<TodoItem>=Vec::new();
    let mut new_text=String::new();
    let mut editing:Option<u64>=None;
    let mut edit_text=String::new();
    let mut focus_edit=false;

    if let Some(data)=local_storage.get_item("simpletodo-edom").unwrap() {
        (n, todolist)=serde_json::from_str(data.as_str()).unwrap();
    }

    edom::wasm::render(move |mut root| {
        root.h1().text("Simple todo list");
        root.form(|form| {
            form.text_input(&mut new_text);
            if form.on_submit() {
                todolist.push(TodoItem {done: false, text: new_text.clone(), id: n});
                new_text.clear();
                n+=1;
            }
        });
        
        root.ul(|ul| {
            ul.style("list-style: none; margin: 0; padding: 0;");
            let mut delete=None;
            ul.for_each(todolist.iter_mut(), |item| item.id, "li", |item,li| {
                li.style("display: flex; ; margin: 0; padding: 0;");
                li.checkbox(&mut item.done);
                li.render_element_if(editing!=Some(item.id), "span", |view| {
                    if view.span(|el| el.text(item.text.as_str())).double_clicked() {
                        editing=Some(item.id);
                        edit_text=item.text.clone();
                        focus_edit=true;
                    }
                });
                li.render_element_if(editing==Some(item.id), "span", |edit| {
                    edit.style("display: flex");
                    edit.form(|form| {
                        let input=form.text_input(&mut edit_text);
                        if focus_edit {
                            input.focus();
                            focus_edit=false;
                        }

                        if form.on_submit() {
                            item.text=edit_text.clone();
                            editing=None;
                        }
                    });
                    if edit.button("Cancel").clicked() { editing=None; }
                    if edit.button("Delete").clicked() { delete=Some(item.id); }
                });
            });
            if let Some(delete_id)=delete {
                todolist.retain(|item| item.id!=delete_id);
            }
        });
        local_storage.set_item("simpletodo-edom",
            serde_json::to_string(&(n, &todolist)).unwrap().as_str()).unwrap();
    });
}