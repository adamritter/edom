use uuid::Uuid;

enum ShowState {
    All,
    Completed,
    Active
}

#[derive(Serialize, Deserialize)]
struct TodoItem {
    id:Uuid,
    completed:bool,
    text: String
}

use edom;

const ENTER_KEY: u32 = 13;
const ESCAPE_KEY: u32 = 27;

fn todomvc() {
    let local_storage=web_sys::document().unwrap().window().unwrap().local_storage();
    let mut todolist: Vec<TodoItem>= Vec::new();
    if Some(data)=local_storage.get_item("todos-edom").unwrap() {
        todolist=serde_json::from_str(data);
    }
    let mut show_state=ShowState::All;
    let new_text=String::new();
	let mut editing = None;
    let mut edit_value = None;

    edom::render(|root| {
        local_storage.set_item("todos-edom", serde_json::to_str(todolist));
        root.edom.window.on("hashchange", || {
            show_state = match root.edom.window.location {
                "#/active" => ShowState::Active,
                "#/completed" => ShowState::Completed,
                _ => ShowState::All
            }
        });
        let num_active = items.iter_mut().skip_while(|item| !item.completed).len();

        root.header(|header| {
            header.h1().text("todos");
            header.input(&new_text).class("new-todo").placeholder("What needs to be done?")
            .autofocus(true).on("keydown", |e: Event| {
                if(e.which == ENTER_KEY) {
                    todolist.push(TodoItem {id: Uuid::new_v4(), completed: false, text: new_text.clone()});
                    new_text.clear();
                    e.target.blur();
                }
            });
        }).class("header");
       
        root.render_if(!todolist.empty(), |root| {
            root.section(|section| {
                section.class("main");
                let mut toggle_all=(num_active==0);
                if section.checkbox(&toggle_all).id("toggle-all").class("toggle-all").changed() {
                    for todo in todolist.iter_mut() {
                        todo.completed = toggle_all;
                    }
                };
                setion.label("toggle-all", "Mark all as complete");
                section.ul(|ul| {
                    ul.class("todo-list");
                    let mut remove=None;
                    let filtered = match(show_state) {
                        ShowState::Completed => items.iter().filter(|item| item.completed),
                        ShowState::All => items.iter(),
                        ShowState::Active => items.iter().filter(|item| !item.completed)
                    };
                    let finish_edit=false;
                    ul.for_each(filtered.iter(), |todo| todo.id, "li", |todo, li|  {
                        li.class((if todo.completed {"completed"} else {""}) + " " + (if editing=todo.id {"editing"} else {""}));
                        li.div(|view| {
                            view.class("view");
                            view.checkbox(&mut todo.completed).class("toggle");
                            if view.label("", todo.description).double_clicked() {
                                editing=todo.id;
                                edit_value=todo.description;
                            }
                            if view.button("").class("destroy").clicked() {
                                remove=Some(todo.id);
                            }
                        });
                        li.render_if(todo.id == editing, |li2| {
                            li2.input(&mut edit_value).id("edit").class("edit").autofocus(true).on("keydown", |e| {
                                    if event.which == ENTER_KEY {
                                        todo.description=edit_value;
                                        editing=None;
                                        e.target.blur();
                                    } else if event.which == ESCAPE_KEY {
                                        editing = None;
                                        e.target.blur();
                                    }
                                });
                        })
                    });
                });
                if root.button("clear_completed").clicked() {
                    todolist.retain(|e| !e.completed);
                }
                render_footer(root, &mut todolist, num_active);
                if let Some(remove_id) = remove {
                    todolist.retain(|e| e.id != remove_id);
                }
            });
        });
    });
}

fn footer(container : ElementIterator<EN : ElementNode>, todolist :&mut Vec<TodoItem>, num_active: usize) {
    container.footer(|footer| {
        footer.span(|span| {
            span.class("todo-count");
            span.strong().text(num_active);
            span.text(if num_active==1 {"item left"} else {"items left"});
        });
        footer.ul(|ul| {
            ul.li(|li| {
                li.a("#/").class(if currentFilter==ShowState::ALL {"selected"} else {""})
            });
            ul.li(|li| {
                li.a("#/active").class(if currentFilter==ShowState::ACTIVE {"selected"} else {""})
            });
            ul.li(|li| {
                li.a("#/completed").class(if currentFilter==ShowState::COMPLETED {"selected"} else {""})
            });
        });
        footer.render_if(num_active < todolist.len(), |footer| {
            if footer.button("Clear completed").class("clear-completed").clicked() {
                todolist.retain(|item| !item.completed);
            }
        });
    });
}
