
extern crate console_error_panic_hook;
extern crate smallstr;

pub use dom::{EventHandler, Document};

pub mod dom;
pub mod wasm;
pub mod vdom;
use std::rc::Rc;
use std::cell::RefCell;
pub mod visitor;
pub use visitor::Visitor;
pub use dom::ElementNode;

pub struct EDOM<EN> where EN:dom::ElementNode {
    firing_event: Option<(u64, String, EN::Event)>,
    last_uid: u64,
    create: bool,
    document: EN::Document,
    pub fire_event: Rc<RefCell<Box<dyn FnMut(u64, String, EN::Event)>>>,
    clone_for_each: bool,  // Clone node for for_each instead of building up the DOM tree.
    nodes_attached: u64,
    use_partial_clone: bool,
    root: Option<vdom::Element<EN>>,
    event_handler: EN::EventHandler
}

impl<EN> EDOM<EN> where EN:dom::ElementNode {
    fn next_uid(&mut self)->u64 {
        let r=self.last_uid;
        self.last_uid+=1;
        return r;
    }
    
    fn render_once<F>(&mut self, mut f:F) where EN:dom::ElementNode, F:FnMut(Visitor<EN>) {
        let mut root=self.root.take();
        let ei=Visitor::new(self, root.as_mut().unwrap(), 0, None);
        f(ei);
        self.root=root;
        self.create=false;
    }

    pub fn render<F>(root: EN, mut f:F)->Rc<RefCell<EDOM<EN>>>
            where EN:dom::ElementNode + 'static, F:FnMut(Visitor<EN>) + 'static {
        let el=vdom::Element::new("body", Some(root), 0);
        let mut edom : EDOM<EN>=EDOM::new(el);
        assert_eq!(0, edom.next_uid());
        edom.create=true;
        edom.render_once(&mut f);

        let fire_event=edom.fire_event.clone();
        let edomrc : Rc<RefCell<EDOM<EN>>>=Rc::new(RefCell::new(edom));
        let moved_edomrc=edomrc.clone();
        *fire_event.borrow_mut()=Box::new(move |a:u64, b:String, e:EN::Event| {
            // EN::Document::log_2(a.to_string().as_str(), b.as_str());
            let mut edom=moved_edomrc.borrow_mut();
            edom.nodes_attached=0;
            edom.firing_event=Some((a, b, e));
            edom.render_once(&mut f);
            edom.firing_event=None;
            edom.render_once(&mut f);
            // EN::Document::log_2("Nodes attached in render:", edom.nodes_attached.to_string().as_str());
        });
        std::mem::forget(fire_event);
        edomrc
   }

    fn new(root: vdom::Element<EN>)->Self {
        let fire_event:Rc<RefCell<Box<dyn FnMut(u64, String, EN::Event)>>>=Rc::new(RefCell::new(
                Box::new(|a:u64, b:String, _:EN::Event|
                    web_sys::console::log_3(&"rc".into(), &a.to_string().into(), 
                    &b.into()))));
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

pub mod noop;
pub mod visitor_html;

#[test]
fn test_create() {
    EDOM::render( noop::ElementNode {tag:"body", generic_node: noop::Node {  }}, move |mut root| {
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
    let nobody=noop::ElementNode {tag:"body", generic_node: noop::Node {  }};
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

        table.for_each(v.iter_mut(), |e| **e, "tr", |id, node| {
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
    let vdom::Node::Element(table)=&root.children[1] else {panic!("No tbody found")};
    assert_eq!("tbody", table.name);
    let vdom::Node::ForEach(fe)=&table.children[0] else {panic!("No foreach found")};
    assert_eq!(3, fe.len());
    let tr=&fe[1].1;
    assert_eq!("tr", tr.name);
    let vdom::Node::Text(s, _)=&tr.children[0] else {panic!("No text found")};
    assert_eq!("5", s.as_str());
    let vdom::Node::Element(table_last_h1)=&tr.children[2] else {panic!("No table_div found")};
    assert!(table_last_h1.dnode.is_none());
}


#[test]
fn test_swap_rows() {
    let mut v:Vec<u64>=vec![1,2,3,4];
    let edom=EDOM::render(noop::ElementNode {tag:"body", generic_node: noop::Node {  }}, move |mut root| {
        let mut button=root.button("Swap rows");
        assert_eq!(1, button.element.uid);
        if button.clicked() {
            v.swap(1, 2);
            println!("Swapped rows, v={:?}", v);
        }
        let mut table=root.element("tbody");
        table.for_each(v.iter_mut(), |e| **e, "tr", |id, node| {
            assert_eq!(0, node.next_dom_child_pos);
            node.text(id.to_string().as_str());
            assert_eq!(1, node.next_dom_child_pos);
            node.div(|e| {
                if e.h1().clicked() {

                }
            });
            assert_eq!(2, node.next_dom_child_pos);
            node.h1();
            assert_eq!(3, node.next_dom_child_pos);
            node.render_element_if(false, "span", |_| ());
            assert_eq!(3, node.next_dom_child_pos);
            node.render_element_if(true, "span", |_| ());
            assert_eq!(4, node.next_dom_child_pos);
        });
    });
    let fire_event=(*edom).borrow_mut().fire_event.clone();
    fire_event.borrow_mut()(1, "click".to_string(), noop::Event {});
    let edom=(*edom).borrow_mut();
    let root=edom.root.as_ref().unwrap();
    let vdom::Node::Element(table)=&root.children[1] else {panic!("No table")};
    assert_eq!("tbody", table.name);
    let vdom::Node::ForEach(fe)=&table.children[0] else {panic!("No foreach found")};
    let vdom::Node::Text(s, _)=&fe[1].1.children[0] else {panic!("No text found")};
    assert_eq!("3", s.as_str());
}


#[test]
fn test_remove_row() {
    let mut v:Vec<u64>=vec![1,2,3,4,5,6,7,8];
    let edom=EDOM::render(noop::ElementNode {tag:"body", generic_node: noop::Node {  }}, move |mut root| {
        let mut button=root.button("Swap rows");
        assert_eq!(1, button.element.uid);
        if button.clicked() {
            v.remove(0);
            assert_eq!(vec![2,3,4,5,6,7,8], v);
        }
        let mut table=root.element("tbody");
        table.for_each(v.iter_mut(), |e| **e, "tr", |id, node| {
            node.text(id.to_string().as_str());
        });
    });
    let fire_event=(*edom).borrow_mut().fire_event.clone();
    fire_event.borrow_mut()(1, "click".to_string(), noop::Event {});
    let edom=(*edom).borrow_mut();
    let root=edom.root.as_ref().unwrap();
    let vdom::Node::Element(table)=&root.children[1] else {panic!("No table")};
    assert_eq!("tbody", table.name);
    let vdom::Node::ForEach(fe)=&table.children[0] else {panic!("No foreach found")};
    let vdom::Node::Text(s, _)=&fe[1].1.children[0] else {panic!("No text found")};
    assert_eq!("3", s.as_str());
}