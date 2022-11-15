use std::rc::Rc;

use crate::dom::EventHandler;
use crate::dom::GenericNode;

use super::dom;
use super::EDOM;
use super::visitor::Visitor;
pub enum Node<EN> where EN:dom::ElementNode {
    Text(String, Option<EN::TextNode>),
    Element(Element<EN>),
    ForEach(Vec<(u64, Element<EN>)>),
    RenderIfElement(RenderIfState, Element<EN>)
}

#[derive(Clone, PartialEq)]
pub enum RenderIfState {
    NotRendered,
    Hidden,
    Visible
}

impl<EN:dom::ElementNode> Node<EN> {
    pub fn set_render_if_state(&mut self, state: RenderIfState) {
        let Node::RenderIfElement(state2, _)=self else { panic!("Not RenderIfElement")};
        *state2=state;
    }
}

pub struct CachedValue<T> {
    value: std::cell::UnsafeCell<Option<T>>
}

impl<T> CachedValue<T> {
    pub fn new(v:Option<T>)->Self {
        Self {value: std::cell::UnsafeCell::new(v)}
    }
    pub fn get<'a,F>(&self, f : F)->&'a T where F:FnOnce()->T {
        let v=unsafe {&mut *self.value.get()};
        if v.is_none() {
            *v=Some(f());
        }
        return v.as_ref().unwrap();
    }
    pub fn is_none(&self)->bool {
        unsafe {&mut *self.value.get()}.is_none()
    }
    pub fn unwrap<'a>(&self)->&'a T {
        let v=unsafe {&mut *self.value.get()};
        if v.is_none() {
            panic!("Couldn't get cached value");
        }
        return v.as_ref().unwrap();
    }
}

pub struct Element<EN> where EN:dom::ElementNode {
    pub name: &'static str,
    pub attr: Vec<(&'static str,String)>,
    pub children: Vec<Node<EN>>,
    pub dnode: CachedValue<EN>,
    pub events: Vec<&'static str>,
    pub uid: u64,
}

impl<EN> Element<EN>  where EN:dom::ElementNode {
    pub fn new(name: &'static str, dnode: Option<EN>, uid: u64)->Self {
        Self {name, attr:vec![], children: vec![], dnode: CachedValue::new(dnode), events: Vec::new(), uid}
    }
    pub fn create_event_listener(&self, name: &'static str, edom: &EDOM<EN>, dnode: &EN) {
        dnode.set_attribute("data-uid", self.uid.to_string().as_str());
        edom.event_handler.create_event_listener(dnode, name.to_string());
        // dnode.create_dnode_event_listener(edom.fire_event.clone(), self.uid, name);
    }
    pub fn clone_using_dnode(&self, target_dnode: EN, edom: &mut EDOM<EN>)->Self {
        let mut r = Self {name: self.name, attr: self.attr.clone(), children: Vec::new(), dnode: CachedValue::new(Some(target_dnode)), events: self.events.clone(), uid: edom.next_uid()};
        let rdnode=r.dnode.get(|| panic!("Should exist"));

        for event_name in &self.events {
            self.create_event_listener(event_name, edom, rdnode);
        }

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

    pub fn shallow_clone(&self, target_dnode: Option<EN>, edom: &mut EDOM<EN>)->Self {
        Self {name: self.name, attr: self.attr.clone(), children: Vec::new(), dnode: CachedValue::new(target_dnode), events: self.events.clone(), uid: edom.next_uid()}
    }

    pub fn partial_clone_using_dnode(&self, target_iterator: Visitor<EN>) {
        // Attach events
        for event_name in &self.events {
            target_iterator.element.create_event_listener(event_name, target_iterator.edom, target_iterator.get_dnode());
        }
        // Attach children
        let mut next_dom_node=0;
        for child in &self.children {
            let mut skip=false;
            let new_elem=match child {
                Node::Text(s, _)=>Node::Text(s.clone(), None),
                Node::Element(e)=> {
                    let mut new_elem=e.shallow_clone(None, target_iterator.edom);
                    let ctarget_iterator : *const Visitor<EN>=&target_iterator;
                    let it=Visitor::new(
                        target_iterator.edom, &mut new_elem, next_dom_node, Some(ctarget_iterator));
                    e.partial_clone_using_dnode(it);
                    Node::Element(new_elem)
                },
                Node::RenderIfElement(state, e)=> {
                    if *state == RenderIfState::Visible {
                        let mut new_elem=e.shallow_clone(None, target_iterator.edom);
                        let ctarget_iterator : *const Visitor<EN>=&target_iterator;
                        let it=Visitor::new(
                            target_iterator.edom, &mut new_elem, next_dom_node, Some(ctarget_iterator));
                        e.partial_clone_using_dnode(it);
                        Node::RenderIfElement(RenderIfState::Visible, new_elem)
                    } else {
                        skip=true;
                        Node::RenderIfElement(RenderIfState::NotRendered, Element::new("", None, 999999))
                    }
                }
                _ => {
                    panic!("Only cloning element and text is supported so far");
                }
            };
            target_iterator.element.children.push(new_elem);
            if !skip {
                next_dom_node+=1;
            }
        }
    }
    
}
