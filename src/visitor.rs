use std::collections::{HashMap, HashSet};
use std::rc::Rc;

use wasm_bindgen::convert::IntoWasmAbi;

use crate::dom::{Document, GenericNode, ElementNode};
use crate::vdom::{RenderIfState, CachedValue};

use super::EDOM;
use super::vdom::{Element,Node};
use super::dom;

pub struct Visitor<'d, 'e, EN> where EN:dom::ElementNode {
    pub edom: &'d mut EDOM<EN>,
    pub element: &'e mut Element<EN>,
    pub attrpos: usize,
    pub childpos: usize,
    pub eventpos: usize,
    pub parent_access_pos: usize,
    pub parent_iterator: Option<*const Visitor<'d, 'd, EN>>,
    pub next_dom_child_pos: usize
}

impl<'d, 'e, 'f, 'a, 'z, 'c, 'q, EN> Visitor<'d, 'e, EN> where EN:dom::ElementNode {
    fn create_element_iterator(&'f mut self, child_pos: usize)->Visitor<'f, 'f, EN> {
        let cself : *const Visitor<EN>=self;
        let Node::Element(element)=&mut self.element.children[child_pos] else {
            panic!("Not Element")
        };
        Visitor::new(self.edom, element, child_pos, Some(cself))
    }
    fn create_element(&mut self, name : &'static str)->Element<EN> {
        Element::new(name, Some(self.edom.document.create_element(name)), self.edom.next_uid())
    }
    pub fn element(&'f mut self, name : &'static str)->Visitor<'f, 'f, EN> {
        let new_pos= if self.edom.create {
            let elem= self.create_element(name);
            self.get_dnode().append_child(&elem.dnode.get(||panic!("Dnode empty")));
            let i = self.element.children.len();
            self.element.children.push(Node::Element(elem));
            i
        } else {
            let i=self.childpos;
            self.childpos+=1;
            i
        };
        self.next_dom_child_pos+=1;
        self.create_element_iterator(new_pos)
    }
    fn get_render_if_element_at(&self, child_pos: usize)->&Element<EN> {
        let Node::RenderIfElement(_, e)=&self.element.children[child_pos] else { panic!("Not RenderIfElement")};
        e
    }
    fn insert_after_last_visible_dnode(&self, child: &EN::GenericNode) {
        self.get_dnode().insert_child_before(
            child,
            self.get_dnode().get_child_node(self.next_dom_child_pos as u32).as_ref());
    }

    pub fn render_element_if<FCB>(&'f mut self, should_render: bool, tag: &'static str, mut fcb: FCB) where FCB:FnMut(&mut Visitor<EN>) {
        let cself : *const Visitor<EN>=self;
        let new_pos= if self.edom.create {
            let i = self.element.children.len();
            self.element.children.push(Node::RenderIfElement(RenderIfState::NotRendered, Element { name: tag, attr: Vec::new(), children: Vec::new(), dnode: CachedValue::new(None), events: Vec::new(), uid: 999999 }));
            i
        } else {
            let i=self.childpos;
            self.childpos+=1;
            i
        };
        let Node::RenderIfElement(state, _)=&self.element.children[new_pos] else { panic!("Not RenderIfElement")};
        match state.clone() {
            RenderIfState::Hidden => {
                if should_render {
                    self.insert_after_last_visible_dnode(
                        self.get_render_if_element_at(new_pos).dnode.unwrap().into_generic_node());
                        self.element.children[new_pos].set_render_if_state(RenderIfState::Visible);
                }
            },
            RenderIfState::NotRendered => {
                if should_render {
                    // Need to create new element
                    let mut elem=self.create_element(tag);
                    let create=self.edom.create;
                    self.edom.create=true;
                    let mut it=Visitor::new(&mut self.edom, 
                        &mut elem, new_pos, Some(cself));
                    fcb(&mut it);
                    self.edom.create=create;
                    self.insert_after_last_visible_dnode(elem.dnode.unwrap().into_generic_node());
                    self.element.children[new_pos].set_render_if_state(RenderIfState::Visible);
                }
            },
            RenderIfState::Visible => {
                if !should_render {
                    self.get_dnode().remove_child(
                        self.get_render_if_element_at(new_pos).dnode.unwrap());
                    self.element.children[new_pos].set_render_if_state(RenderIfState::Hidden);
                }
            }
        }
        if should_render {
            self.next_dom_child_pos+=1;
        }
    }
    
    pub fn new(edom:&'d mut EDOM<EN>, element:&'e mut Element<EN>, parent_access_pos: usize, parent_iterator: Option<*const Visitor<'d ,'d,EN>>)->Visitor<'d,'e,EN> {
         Visitor {edom, element, attrpos: 0, childpos: 0, eventpos: 0, parent_access_pos, parent_iterator, next_dom_child_pos: 0}
     }
    fn get_dnode_using_parameters(dnode: &'a CachedValue<EN>, parent_iterator: &Option<*const Visitor<EN>>, parent_access_pos: usize)->&'a EN {
        dnode.get(|| unsafe {&**parent_iterator.as_ref().unwrap()}.get_dnode().get_child_node(parent_access_pos as u32).unwrap().into_element_node())
    }
    pub fn get_dnode(&self)->&EN {
        Self::get_dnode_using_parameters(&self.element.dnode, &self.parent_iterator, self.parent_access_pos)
        // self.element.dnode.get(|| unsafe {(&**self.parent_iterator.as_ref().unwrap())}.get_dnode().get_child_node(self.parent_access_pos as u32).into_element_node())
    }
    
    pub fn attr(&'f mut self, name: &'static str, value: &str)->&'f mut Visitor<'d,'e,EN> {
        if self.edom.create {
            self.get_dnode().set_attribute(name, value);
            self.element.attr.push((name, Rc::new(value.into())));
        } else { 
            let thisattr=&mut self.element.attr[self.attrpos];
            if thisattr.0 != name {
                panic!("name change")
            }
            if *thisattr.1 != value {
                thisattr.1=Rc::new(value.into());
                // thisattr.1=value.into();
                self.get_dnode().set_attribute(name, value);
            }
            self.attrpos+=1
        }
        self
    }
    
    pub fn text(&mut self, text:&str) {
        if self.edom.create {
            if self.element.children.len()==0 {
                self.get_dnode().set_text_content(text);
                self.element.children.push(Node::Text(text.into(), None));
            } else {
                let tdnode=self.edom.document.create_text_node(text);
                self.get_dnode().append_text_child(&tdnode);
                let elem=Node::Text(text.into(), Some(tdnode));
                self.element.children.push(elem);
            }
        } else {
            let elem = &mut self.element.children[self.childpos];
            let Node::Text(text2, _)=elem else {
                panic!("Not text");
            };
            if *text != **text2 {
                Self::get_dnode_using_parameters(
                    &self.element.dnode, &self.parent_iterator, self.parent_access_pos);
                *text2=text.into();
                self.update_text_content_for_current_child();
            }
            self.childpos+=1;
        }
        self.next_dom_child_pos+=1;
    }

    fn update_text_content_for_current_child(&mut self) {
        let n=self.element.children.len();
        let dnode=self.element.dnode.unwrap();
        let elem = &mut self.element.children[self.childpos];
        let Node::Text(text, text_dnode)=elem else {
            panic!("No text child found");
        };

        if n==1 {
            *text_dnode=None;
            dnode.set_text_content(text);
        } else {
            let new_child=self.edom.document.create_text_node(text);
            if text_dnode.is_none() {
                *text_dnode=Some(dnode.get_child_node(self.childpos as u32).unwrap().into_text_node());
            }
            dnode.replace_text_child(&new_child, text_dnode.as_ref().unwrap());
            *text_dnode=Some(new_child);
        }
    }

    pub fn on<F>(&'f mut self, name:&'static str, mut f: F)->&'f mut Self where F:FnMut(&EN::Event) {
        if self.edom.create {
            self.element.events.push(name);
            self.element.create_event_listener(name, self.edom, self.get_dnode());
        } else if let Some(ev) = &self.edom.firing_event  {
            if self.element.uid == ev.0 {
                if *self.element.events[self.eventpos]==*ev.1  {
                    f(&self.edom.firing_event.as_ref().unwrap().2);
                }
                self.eventpos+=1;
            }
        }
        self
    }

    fn position_by_idx(v:&mut Vec<(u64, Element<EN>)>)->HashMap<u64, usize> {
        let mut position:HashMap<u64, usize>=HashMap::new();
        for (i, (idx, _)) in &mut v.iter().enumerate() {
            position.insert(*idx, i);
        }
        position
    }

    fn for_each_consolidate_changes<'g, TIdx: std::hash::Hash, FIdx, FCB, I,
            L: Iterator<Item=I>>(
            &'f mut self, list : L, mut fidx: FIdx, tag: &'static str, mut fcb: FCB) 
            where FIdx:FnMut(&I)->TIdx, FCB:FnMut(I, &mut Visitor<EN>),
            TIdx: std::cmp::Eq {
        let self_ptr : *mut Visitor<EN>=self;
        self.get_dnode();
        let dnode = self.element.dnode.unwrap();
        let Node::ForEach(v) : &mut Node<EN>=&mut self.element.children[self.childpos] else {
            panic!("Bad node");
        };

        // position[idx]+relpos will contain the position of idx Element for all shown elements in v[ii] where ii>=i.
        let mut position=Self::position_by_idx(v);
        let mut relpos: isize=0;
        let mut wrong_place: HashSet<u64>=HashSet::new();
        let mut edom : &mut EDOM<EN>=&mut self.edom;
        let mut i=0;

        for e in list {
            let mut hasher= std::collections::hash_map::DefaultHasher::new();
            fidx(&e).hash(&mut hasher);
            let idx=hasher.finish();
            if let Some(pos)=position.get(&idx).cloned() {
                let abspos=add_offset(pos, relpos).unwrap();
                if abspos < i {
                    panic!("Index is multiple times in the list probably: abspos={}, i={}, relpos={}, wrong_place={:?}, position={:?}, idx={}", abspos, i, relpos, wrong_place, position, idx);
                }
                if abspos==i+1 { // Special case, put current element at end of list, mark as removed
                    let new_idx=v[i].0;
                    wrong_place.insert(new_idx);
                    v[i..].rotate_left(1);
                    assert_eq!(new_idx, v.last().unwrap().0);
                    relpos-=1;
                    position.insert(new_idx, add_offset(v.len()-1, -relpos).unwrap());
                } else if abspos!=i {
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
                        // TODO: get last visited child to support for_each after other elements
                        dnode.prepend_child(&v[i].1.dnode.unwrap());
                    } else {
                        dnode.append_child_after(&v[i].1.dnode.unwrap(),
                        &v[i-1].1.dnode.unwrap())
                    }
                }
                let mut it : Visitor<EN>=Visitor::new(
                    edom, &mut v[i].1, self.next_dom_child_pos, Some(self_ptr));
                fcb(e, &mut it);
                edom=it.edom;
            } else {
                let last_elem=if i>0 {Some(&v[0].1)} else {None};
                let elem=Self::create_for_each_element(
                    e, edom, self.next_dom_child_pos,
                    &mut fcb, self_ptr, 
                    tag, last_elem);

                v.insert(i, (idx, elem));
                relpos+=1;
                    
                if i+1 == v.len() {
                    dnode.append_child(&v[i].1.dnode.unwrap());
                } else {
                    dnode.insert_child_before(&v[i].1.dnode.unwrap().into_generic_node(), 
                        Some(&v[i+1].1.dnode.unwrap().into_generic_node()));  // Not sure that this is the next sibling
                }
            }
            i+=1;
            self.next_dom_child_pos+=1;
        }

        while v.len() > i {  // Remove remaining children
            v.last().unwrap().1.dnode.unwrap().remove();
            v.pop();
        }

        self.childpos+=1;
    }

    fn create_for_each_element<'x, FCB, I>(
            item: I, mut edom: &'x mut EDOM<EN>, parent_access_pos: usize,
            mut fcb: FCB, self_ptr: *const Visitor<'x, 'x, EN>, 
            tag: &'static str, last_elem: Option<&Element<EN>>)->Element<EN> 
            where FCB:FnMut(I, &mut Visitor<EN>)  {
        // Create new DOM or clone.
        let mut element:Element<EN>;
        if last_elem.is_none() || !edom.clone_for_each {
            let create=edom.create;
            edom.create=true;
            element=Element::new(tag, Some(edom.document.create_element(tag)), edom.next_uid());
            let mut it:Visitor<EN>=Visitor::new(edom, &mut element, parent_access_pos, Some(self_ptr));
            fcb(item, &mut it);
            edom=it.edom;
            edom.create=create;
        } else {
            let new_dnode=last_elem.as_ref().unwrap().dnode.unwrap().deep_clone();
            let create=edom.create;
            edom.create=false;
            if edom.use_partial_clone {
                element=last_elem.unwrap().shallow_clone(Some(new_dnode), edom);
                let it=Visitor::new(edom, &mut element, parent_access_pos, None);
                last_elem.unwrap().partial_clone_using_dnode(it);
            } else {
                element=last_elem.unwrap().clone_using_dnode(new_dnode, edom);
            }
            let mut it:Visitor<EN>=Visitor::new(edom, &mut element, parent_access_pos, Some(self_ptr));
            fcb(item, &mut it);
            edom=it.edom;
            edom.create=create;
        };
        element
    }

    pub fn create_for_each<FIdx, FCB, I, L: Iterator<Item=I>, TIdx : Hash>(&mut self, list : L, 
            mut fidx: FIdx, tag: &'static str, mut fcb: FCB)
            where FIdx:FnMut(&I)->TIdx, FCB:FnMut(I, &mut Visitor<EN>) {
        let self_ptr : *mut Visitor<EN>=self;
       
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
            let mut hasher= std::collections::hash_map::DefaultHasher::new();
            fidx(&l).hash(&mut hasher);
            let idx=hasher.finish();
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
            self.next_dom_child_pos+=1;
        }
    }

    pub fn for_each<FIdx, TIdx : std::cmp::Eq, FCB, I, L: Iterator<Item=I>>(&'f mut self, list : L,
            fidx: FIdx, tag: &'static str, fcb: FCB)
            where FIdx:FnMut(&I)->TIdx, FCB:FnMut(I, &mut Visitor<EN>),
            TIdx : Hash {
        if self.edom.create {
            self.create_for_each(list, fidx, tag, fcb);
        } else {
            self.for_each_consolidate_changes(list, fidx, tag, fcb);
        }
    }
}

fn add_offset(big_indexer: usize, delta: isize) -> Option<usize> {
    if delta < 0 {
        big_indexer.checked_sub(delta.wrapping_abs() as usize)
    } else {
        big_indexer.checked_add(delta as usize)
    }
}
use std::hash::{Hash, Hasher};
