use uuid::Uuid;
use serde::{Serialize, Deserialize};
use wasm_bindgen::{JsCast};
use edom::ElementNode;
use edom::Visitor;

#[derive(PartialEq)]
enum ShowState {
    All,
    Completed,
    Active
}

#[derive(Serialize, Deserialize)]
struct TodoItem {
    id:Uuid,
    completed:bool,
    description: String
}

use edom;

const ENTER_KEY: u32 = 13;
const ESCAPE_KEY: u32 = 27;

use wasm_bindgen::prelude::{wasm_bindgen, JsValue};

// TODO: how to run server?
#[wasm_bindgen(start)]
pub fn main() {
    todomvc();
}

fn todomvc() {
    let window = web_sys::window().unwrap();
    let local_storage=window.local_storage().unwrap().unwrap();
    let mut todolist: Vec<TodoItem>= Vec::new();
    if let Some(data)=local_storage.get_item("todos-edom").unwrap() {
        todolist=serde_json::from_str(data.as_str()).unwrap();
    }
    let mut show_state=ShowState::All;
    let mut new_text=String::new();
	let mut editing = None;
    let mut edit_value = None;
    edom::wasm::render(move |mut root| {
        local_storage.set_item("todos-edom", serde_json::to_string(&todolist).unwrap().as_str()).unwrap();
        let mut root=root.element("span");
        /*
            root.head(|head| {
                head.title("TODOMVC implemented with EDOM");
            })
         */
        // root.edom.window.on("hashchange", |_| {
        //     show_state = match root.edom.window.location {
        //         "#/active" => ShowState::Active,
        //         "#/completed" => ShowState::Completed,
        //         _ => ShowState::All
        //     }
        // });
        let num_active = todolist.iter_mut().skip_while(|item| !item.completed).count();

        root.header(|header| {
            header.h1().text("todos");
            header.text_input(&mut new_text);
            header.text_input(&mut new_text).class("new-todo")
            .placeholder("What needs to be done?").autofocus(true).on("keydown", |e| {
                if e.dyn_ref::<web_sys::KeyboardEvent>().unwrap().which() == ENTER_KEY {
                    todolist.push(TodoItem {id: Uuid::new_v4(), completed: false, description: new_text.clone()});
                    new_text.clear();
                    web_sys::console::log_2(&"Enter".into(), &new_text.clone().into());
                    // e.target().unwrap().dyn_ref::<web_sys::HtmlElement>().unwrap().blur().unwrap();
                    e.prevent_default();
                }
            });
        }).class("header");
       
        root.render_element_if(todolist.len()>0, "section", |mut section| {
            section.class("main");
            let mut toggle_all = num_active==0;
            if section.checkbox(&mut toggle_all).id("toggle-all").class("toggle-all").changed() {
                for todo in todolist.iter_mut() {
                    todo.completed = toggle_all;
                }
            };
            section.label("toggle-all", "Mark all as complete");
            let mut remove=None;

            section.ul(|ul| {
                ul.class("todo-list");

                let filtered = todolist.iter_mut().filter(|item|
                    match show_state {
                        ShowState::Completed => item.completed,
                        ShowState::All => true,
                        ShowState::Active => !item.completed
                    }
                );
                ul.for_each(filtered, |todo| todo.id, "li", |todo, li|  {
                    li.classes(&[("completed", todo.completed), ("editing", editing==Some(todo.id))]);
                    li.div(|view| {
                        view.class("view");
                        view.checkbox(&mut todo.completed).class("toggle");
                        if view.label("", todo.description.as_str()).double_clicked() {
                            editing=Some(todo.id);
                            edit_value=Some(todo.description.clone());
                        }
                        if view.button("").class("destroy").clicked() {
                            remove=Some(todo.id);
                        }
                    });
                    li.render_element_if(Some(todo.id) == editing, "span", |li2| {
                        li2.text_input(&mut new_text).id("edit").class("edit").autofocus(true).on("keydown", |e| {
                            let which=e.dyn_ref::<web_sys::KeyboardEvent>().unwrap().which();
                            if which == ENTER_KEY {
                                todo.description=new_text.clone();
                                editing=None;
                                e.target().unwrap().dyn_ref::<web_sys::HtmlElement>().unwrap().blur().unwrap();
                            } else if which == ESCAPE_KEY {
                                editing = None;
                                e.target().unwrap().dyn_ref::<web_sys::HtmlElement>().unwrap().blur().unwrap();
                            }
                        });
                    });
                });
            });
            if section.button("clear_completed").clicked() {
                todolist.retain(|e| !e.completed);
            }
            render_footer(&mut section, &mut todolist, num_active, &mut show_state);
            if let Some(remove_id) = remove {
                todolist.retain(|e| e.id != remove_id);
            }
        });
    });
}

fn render_footer<EN: edom::ElementNode>(container : &mut edom::Visitor<EN>, todolist :&mut Vec<TodoItem>,
        num_active: usize, show_state: &mut ShowState) {
    container.footer(|footer| {
        footer.span(|span| {
            span.class("todo-count");
            span.strong().text(num_active.to_string().as_str());
            span.text(if num_active==1 {"item left"} else {"items left"});
        });
        footer.ul(|ul| {
            ul.li(|li| {
                li.a("#/").class(if *show_state==ShowState::All {"selected"} else {""});
            });
            ul.li(|li| {
                li.a("#/active").class(if *show_state==ShowState::Active {"selected"} else {""});
            });
            ul.li(|li| {
                li.a("#/completed").class(if *show_state==ShowState::Completed {"selected"} else {""});
            });
        });
        footer.render_element_if(num_active < todolist.len(), "span", |footer| {
            if footer.button("Clear completed").class("clear-completed").clicked() {
                todolist.retain(|item| !item.completed);
            }
        });
    });
}
