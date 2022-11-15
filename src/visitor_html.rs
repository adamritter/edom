use std::rc::Rc;

use super::dom;
use super::visitor::Visitor;

impl<'d, 'e, 'f, 'a, 'z, 'c, 'q, EN> Visitor<'d, 'e, EN> where EN:dom::ElementNode {
    pub fn button(&'f mut self, text: &str)->Visitor<'f,'f,EN> {
        let mut elem=self.element("button");
        elem.text(text);
        elem
    }
    pub fn div<FCB:FnMut(&mut Visitor<EN>)>(
            &'f mut self, mut fcb: FCB)->Visitor<'f,'f,EN>  {
        let mut r=self.element("div");
        fcb(&mut r);
        return r;
    }
    pub fn ul<FCB:FnMut(&mut Visitor<EN>)>(
        &'f mut self, mut fcb: FCB)->Visitor<'f,'f,EN>  {
        let mut r=self.element("ul");
        fcb(&mut r);
        return r;
    }
    pub fn span<FCB:FnMut(&mut Visitor<EN>)>(
            &'f mut self, mut fcb: FCB)->Visitor<'f,'f,EN> {
        let mut r=self.element("span");
        fcb(&mut r);
        return r;
    }

    pub fn li<FCB:FnMut(&mut Visitor<EN>)>(
        &'f mut self, mut fcb: FCB)->Visitor<'f,'f,EN> {
        let mut r=self.element("li");
        fcb(&mut r);
        return r;
    }
    pub fn header<FCB:FnMut(&mut Visitor<EN>)>(&
            'f mut self, mut fcb: FCB)->Visitor<'f,'f,EN> {
        let mut r=self.element("header");
        fcb(&mut r);
        return r;
    }
    pub fn footer<FCB:FnMut(&mut Visitor<EN>)>(
            &'f mut self, mut fcb: FCB)->Visitor<'f,'f,EN> {
        let mut r=self.element("footer");
        fcb(&mut r);
        return r;
    }


    pub fn placeholder(&'f mut self, text: &str)->&'f mut Self {
        self.attr("placeholder", text)
    }
    pub fn autofocus(&'f mut self, value: bool)->&'f mut Self {
        self.attr("autofocus", value.to_string().as_str())
    }
    pub fn h1(&'f mut self)->Visitor<'f,'f,EN> {
        self.element("h1")
    }
    pub fn strong(&'f mut self)->Visitor<'f,'f,EN> {
        self.element("strong")
    }
    pub fn a(&'f mut self, text: &str)->Visitor<'f, 'f, EN> {
        let mut r=self.element("a");
        r.attr("href", text);
        r
    }
    pub fn label(&'f mut self, for_: &str, text: &str)->Visitor<'f, 'f, EN> {
        let mut r=self.element("a");
        r.attr("for", for_);
        r.text(text);
        r
    }
    fn f64_input(&'f mut self, value: &mut f64)->Visitor<'f,'f,EN> {
        let mut r=self.element("input");
        r.attr("value", value.to_string().as_str());
        if r.changed() {
            *value=r.get_dnode().get_attribute("value").parse::<f64>().unwrap();
            r.element.attr[0]=("value", Rc::new(value.to_string()));
        }
        r
    }
    pub fn number_input(&'f mut self, value: &mut f64)->Visitor<'f,'f,EN> {
        let mut r=self.f64_input(value);
        r.attr("type", "number");
        r
    }
    pub fn range_input(&'f mut self, value: &mut f64, min: f64, max: f64)->Visitor<'f,'f,EN> {
        let mut r=self.f64_input(value);
        r.attr("type", "range");
        r.attr("min", min.to_string().as_str());
        r.attr("max", max.to_string().as_str());
        r
    }
    pub fn checkbox(&'f mut self, checked: &mut bool)->Visitor<'f,'f,EN> {
        let mut cb=self.element("input");
        // checked attribute is treated specially in dnode.set_attribute
        cb.attr("checked", checked.to_string().as_str());
        if cb.changed() {
            *checked=!*checked;
            cb.element.attr[0]=("checked", Rc::new(checked.to_string()));
        }
        cb.attr("type", "checkbox");
        cb
    }
    pub fn radio_input(&'f mut self, name: &str, value: &str, checked: &mut bool)->Visitor<'f,'f,EN> {
        let mut cb=self.element("input");
        // checked attribute is treated specially in dnode.set_attribute
        cb.attr("checked", checked.to_string().as_str());
        if cb.changed() {
            *checked=!*checked;
            cb.element.attr[0]=("checked", Rc::new(checked.to_string()));
        }
        cb.attr("type", "radio").attr("name", name).attr("value", value);
        cb
    }

    pub fn text_input(&'f mut self, value: &mut String)->Visitor<'f,'f,EN> {
        let mut r=self.element("input");
        r.attr("value", value.as_str());
        if r.changed() {
            *value=r.get_dnode().get_attribute("value");
            r.element.attr[0]=("value", Rc::new(value.to_string()));
        }
        r.attr("type", "text");
        r
    }
    pub fn texttextarea(&'f mut self, value: &mut String)->Visitor<'f,'f,EN> {
        let mut r=self.element("textarea");
        r.attr("value", value.as_str());
        if r.changed() {
            *value=r.get_dnode().get_attribute("value");
            r.element.attr[0]=("value", Rc::new(value.to_string()));
        }
        r
    }
    pub fn changed(&mut self)->bool {
        let mut changed=false;
        self.on("change", |_| { changed=true; });
        changed
    }
    pub fn id(&'f mut self, id: &str)->&'f mut Visitor<'d,'e,EN> {
        self.attr("id", id)
    }
    pub fn class(&'f mut self, id: &str)->&'f mut Self {
        self.attr("class", id)
    }
    pub fn classes(&'f mut self, data: &[(&str, bool)])->&'f mut Self {
        let mut class=String::new();
        for (s, enabled) in data {
            if *enabled {
                if class.len() > 0 {
                    class.push(' ');
                }
                class.push_str(s);
            }
        }
        self.class(class.as_str())
    }
   
    pub fn click<F>(&'c mut self, f:F)->&'c mut Self where F:FnMut(&EN::Event) {
        self.on("click", f)
    }
    pub fn clicked(&'c mut self)->bool {
        let mut r=false;
        self.on("click", |_| r=true);
        r
    }
    pub fn double_clicked(&'c mut self)->bool {
        let mut r=false;
        self.on("doubleclick", |_| r=true);
        r
    }
}