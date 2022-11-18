use edom;
use wasm_bindgen::prelude::wasm_bindgen;

#[wasm_bindgen(start)]
pub fn demo() {
    let mut name = "Arthur".to_string();
    let mut age:f64 = 42.0;
    edom::wasm::render(move |mut root| {
        root.h1().text("My edom application");
        root.div(|div| {
            div.text("Your name: ");
            div.text_input(&mut name);
        });
        root.div(|div| {
            div.range_input(&mut age, 0.0, 120.0);
            div.number_input(&mut age).min(0.0).max(120.0);
            div.text("age");
        });
        if root.button("Click each year").clicked() {
            age+=1.0;
        }
        root.br();
        root.text(format!("Hello '{}', age {}", name, age).as_str());
    });
}