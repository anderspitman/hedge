#![no_std]

extern crate alloc;

use alloc::string::{String,ToString};
//use alloc::vec;
use alloc::vec::Vec;

use serde::{Deserialize, Serialize};

#[derive(Debug,Serialize,Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum InMessage {
    WidgetPressed {
        path: String,
        name: Option<String>,
    },
    TextChanged {
        path: String,
        text: String,
    },
    //FolderOpened {
    //    path: String,
    //},
}

#[derive(Debug,Serialize,Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum OutMessage {
    SetTree {
        path: String,
        tree: Widget,
    },
    OpenFolder,
}

#[derive(Debug,Clone,Serialize,Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum Widget {
    Container {
        children: Vec<Widget>,
    },
    Row {
        children: Vec<Widget>,
    },
    Column {
        children: Vec<Widget>,
    },
    Textbox {
        text: String,
    },
    Button {
        text: String,
        name: Option<String>,
    },
    Label {
        text: String,
    }
}

pub struct ButtonBuilder {
    text: String,
    name: Option<String>,
}

impl ButtonBuilder {
    pub fn new() -> Self {
        Self{
            text: String::new(),
            name: None,
        }
    }

    pub fn text(mut self, text: &str) -> Self {
        self.text = text.to_string();
        self
    }

    pub fn name(mut self, name: &str) -> Self {
        self.name = Some(name.to_string());
        self
    }

    pub fn build(self) -> Widget {
        Widget::Button{
            text: self.text,
            name: self.name,
        }
    }
}
