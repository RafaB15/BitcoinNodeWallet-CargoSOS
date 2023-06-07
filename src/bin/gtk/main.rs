use gtk::prelude::*;

fn main() {
    /* 
    if gtk::init().is_err() {
        println!("Failed to initialize GTK.");
        return;
    }

    let glade_src = include_str!("MainWindow.glade");
    let builder = gtk::Builder::from_string(glade_src);

    let window: gtk::Window = builder.object("MainWindow").unwrap();
    let grid: gtk::Grid = builder.object("Grid").unwrap();
    let button1: gtk::Button = builder.object("button1").unwrap();
    let button2: gtk::Button = builder.object("button2").unwrap();
    let button3: gtk::Button = builder.object("button3").unwrap();
    let button4: gtk::Button = builder.object("button4").unwrap();

    window.show_all();

    gtk::main();
    */

    if gtk::init().is_err() {
        println!("Failed to initialize GTK.");
        return;
    }

    let glade_src = include_str!("WindowNotebook.glade");
    let builder = gtk::Builder::from_string(glade_src);

    let window: gtk::Window = builder.object("MainWindow").unwrap();
    //let notebook: gtk::Notebook = builder.object("Notebook").unwrap();

    window.show_all();

    gtk::main();
}