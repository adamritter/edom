use std::cell::RefCell;
use std::panic;
use std::rc::Rc;

use super::dom;
pub struct WasmEventHandler {
    closure: Closure<dyn FnMut(web_sys::Event)>
}
impl dom::EventHandler for WasmEventHandler {
    type ElementNode = web_sys::Element;
    type Event = web_sys::Event;
    fn new(fire_event: Rc<RefCell<Box<dyn FnMut(u64, String, web_sys::Event)>>>)->Self {
        let closure=Closure::wrap(Box::new(move |e: web_sys::Event| {
            let el : web_sys::Element=e.target().unwrap().dyn_into().unwrap();
            web_sys::console::log_2(&"in event handler data-uid string=".to_string().into(),
                &el.get_attribute("data-uid").unwrap().to_string().into());
            let uid:u64=el.get_attribute("data-uid").unwrap().parse::<u64>().unwrap();
            let name=e.type_();
            web_sys::console::log_2(&"in event handler e=".to_string().into(),
                &e.type_().into());
            web_sys::console::log_2(&"in event handler data-uid=".to_string().into(),
                &uid.to_string().into());
            // e.prevent_default();
        ((*fire_event).borrow_mut())(uid, name.to_string(), e);
        })  as Box<dyn FnMut(_)>);
        Self {closure}
    }
    fn create_event_listener(&self, e: &Self::ElementNode, name: String) {
        e.add_event_listener_with_callback(
                name.as_str(), self.closure.as_ref().unchecked_ref()).unwrap();
    }
}




impl dom::Document for web_sys::Document {
    type TextNode=web_sys::Text;
    type ElementNode=web_sys::Element;
    fn new()->Self {
        panic::set_hook(Box::new(console_error_panic_hook::hook));
        let window = web_sys::window().unwrap();
        window.document().unwrap()
    }
    fn create_text_node(&self, text: &str)->web_sys::Text {
        self.create_text_node(text)
    }
    
    fn create_element(&self, tag: &'static str)->web_sys::Element {
        self.create_element(tag).unwrap()
    }
    fn log_1(s: &str) {
        web_sys::console::log_1(&s.into());
    }
    fn log_2(s: &str, s2: &str) {
        web_sys::console::log_2(&s.into(), &s2.into());
    }
}

impl dom::TextNode for web_sys::Text {
    fn new(text:&str)->Self {
        let window = web_sys::window().unwrap();
        let document = window.document().unwrap();
        document.create_text_node(text)
    }
}


impl dom::GenericNode for web_sys::Node {
    type ElementNode = web_sys::Element;
    type TextNode = web_sys::Text;
    fn into_text_node(self)->Self::TextNode {
        self.dyn_into().unwrap()
    }
    fn into_element_node(self)->Self::ElementNode {
        self.dyn_into().unwrap()
    }
}


use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;
use web_sys::HtmlElement;


impl dom::Event for web_sys::Event {
    fn prevent_default(&self) {
        web_sys::Event::prevent_default(&self)
    }
}

impl dom::ElementNode for web_sys::Element {
    type TextNode = web_sys::Text;
    type Document = web_sys::Document;
    type GenericNode = web_sys::Node;
    type EventHandler = WasmEventHandler;
    type Event = web_sys::Event;
    fn replace_text_child(&self, new: &Self::TextNode, old: &Self::TextNode) {
        self.replace_child(new, old).unwrap();
    }
    fn append_child(&self, child: &Self) {
        web_sys::Node::append_child(self, child).unwrap();
    }
    fn insert_child_before(&self, child: &web_sys::Node, next_sibling: Option<&web_sys::Node>) {
        self.insert_before(child, next_sibling).unwrap();
    }
    fn append_child_after(&self, child: &Self, prev_sibling: &Self) {
        let next_sibling=prev_sibling.next_sibling();
        self.insert_before(child, next_sibling.as_ref()).unwrap();
    }
    fn remove_child(&self, child: &Self) {
        web_sys::Node::remove_child(self, child).unwrap();
    }
    fn prepend_child(&self, child: &Self) {
        self.insert_before(child, self.first_child().as_ref()).unwrap();
    }
    fn append_text_child(&self, child: &Self::TextNode) {
        self.insert_before(child, None).unwrap();
    }
    fn set_attribute(&self, name: &str, value: &str) {
        if name=="checked" {
            let e: &web_sys::HtmlInputElement= self.dyn_ref().unwrap();
            e.set_checked(value=="true");
        } else if name=="value" {
            let e: &web_sys::HtmlInputElement= self.dyn_ref().unwrap();
            e.set_value(value);
         } else {
            web_sys::Element::set_attribute(self, name, value).unwrap();
        }
    }
    fn get_attribute(&self, name: &str)->String {
        if name=="checked" {
            let e: &web_sys::HtmlInputElement= self.dyn_ref().unwrap();
            if e.checked() {"true".to_string()} else {"false".to_string()}
        } else if name=="value" {
            let e: &web_sys::HtmlInputElement= self.dyn_ref().unwrap();
            e.value()
        } else {
            web_sys::Element::get_attribute(self, name).unwrap()
        }
    }
    fn remove(&self) {
        web_sys::Element::remove(&self);
    }
    fn new(tag: &'static str)->Self {
        let window = web_sys::window().unwrap();
        let document = window.document().unwrap();
        document.create_element(tag).unwrap()
    }
    fn create_dnode_event_listener(&self, f : Rc<RefCell<dyn FnMut(u64, &'static str)>>, uid:u64, name:&'static str) {
        let closure = Closure::wrap(Box::new(move |e: Self::Event| {
            web_sys::console::log_2(&uid.to_string().into(), &name.into());
            web_sys::console::log_2(&"e=".to_string().into(), &e.type_().into());

            f.borrow_mut()(uid, name);
            // e.prevent_default();
        })  as Box<dyn FnMut(_)>);
        self.add_event_listener_with_callback(name, closure.as_ref().unchecked_ref()).unwrap();
        closure.forget();
    }
    fn deep_clone(&self)->Self {
        self.clone_node_with_deep(true).unwrap().dyn_into::<web_sys::Element>().unwrap()
    }
    fn get_child_nodes(&self)->Vec<Self::GenericNode> {
        let this_node : &web_sys::Node = self.as_ref();
        let node_list = this_node.child_nodes();
        let mut r : Vec<Self::GenericNode>=Vec::new();
        for i in 0..node_list.length() {
            r.push(node_list.get(i).unwrap());
        }
        r
    }

    fn get_child_node(&self, i: u32)->Option<Self::GenericNode> {
        let this_node : &web_sys::Node = self.as_ref();
        if i==0 {
            return this_node.first_child();
        }
        let node_list = this_node.child_nodes();
        node_list.get(i)
    }
    fn set_text_content(&self, s:&str) {
        web_sys::Node::set_text_content(self, Some(s));
    }
    fn into_generic_node(&self)->&Self::GenericNode {
        self.as_ref()
    }
    fn focus(&self) {
        self.dyn_ref::<HtmlElement>().unwrap().focus().unwrap();
    }
}

pub fn render<F>(f:F) where F:FnMut(super::Visitor<web_sys::Element>) + 'static {
    let window = web_sys::window().unwrap();
    let document = window.document().unwrap();
    let body = document.body().unwrap();
    let body=web_sys::Element::from(body);
    super::EDOM::render(body, f);
}