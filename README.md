# An immediate mode web frontend library written in Rust.

It builds up VDOM for not having to run too many DOM operations,
but as it runs every time any change is executed, it allows for a simple
programming model without message passing / callbacks / signals, just like EGUI.

The render function is called once for creating the initial web page, and then
twice for each event: 
- once for computing the side effects of the event
- once more for rendering the changes that happened by modifying the state (variables)

# A very simple program to illustrate usage
(in `examples/demo` directory):

# An immediate mode web frontend library written in Rust.

It builds up VDOM for not having to run too many DOM operations,
but as it runs every time any change is executed, it allows for a simple
programming model without message passing / callbacks / signals, just like EGUI.

The render function is called once for creating the initial web page, and then
twice for each event: 
- once for computing the side effects of the event
- once more for rendering the changes that happened by modifying the state (variables)

# A very simple program to illustrate usage
(in `examples/demo` directory):
