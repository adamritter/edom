
extern crate console_error_panic_hook;
extern crate smallstr;

use web_sys;
use std::{collections::{HashSet, HashMap}};
use std::panic;
use std::slice::IterMut;

struct CachedValue<T> {
    value: std::cell::UnsafeCell<Option<T>>
}

impl<T> CachedValue<T> {
    fn new(v:Option<T>)->Self {
        Self {value: std::cell::UnsafeCell::new(v)}
    }
    fn get<'a,F>(&self, f : F)->&'a T where F:FnOnce()->T {
        let v=unsafe {&mut *self.value.get()};
        if v.is_none() {
            *v=Some(f());
        }
        return v.as_ref().unwrap();
    }
    fn is_none(&self)->bool {
        unsafe {&mut *self.value.get()}.is_none()
    }
    fn unwrap<'a>(&self)->&'a T {
        let v=unsafe {&mut *self.value.get()};
        if v.is_none() {
            panic!("Couldn't get cached value");
        }
        return v.as_ref().unwrap();
    }
}


fn add_offset(big_indexer: usize, delta: isize) -> Option<usize> {
    if delta < 0 {
        big_indexer.checked_sub(delta.wrapping_abs() as usize)
    } else {
        big_indexer.checked_add(delta as usize)
    }
}
pub trait TextNode {
    fn new(text: &str)->Self;
}
pub struct NoopTextNode  {
    pub text: String
}
impl TextNode for NoopTextNode {
    fn new(text: &str)->Self {
        Self { text: text.to_string() }
    }
}
pub struct NoopElementNode {
    pub tag: &'static str
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

pub struct NoopDocument {
}
impl Document for NoopDocument {
    type TextNode=NoopTextNode;
    type ElementNode=NoopElementNode;
    fn create_text_node(&self, text: &str)->NoopTextNode {
        NoopTextNode {  text: text.to_string() }
    }
    fn new()->Self {
        NoopDocument {}
    }
    fn create_element(&self, tag: &'static str)->Self::ElementNode {
        NoopElementNode {tag}
    }
    fn log_1(s: &str) {
        println!("{}", s);
    }
    fn log_2(s: &str, s2: &str) {
        println!("{} {}", s, s2);
    }
}

pub trait EventHandler {
    type ElementNode:ElementNode;
    fn new(fire_event: Rc<RefCell<Box<dyn FnMut(u64, String)>>>)->Self;
    fn create_event_listener(&self, e: &Self::ElementNode, name: String);
}

pub struct WasmEventHandler {
    closure: Closure<dyn FnMut(web_sys::Event)>
}
impl EventHandler for WasmEventHandler {
    type ElementNode = web_sys::Element;
    fn new(fire_event: Rc<RefCell<Box<dyn FnMut(u64, String)>>>)->Self {
        let closure=Closure::wrap(Box::new(move |e: web_sys::Event| {
            let el : web_sys::Element=e.target().unwrap().dyn_into().unwrap();
            web_sys::console::log_2(&"in event handler data-uid string=".to_string().into(), &el.get_attribute("data-uid").unwrap().to_string().into());
            let uid:u64=el.get_attribute("data-uid").unwrap().parse::<u64>().unwrap();
            let name=e.type_();
            web_sys::console::log_2(&"in event handler e=".to_string().into(), &e.type_().into());
            web_sys::console::log_2(&"in event handler data-uid=".to_string().into(), &uid.to_string().into());

        ((*fire_event).borrow_mut())(uid, name.to_string());
            e.prevent_default();
        })  as Box<dyn FnMut(_)>);
        Self {closure}
    }
    fn create_event_listener(&self, e: &Self::ElementNode, name: String) {
        e.add_event_listener_with_callback(
                name.as_str(), self.closure.as_ref().unchecked_ref()).unwrap();
    }
}

pub struct NoopEventHandler {
}

impl EventHandler for NoopEventHandler {
    type ElementNode=NoopElementNode;
    fn new(_fire_event: Rc<RefCell<Box<dyn FnMut(u64, String)>>>)->Self {
        Self {}
    }
    fn create_event_listener(&self, _e: &Self::ElementNode, _name: String) {
    }
}



impl Document for web_sys::Document {
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

impl TextNode for web_sys::Text {
    fn new(text:&str)->Self {
        let window = web_sys::window().unwrap();
        let document = window.document().unwrap();
        document.create_text_node(text)
    }
}

pub trait GenericNode  : Sized {
    type TextNode : TextNode;
    type ElementNode : ElementNode;
    fn into_text_node(self)->Self::TextNode;
    fn into_element_node(self)->Self::ElementNode;
}
impl GenericNode for web_sys::Node {
    type ElementNode = web_sys::Element;
    type TextNode = web_sys::Text;
    fn into_text_node(self)->Self::TextNode {
        self.dyn_into().unwrap()
    }
    fn into_element_node(self)->Self::ElementNode {
        self.dyn_into().unwrap()
    }
}

pub struct NoopNode {
}

impl GenericNode for NoopNode {
    type ElementNode = NoopElementNode;
    type TextNode = NoopTextNode;
    fn into_text_node(self)->Self::TextNode {
        NoopTextNode { text: "hello".to_string()}
    }
    fn into_element_node(self)->Self::ElementNode {
        NoopElementNode { tag: "hello"}
    }
}

pub trait ElementNode : Sized {
    type GenericNode : GenericNode<TextNode=Self::TextNode, ElementNode=Self>;
    type TextNode : TextNode;
    type Document : Document<TextNode=Self::TextNode, ElementNode=Self>;
    type EventHandler : EventHandler<ElementNode=Self>;
    fn replace_text_child(&self, new: &Self::TextNode, old: &Self::TextNode);
    fn append_child(&self, child: &Self);
    fn append_child_before(&self, child: &Self, next_sibling: &Self);
    fn append_child_after(&self, child: &Self, prev_sibling: &Self);
    fn prepend_child(&self, child: &Self);
    fn append_text_child(&self, child: &Self::TextNode);
    fn set_attribute(&self, name: &str, value: &str);
    fn remove(&self);
    fn new(tag: &'static str)->Self;
    fn create_dnode_event_listener(&self, f : Rc<RefCell<dyn FnMut(u64, &'static str)>>, uid:u64, name:&'static str);
    fn deep_clone(&self)->Self;
    fn get_child_nodes(&self)->Vec<Self::GenericNode>;
    fn get_child_node(&self, i:u32)->Self::GenericNode;
    fn set_text_content(&self, s:&str);

}

use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;
use web_sys::Event;
use std::rc::Rc;
use std::cell::RefCell;

impl ElementNode for web_sys::Element {
    type TextNode = web_sys::Text;
    type Document = web_sys::Document;
    type GenericNode = web_sys::Node;
    type EventHandler = WasmEventHandler;
    fn replace_text_child(&self, new: &Self::TextNode, old: &Self::TextNode) {
        self.replace_child(new, old).unwrap();
    }
    fn append_child(&self, child: &Self) {
        web_sys::Node::append_child(self, child).unwrap();
    }
    fn append_child_before(&self, child: &Self, next_sibling: &Self) {
        self.insert_before(child, Some(next_sibling)).unwrap();
    }
    fn append_child_after(&self, child: &Self, prev_sibling: &Self) {
        let next_sibling=prev_sibling.next_sibling();
        self.insert_before(child, next_sibling.as_ref()).unwrap();
    }
    fn prepend_child(&self, child: &Self) {
        self.insert_before(child, self.first_child().as_ref()).unwrap();
    }
    fn append_text_child(&self, child: &Self::TextNode) {
        self.insert_before(child, None).unwrap();
    }
    fn set_attribute(&self, name: &str, value: &str) {
        web_sys::Element::set_attribute(self, name, value).unwrap();
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
        let closure = Closure::wrap(Box::new(move |e: Event| {
            web_sys::console::log_2(&uid.to_string().into(), &name.into());
            web_sys::console::log_2(&"e=".to_string().into(), &e.type_().into());

            f.borrow_mut()(uid, name);
            e.prevent_default();
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

    fn get_child_node(&self, i: u32)->Self::GenericNode {
        let this_node : &web_sys::Node = self.as_ref();
        if i==0 {
            return this_node.first_child().unwrap();
        }
        let node_list = this_node.child_nodes();
        node_list.get(i).unwrap()
    }
    fn set_text_content(&self, s:&str) {
        web_sys::Node::set_text_content(self, Some(s));
    }

}
impl  ElementNode for NoopElementNode {
    type TextNode=NoopTextNode;
    type Document=NoopDocument;
    type GenericNode=NoopNode;
    type EventHandler=NoopEventHandler;
    fn new(tag: &'static str)->Self {
        Self { tag }
    } 
    fn create_dnode_event_listener(&self, _f : Rc<RefCell<dyn FnMut(u64, &'static str)>>, _uid:u64,
            _name:&'static str) {
    }
    fn set_text_content(&self, _s:&str) {
    }
    fn replace_text_child(&self, _new: &NoopTextNode, _old: &NoopTextNode) {
    }
    fn append_child(&self, _child: &NoopElementNode) {

    }
    fn append_child_before(&self, _child: &NoopElementNode, _next_sibling: &NoopElementNode) {

    }
    fn append_child_after(&self, _child: &NoopElementNode, _prev_sibling: &NoopElementNode) {

    }
    fn prepend_child(&self, _child: &NoopElementNode) {

    }
    fn append_text_child(&self, _child: &NoopTextNode) {

    }
    fn set_attribute(&self, _name: &str, _value: &str) {

    }
    fn remove(&self) {
    }
    fn deep_clone(&self)->Self {
        NoopElementNode { tag: self.tag }
    }
    fn get_child_nodes(&self)->Vec<Self::GenericNode> {
        Vec::new()
    }
    fn get_child_node(&self, _i:u32)->Self::GenericNode {
        NoopNode {}
    }

}


enum Node<EN> where EN:ElementNode {
    Text(String, Option<EN::TextNode>),
    Element(Element<EN>),
    ForEach(Vec<(u64, Element<EN>)>)
}

struct Element<EN> where EN:ElementNode {
    name: &'static str,
    attr: Vec<(&'static str,String)>,
    children: Vec<Node<EN>>,
    dnode: CachedValue<EN>,
    events: Vec<&'static str>,
    uid: u64,
}

impl<EN> Element<EN>  where EN:ElementNode {
    fn new(name: &'static str, dnode: Option<EN>, uid: u64)->Self {
        Self {name, attr:vec![], children: vec![], dnode: CachedValue::new(dnode), events: Vec::new(), uid}
    }
    fn create_event_listener(&self, name: &'static str, edom: &EDOM<EN>, dnode: &EN) {
        dnode.set_attribute("data-uid", self.uid.to_string().as_str());
        edom.event_handler.create_event_listener(dnode, name.to_string());
        // dnode.create_dnode_event_listener(edom.fire_event.clone(), self.uid, name);
    }
    fn clone_using_dnode(&self, target_dnode: EN, edom: &mut EDOM<EN>)->Self {
        let mut r = Self {name: self.name, attr: self.attr.clone(), children: Vec::new(), dnode: CachedValue::new(Some(target_dnode)), events: self.events.clone(), uid: edom.next_uid()};
        // TODO: attach events
        let rdnode=r.dnode.get(|| panic!("Should exist"));
        for event_name in &self.events {
            self.create_event_listener(event_name, edom, rdnode);
        }
        // TODO: attach children
        let mut next_child_idx=0;
        let new_dchildren=rdnode.get_child_nodes();
        for new_dchild in new_dchildren.into_iter() {
            let child = &self.children[next_child_idx];
            match child {
                Node::Text(s, _)=>{
                    r.children.push(Node::Text(s.clone(), Some(new_dchild.into_text_node())));
                },
                Node::Element(e)=>{
                    r.children.push(Node::Element(e.clone_using_dnode(new_dchild.into_element_node(), edom)));
                },
                _ => {
                    panic!("Only cloning element and text is supported so far");
                }
            }
            next_child_idx+=1;
        }
        r
    }

    fn shallow_clone(&self, target_dnode: Option<EN>, edom: &mut EDOM<EN>)->Self {
        Self {name: self.name, attr: self.attr.clone(), children: Vec::new(), dnode: CachedValue::new(target_dnode), events: self.events.clone(), uid: edom.next_uid()}
    }

    fn partial_clone_using_dnode(&self, target_iterator: ElementIterator<EN>) {
        // Attach events
        for event_name in &self.events {
            target_iterator.element.create_event_listener(event_name, target_iterator.edom, target_iterator.get_dnode());
        }
        // Attach children
        let mut i=0;
        for child in &self.children {
            let new_elem=match child {
                Node::Text(s, _)=>Node::Text(s.clone(), None),
                Node::Element(e)=> {
                    let mut new_elem=e.shallow_clone(None, target_iterator.edom);
                    let ctarget_iterator : *const ElementIterator<EN>=&target_iterator;
                    let it=ElementIterator::new(target_iterator.edom, &mut new_elem, i, Some(ctarget_iterator));
                    e.partial_clone_using_dnode(it);
                    Node::Element(new_elem)
                }
                _ => {
                    panic!("Only cloning element and text is supported so far");
                }
            };
            target_iterator.element.children.push(new_elem);
            i+=1;
        }
    }
    
}

pub struct ElementIterator<'d, 'e, EN> where EN: ElementNode {
    edom: &'d mut EDOM<EN>,
    element: &'e mut Element<EN>,
    attrpos: usize,
    childpos: usize,
    eventpos: usize,
    parent_access_pos: usize,
    parent_iterator: Option<*const ElementIterator<'d, 'd, EN>>
}

impl<'d, 'e, 'f, 'a, 'z, 'c, 'q, EN> ElementIterator<'d, 'e, EN> where EN:ElementNode {
    fn create_element_iterator(&'f mut self, child_pos: usize)->ElementIterator<'f, 'f, EN> {
        let cself : *const ElementIterator<EN>=self;
        let Node::Element(element)=&mut self.element.children[child_pos] else {
            panic!("Not Element")
        };
        ElementIterator {edom: self.edom, element: element, attrpos: 0, childpos: 0, eventpos: 0, parent_access_pos: child_pos, parent_iterator: Some(cself)}
    }
    pub fn element(&'f mut self, name : &'static str)->ElementIterator<'f, 'f, EN> {
        let new_pos= if self.edom.create {
            self.get_dnode();
            let dnode=self.element.dnode.unwrap();
            let children=&mut self.element.children;
            let elem= Element::new(name, Some(self.edom.document.create_element(name)), self.edom.next_uid());
            dnode.append_child(&elem.dnode.get(||panic!("Dnode empty")));
            let i = children.len();
            children.push(Node::Element(elem));
            i
        } else {
            let i=self.childpos;
            self.childpos+=1;
            i
        };
        self.create_element_iterator(new_pos)
    }
     pub fn button(&'f mut self, text: &str)->ElementIterator<'f,'f,EN> {
        let mut elem=self.element("button");
        elem.text(text);
        elem
    }
    pub fn div<FCB>(&'f mut self, mut fcb: FCB)->ElementIterator<'f,'f,EN> where FCB:FnMut(&mut ElementIterator<EN>) {
        let mut r=self.element("div");
        fcb(&mut r);
        return r;
    }
    pub fn h1(&'f mut self)->ElementIterator<'f,'f,EN> {
        self.element("h1")
    }
    fn new(edom:&'d mut EDOM<EN>, element:&'e mut Element<EN>, parent_access_pos: usize, parent_iterator: Option<*const ElementIterator<'d ,'d,EN>>)->ElementIterator<'d,'e,EN> {
         ElementIterator {edom, element, attrpos: 0, childpos: 0, eventpos: 0, parent_access_pos, parent_iterator}
     }
    fn get_dnode2(dnode: &'a CachedValue<EN>, parent_iterator: &Option<*const ElementIterator<EN>>, parent_access_pos: usize)->&'a EN {
        dnode.get(|| unsafe {&**parent_iterator.as_ref().unwrap()}.get_dnode().get_child_node(parent_access_pos as u32).into_element_node())
    }
    fn get_dnode(&self)->&EN {
        Self::get_dnode2(&self.element.dnode, &self.parent_iterator, self.parent_access_pos)
        // self.element.dnode.get(|| unsafe {(&**self.parent_iterator.as_ref().unwrap())}.get_dnode().get_child_node(self.parent_access_pos as u32).into_element_node())
    }

    pub fn attr(&'f mut self, name: &'static str, value: &str)->&'f mut ElementIterator<'d,'e,EN> {
        if self.edom.create {
            self.get_dnode().set_attribute(name, value);
            self.element.attr.push((name, value.into()));
        } else { 
            let thisattr=&mut self.element.attr[self.attrpos];
            if thisattr.0 != name {
                panic!("name change")
            }
            if thisattr.1 != value {
                self.get_dnode().set_attribute(name, value)
            }
            self.attrpos+=1
        }
        self
    }
    
    pub fn text(&mut self, text:&str) {
        let use_set_text_content = true;
        if self.edom.create {
            if use_set_text_content && self.element.children.len()==0 {
                self.get_dnode().set_text_content(text);
                self.element.children.push(Node::Text(text.into(), None));

            } else {
                let tdnode=self.edom.document.create_text_node(text);
                self.get_dnode().append_text_child(&tdnode);
                let elem=Node::Text(text.into(), Some(tdnode));
                self.element.children.push(elem);
            }
        } else {
            let n=self.element.children.len();
            let elem = &mut self.element.children[self.childpos];
            if let Node::Text(text2, tdnode)=elem {
                if *text != **text2 {
                    Self::get_dnode2(&self.element.dnode, &self.parent_iterator, self.parent_access_pos);
                    let dnode=self.element.dnode.unwrap();
                    *text2=text.into();
                    if n==1 && use_set_text_content {
                        *tdnode=None;
                        dnode.set_text_content(text);
                    } else {
                        let new_child=self.edom.document.create_text_node(text);
                        if tdnode.is_none() {
                            *tdnode=Some(dnode.get_child_node(self.childpos as u32).into_text_node());
                        }
                        dnode.replace_text_child(&new_child, tdnode.as_ref().unwrap());
                        *tdnode=Some(new_child);
                    }
                }
            } else {
                panic!("Not text");
            }
            self.childpos+=1;
        }
    }
    fn event<F>(&'f mut self, name:&'static str, mut f: F)->&'f mut Self where F:FnMut() {
        if self.edom.create {
            self.element.events.push(name);
            self.element.create_event_listener(name, self.edom, self.get_dnode());

            // self.get_dnode().create_event_listener(self.edom.fire_event.clone(), self.element.uid, name);
        } else if let Some(ev) = &self.edom.firing_event  {
            if self.element.uid == ev.0 {
                if *self.element.events[self.eventpos]==*ev.1  {
                    f();
                }
                self.eventpos+=1;
            }
        }
        self
    }
    
    pub fn id(&'f mut self, id: &str)->&'f mut ElementIterator<'d,'e,EN> {
        self.attr("id", id)
    }
    pub fn class(&'f mut self, id: &str)->&'f mut Self {
        self.attr("class", id)
    }
   
    pub fn click<F>(&'c mut self, f:F)->&'c mut Self where F:FnMut() {
        self.event("click", f)
    }
    pub fn clicked(&'c mut self)->bool {
        let mut r=false;
        self.event("click", || r=true);
        r
    }
    
    fn for_each_consolidate_changes<FIdx, FCB, I>(&'f mut self, list : IterMut<I>, mut fidx: FIdx, tag: &'static str, mut fcb: FCB) where FIdx:FnMut(&I)->u64, FCB:FnMut(&mut I, &mut ElementIterator<EN>) {
        let self_ptr : *mut ElementIterator<EN>=self;
        self.get_dnode();
        let dnode = self.element.dnode.unwrap();
        let Node::ForEach(v) : &mut Node<EN>=&mut self.element.children[self.childpos] else {
            panic!("Bad node");
        };
        EN::Document::log_2("consolidate, vlength:", &v.len().to_string());

        // position[idx]+relpos will contain the position of idx Element for all shown elements in v[ii] where ii>=i.
        let mut position:HashMap<u64, usize>=HashMap::new();
        for (i, (idx, _)) in &mut v.iter().enumerate() {
            position.insert(*idx, i);
        }
        let mut relpos: isize=0;
        let mut wrong_place: HashSet<u64>=HashSet::new();
        let mut edom : &mut EDOM<EN>=&mut self.edom;

        // first_create is slower for some reason
        let first_create=true;
        let mut i=0;

        for mut e in list {
            let idx=fidx(&e);
            if let Some(pos)=position.get(&idx).cloned() {
                let abspos=add_offset(pos, relpos).unwrap();
                if abspos!=i {
                    // Switch, set wrong_place indicators.
                    wrong_place.insert(idx);
                    wrong_place.insert(v[i].0);
                    v.swap(i, abspos);
                    // Update positions
                    position.insert(v[i].0, i);
                    position.insert(v[abspos].0, abspos);
                }
                if wrong_place.contains(&idx) {
                    wrong_place.remove(&idx);
                    if i==0 {
                        dnode.prepend_child(&v[i].1.dnode.unwrap());
                    } else {
                        dnode.append_child_after(&v[i].1.dnode.unwrap(),
                        &v[i-1].1.dnode.unwrap())
                    }
                }
                let mut it : ElementIterator<EN>=ElementIterator::new(
                    edom, &mut v[i].1, i, Some(self_ptr));
                fcb(&mut e, &mut it);
                edom=it.edom;
            } else {
                let last_elem=if i>0 {Some(&v[0].1)} else {None};
                let elem=Self::create_for_each_element(
                    e, edom, v.len(),
                    &mut fcb, self_ptr, 
                    tag, last_elem);

                v.insert(i, (idx, elem));
                relpos+=1;
                    
                if i+1 == v.len() {
                    dnode.append_child(&v[i].1.dnode.unwrap());
                } else {
                    dnode.append_child_before(&v[i].1.dnode.unwrap(), 
                        &v[i+1].1.dnode.unwrap());  // Not sure that this is the next sibling
                }
            }
            i+=1;
        }

        while v.len() > i {  // Remove remaining children
            v.last().unwrap().1.dnode.unwrap().remove();
            v.pop();
        }

        self.childpos+=1;
    }

    fn create_for_each_element<'x, FCB, I>(
            item: &mut I, mut edom: &'x mut EDOM<EN>, parent_access_pos: usize,
            mut fcb: FCB, self_ptr: *const ElementIterator<'x, 'x, EN>, 
            tag: &'static str, last_elem: Option<&Element<EN>>)->Element<EN> where FCB:FnMut(&mut I, &mut ElementIterator<EN>)  {
        // Create new DOM or clone.
        let mut element:Element<EN>;
        if last_elem.is_none() || !edom.clone_for_each {
            let create=edom.create;
            edom.create=true;
            element=Element::new(tag, Some(edom.document.create_element(tag)), edom.next_uid());
            let mut it:ElementIterator<EN>=ElementIterator::new(edom, &mut element, parent_access_pos, Some(self_ptr));
            fcb(item, &mut it);
            edom=it.edom;
            edom.create=create;
        } else {
            let new_dnode=last_elem.as_ref().unwrap().dnode.unwrap().deep_clone();
            let create=edom.create;
            edom.create=false;
            if edom.use_partial_clone {
                element=last_elem.unwrap().shallow_clone(Some(new_dnode), edom);
                let it=ElementIterator::new(edom, &mut element, parent_access_pos, None);
                last_elem.unwrap().partial_clone_using_dnode(it);
            } else {
                element=last_elem.unwrap().clone_using_dnode(new_dnode, edom);
            }
            let mut it:ElementIterator<EN>=ElementIterator::new(edom, &mut element, parent_access_pos, Some(self_ptr));
            fcb(item, &mut it);
            edom=it.edom;
            edom.create=create;
        };
        element
    }

    pub fn create_for_each<FIdx, FCB, I>(&mut self, list : IterMut<I>, 
            mut fidx: FIdx, tag: &'static str, mut fcb: FCB)
            where FIdx:FnMut(&I)->u64, FCB:FnMut(&mut I, &mut ElementIterator<EN>) {
        let self_ptr : *mut ElementIterator<EN>=self;
       
        self.get_dnode();
        self.element.children.push(Node::ForEach(Vec::new()));
        let element=&mut self.element;
        let Node::ForEach(v)=element.children.last_mut().unwrap() else {
            panic!("Not foreach")
        };
        let mut s:HashSet<u64>=HashSet::new();
        let dnode = element.dnode.unwrap();
        let mut last_elem:Option<&Element<EN>>=None;

        for l in list {
            let idx=fidx(&*l);
            if !s.insert(idx) {
                panic!("Idx must be unique.")
            }

            let elem=Self::create_for_each_element(
                l, self.edom, v.len(),
                &mut fcb, self_ptr, 
                tag, last_elem);

            let elem_dnode=elem.dnode.unwrap();
            dnode.append_child(elem_dnode);
            v.push((idx, elem));
            last_elem=Some(&v.last().unwrap().1);
        }
    }

    pub fn for_each<FIdx, FCB, I>(&mut self, list : IterMut<I>, fidx: FIdx, tag: &'static str, fcb: FCB)
            where FIdx:FnMut(&I)->u64, FCB:FnMut(&mut I, &mut ElementIterator<EN>) {
        if self.edom.create {
            self.create_for_each(list, fidx, tag, fcb);
        } else {
            self.for_each_consolidate_changes(list, fidx, tag, fcb);
        }
    }
}

pub struct EDOM<EN> where EN:ElementNode {
    firing_event: Option<(u64, String)>,
    last_uid: u64,
    create: bool,
    document: EN::Document,
    fire_event: Rc<RefCell<Box<dyn FnMut(u64, String)>>>,
    clone_for_each: bool,  // Clone node for for_each instead of building up the DOM tree.
    nodes_attached: u64,
    use_partial_clone: bool,
    root: Option<Element<EN>>,
    event_handler: EN::EventHandler
}

impl<EN> EDOM<EN> where EN:ElementNode {
    fn next_uid(&mut self)->u64 {
        let r=self.last_uid;
        self.last_uid+=1;
        return r;
    }
    
    fn render_once<F>(&mut self, mut f:F) where EN:ElementNode, F:FnMut(ElementIterator<EN>) {
        let mut root=self.root.take();
        let ei=ElementIterator::new(self, root.as_mut().unwrap(), 0, None);
        f(ei);
        self.root=root;
        self.create=false;
    }

    pub fn render<F>(root: EN, mut f:F)->Rc<RefCell<EDOM<EN>>>
            where EN:ElementNode + 'static, F:FnMut(ElementIterator<EN>) + 'static {
        let el=Element::new("body", Some(root), 0);
        let mut edom : EDOM<EN>=EDOM::new(el);
        assert_eq!(0, edom.next_uid());
        edom.create=true;
        edom.render_once(&mut f);

        let fire_event=edom.fire_event.clone();
        let edomrc : Rc<RefCell<EDOM<EN>>>=Rc::new(RefCell::new(edom));
        let moved_edomrc=edomrc.clone();
        *fire_event.borrow_mut()=Box::new(move |a:u64, b:String| {
            EN::Document::log_2(a.to_string().as_str(), b.as_str());
            let mut edom=moved_edomrc.borrow_mut();
            edom.nodes_attached=0;
            edom.firing_event=Some((a, b));
            edom.render_once(&mut f);
            edom.firing_event=None;
            edom.render_once(&mut f);
            EN::Document::log_2("Nodes attached in render:", edom.nodes_attached.to_string().as_str());
        });
        std::mem::forget(fire_event);
        edomrc
   }

    fn new(root: Element<EN>)->Self {
        let fire_event:Rc<RefCell<Box<dyn FnMut(u64, String)>>>=Rc::new(RefCell::new(Box::new(|a:u64, b:String| web_sys::console::log_3(&"rc".into(), &a.to_string().into(), &b.into()))));
        let fe2=fire_event.clone();
        EDOM {fire_event, firing_event: None, last_uid: 0, create: true, document:EN::Document::new(),
            root: Some(root),
            nodes_attached: 0,
            clone_for_each: true, 
            use_partial_clone: true,
            event_handler: EN::EventHandler::new(fe2)
        }
    }
}


#[test]
fn test_create() {
    EDOM::render( NoopElementNode {tag:"body"}, move |mut root| {
        assert_eq!(0, root.element.children.len());
        root.div(|main|{
            assert_eq!(0, main.element.children.len());
            main.id("main");
            main.div(|container| {
                container.class("container");
            });
            assert_eq!(1, main.element.children.len());
        });
        assert_eq!(1, root.element.children.len());
    });
}

#[test]
fn test_nodes_attached() {
    let nobody=NoopElementNode {tag:"body"};
    let edom=EDOM::render(nobody, 
            move |mut root| {
        root.div(|main|{
            main.id("main");
            main.div(|container| {
                container.class("container");
            });
        });

        let mut table=root.element("tbody");
        let mut v:Vec<u64>=vec![2,5,9];

        table.for_each(v.iter_mut(), |e| *e, "tr", |id, node| {
            node.text(id.to_string().as_str());
            node.div(|e| {
                if e.h1().clicked() {

                }
            });
            node.h1();
        });
    });
    let edom=(*edom).borrow_mut();
    let root=edom.root.as_ref().unwrap();
    assert_eq!("body", root.name);
    let Node::Element(table)=&root.children[1] else {panic!("No tbody found")};
    assert_eq!("tbody", table.name);
    let Node::ForEach(fe)=&table.children[0] else {panic!("No foreach found")};
    assert_eq!(3, fe.len());
    let tr=&fe[1].1;
    assert_eq!("tr", tr.name);
    let Node::Text(s, _)=&tr.children[0] else {panic!("No text found")};
    assert_eq!("5", s);
    let Node::Element(table_last_h1)=&tr.children[2] else {panic!("No table_div found")};
    assert!(table_last_h1.dnode.is_none());
}


#[test]
fn test_swap_rows() {
    let mut v:Vec<u64>=vec![1,2,3,4];
    let edom=EDOM::render(NoopElementNode {tag:"body"}, move |mut root| {
        let mut button=root.button("Swap rows");
        assert_eq!(1, button.element.uid);
        if button.clicked() {
            v.swap(1, 2);
            println!("Swapped rows, v={:?}", v);
        }
        let mut table=root.element("tbody");
        table.for_each(v.iter_mut(), |e| *e, "tr", |id, node| {
            node.text(id.to_string().as_str());
            node.div(|e| {
                if e.h1().clicked() {

                }
            });
            node.h1();
        });
    });
    let fire_event=(*edom).borrow_mut().fire_event.clone();
    fire_event.borrow_mut()(1, "click".to_string());
    let edom=(*edom).borrow_mut();
    let root=edom.root.as_ref().unwrap();
    let Node::Element(table)=&root.children[1] else {panic!("No table")};
    assert_eq!("tbody", table.name);
    let Node::ForEach(fe)=&table.children[0] else {panic!("No foreach found")};
    assert_eq!(3, fe[1].0);
    let Node::Text(s, _)=&fe[1].1.children[0] else {panic!("No text found")};
    assert_eq!("3", s);
}