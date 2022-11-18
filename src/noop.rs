use std::{cell::RefCell, rc::Rc};

use super::dom;

pub struct EventHandler {
}

impl dom::EventHandler for EventHandler {
    type ElementNode=ElementNode;
    type Event=Event;
    fn new(_fire_event: Rc<RefCell<Box<dyn FnMut(u64, String, Event)>>>)->Self {
        Self {}
    }
    fn create_event_listener(&self, _e: &Self::ElementNode, _name: String) {
    }
}

pub struct Node {
}


impl dom::GenericNode for Node {
    type ElementNode = ElementNode;
    type TextNode = TextNode;
    fn into_text_node(self)->Self::TextNode {
        TextNode { text: "hello".to_string()}
    }
    fn into_element_node(self)->Self::ElementNode {
        ElementNode { tag: "hello", generic_node: Node {  }}
    }
}

pub struct Event {}
impl dom::Event for Event {
    fn prevent_default(&self) {}
}



impl  dom::ElementNode for ElementNode {
    type TextNode=TextNode;
    type Document=Document;
    type GenericNode=Node;
    type EventHandler=EventHandler;
    type Event=Event;
    fn new(tag: &'static str)->Self {
        Self { tag, generic_node: Node {  }}
    } 
    fn create_dnode_event_listener(&self, _f : Rc<RefCell<dyn FnMut(u64, &'static str)>>, _uid:u64,
            _name:&'static str) {
    }
    fn set_text_content(&self, _s:&str) {
    }
    fn replace_text_child(&self, _new: &TextNode, _old: &TextNode) {
    }
    fn append_child(&self, _child: &ElementNode) {

    }
    fn insert_child_before(&self, _child: &Node, _next_sibling: Option<&Node>) {

    }
    fn append_child_after(&self, _child: &ElementNode, _prev_sibling: &ElementNode) {

    }
    fn remove_child(&self, child: &Self) {
    }
    fn prepend_child(&self, _child: &ElementNode) {

    }
    fn append_text_child(&self, _child: &TextNode) {

    }
    fn set_attribute(&self, _name: &str, _value: &str) {

    }
    fn get_attribute(&self, _name: &str)->String {
        "no_attribute".to_string()
    }

    fn remove(&self) {
    }
    fn deep_clone(&self)->Self {
        ElementNode { tag: self.tag, generic_node: Node {  } }
    }
    fn get_child_nodes(&self)->Vec<Self::GenericNode> {
        Vec::new()
    }
    fn get_child_node(&self, _i:u32)->Option<Self::GenericNode> {
        Some(Node {})
    }
    fn into_generic_node(&self)->&Self::GenericNode {
        &self.generic_node
    }

}


pub struct ElementNode {
    pub generic_node: Node,
    pub tag: &'static str
}

pub struct Document {
}
impl dom::Document for Document {
    type TextNode=TextNode;
    type ElementNode=ElementNode;
    fn create_text_node(&self, text: &str)->TextNode {
        TextNode {  text: text.to_string() }
    }
    fn new()->Self {
        Document {}
    }
    fn create_element(&self, tag: &'static str)->Self::ElementNode {
        ElementNode {generic_node: Node {  }, tag}
    }
    fn log_1(s: &str) {
        println!("{}", s);
    }
    fn log_2(s: &str, s2: &str) {
        println!("{} {}", s, s2);
    }
}

pub struct TextNode  {
    pub text: String
}
impl dom::TextNode for TextNode {
    fn new(text: &str)->Self {
        Self { text: text.to_string() }
    }
}