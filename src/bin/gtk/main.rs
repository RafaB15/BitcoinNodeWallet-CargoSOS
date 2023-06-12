use gtk::{prelude::*, glib::Object, Button, Entry, Application, Builder, Window};


pub trait VecOwnExt {
    fn search_by_name(&self, name: &str) -> Object;
    fn search_window_named(&self, name: &str) -> Window;
    fn search_button_named(&self, name: &str) -> Button;
    fn search_entry_named(&self, name: &str) -> Entry;
    //fn search_radio_button_named(&self, name: &str) -> RadioButton;

}

pub trait ObjectOwnExt {
    fn is_named(&self, name: &str) -> bool;
}

impl ObjectOwnExt for Object {
    fn is_named(&self, name: &str) -> bool {
        self.property_value("name").get::<String>().unwrap() == *name
    }
}

impl VecOwnExt for Vec<Object> {

    fn search_by_name(&self, name: &str) -> Object {
        let found = self.iter().find(|&object| object.is_named(name));
        if let Some(found) = found {
            (*found).clone()
        } else {
            println!("Todo para el orto che {name}");
            (*found.unwrap()).clone()
        }
    }

    fn search_button_named(&self, name: &str) -> Button {
        self.search_by_name(name)
            .downcast_ref::<gtk::Button>()
            .unwrap()
            .clone()
    }
    fn search_entry_named(&self, name: &str) -> Entry {
        self.search_by_name(name)
            .downcast_ref::<gtk::Entry>()
            .unwrap()
            .clone()
    }

    fn search_window_named(&self, name: &str) -> Window {
        self.search_by_name(name)
            .downcast_ref::<gtk::Window>()
            .unwrap()
            .clone()
    }
}

fn build_ui(application: &gtk::Application, glade_src: &str) {
    let builder: Builder = Builder::from_string(glade_src);

    let objects = builder.objects();
    let window = objects.search_window_named("MainWindow");
    window.set_application(Some(application));

    let send_button = objects.search_button_named("send_button");
    let obj_cl = objects.clone();
    send_button.connect_clicked(move |_| {
        let entry_address = obj_cl.search_entry_named("address_entry");
        let entry_amount = obj_cl.search_entry_named("amount_entry");
        let entry_label = obj_cl.search_entry_named("label_entry");

        

        println!("{:?} {:?} {:?}", entry_address.text(), entry_amount.text(), entry_label.text());
        entry_address.set_text("");
        entry_amount.set_text("");
        entry_label.set_text("");
    });

    window.show_all();

}

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

    let application = Application::builder().build();

    application.connect_activate(move |app| build_ui(app, glade_src));
    application.run();
}