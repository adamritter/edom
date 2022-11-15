use std::{cell::RefCell, rc::Rc};

pub trait ElementNode : Sized {
    type GenericNode : GenericNode<TextNode=Self::TextNode, ElementNode=Self>;
    type TextNode : TextNode;
    type Document : Document<TextNode=Self::TextNode, ElementNode=Self>;
    type EventHandler : EventHandler<ElementNode=Self,Event=Self::Event>;
    type Event : Event;
    fn replace_text_child(&self, new: &Self::TextNode, old: &Self::TextNode);
    fn append_child(&self, child: &Self);
    fn insert_child_before(&self, child: &Self::GenericNode, next_sibling: Option<&Self::GenericNode>);
    fn append_child_after(&self, child: &Self, prev_sibling: &Self);
    fn remove_child(&self, child: &Self);
    fn prepend_child(&self, child: &Self);
    fn append_text_child(&self, child: &Self::TextNode);
    fn set_attribute(&self, name: &str, value: &str);
    fn get_attribute(&self, name: &str)->String;
    fn remove(&self);
    fn new(tag: &'static str)->Self;
    fn create_dnode_event_listener(&self, f : Rc<RefCell<dyn FnMut(u64, &'static str)>>, uid:u64, name:&'static str);
    fn deep_clone(&self)->Self;
    fn get_child_nodes(&self)->Vec<Self::GenericNode>;
    fn get_child_node(&self, i:u32)->Option<Self::GenericNode>;
    fn set_text_content(&self, s:&str);
    fn into_generic_node(&self)->&Self::GenericNode;
}

pub trait GenericNode  : Sized {
    type TextNode : TextNode;
    type ElementNode : ElementNode;
    fn into_text_node(self)->Self::TextNode;
    fn into_element_node(self)->Self::ElementNode;
}

pub trait EventHandler {
    type ElementNode:ElementNode;
    type Event:Event;
    fn new(fire_event: Rc<RefCell<Box<dyn FnMut(u64, String, Self::Event)>>>)->Self;
    fn create_event_listener(&self, e: &Self::ElementNode, name: String);
}

pub trait Document {
    type TextNode : TextNode;
    type ElementNode : ElementNode;
    fn create_text_node(&self, text: &str)->Self::TextNode;
    fn new()->Self;
    fn create_element(&self, tag: &'static str)->Self::ElementNode;
    fn log_1(s: &str);
    fn log_2(s: &str, s2: &str);
}

pub trait TextNode {
    fn new(text: &str)->Self;
}

pub trait Event {}
