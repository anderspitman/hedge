use std::fs::File;
use std::io::Write;
use std::collections::HashMap;
use wasmtime::{Caller, Engine, Linker, Module, Store, Memory, TypedFunc, Extern};
use hedge::{InMessage,OutMessage};

pub struct App {
    pub store: Store<CallerState>,
    pub memory: Memory,
    pub input_message_buffer_offset: usize,
    pub output_message_buffer_offset: usize,
    pub eri_update: TypedFunc<(), ()>,
}

#[repr(u32)]
enum InMessageType {
    WidgetPressed = 0x40,
    TextChanged,
}

#[repr(u32)]
enum OutMessageType {
    SetTree = 0x10,
    Unknown,
}

#[repr(u32)]
enum MessageAttrType {
    Name = 0x50,
    Path,
    Text,
}

impl From <u32> for OutMessageType {
    fn from(val: u32) -> Self {
        match val {
            0x10 => OutMessageType::SetTree,
            _ => OutMessageType::Unknown,
        }
    }
}

#[repr(u32)]
enum WidgetType {
    Container = 0x20,
    Button,
    Row,
    Column,
    Textbox,
    Unknown,
}

impl From <u32> for WidgetType {
    fn from(val: u32) -> Self {
        match val {
            0x20 => WidgetType::Container,
            0x21 => WidgetType::Button,
            0x22 => WidgetType::Row,
            0x23 => WidgetType::Column,
            0x24 => WidgetType::Textbox,
            _ => WidgetType::Unknown,
        }
    }
}

#[repr(u32)]
enum WidgetAttrType {
    Text = 0x30,
    Name,
    BackgroundColor,
    Children,
    Unknown,
}

impl From <u32> for WidgetAttrType {
    fn from(val: u32) -> Self {
        match val {
            0x30 => WidgetAttrType::Text,
            0x31 => WidgetAttrType::Name,
            0x32 => WidgetAttrType::BackgroundColor,
            0x33 => WidgetAttrType::Children,
            _ => WidgetAttrType::Unknown,
        }
    }
}

pub struct CallerState {
    next_file_id: i32,
    files: HashMap<i32, File>,
}

impl CallerState {
    pub fn new() -> Self {
        Self {
            next_file_id: 0,
            files: HashMap::new(),
        }
    }
}

impl App {
    pub fn new_from_path(wasm_path: std::path::PathBuf) -> Self {
        let wasm_bytes = std::fs::read(wasm_path).unwrap();
        return Self::new_from_bytes(wasm_bytes);
    }
    
    pub fn new_from_bytes(wasm_bytes: Vec<u8>) -> Self {

        let engine = Engine::default();

        let start = std::time::Instant::now();
        let module = Module::new(&engine, wasm_bytes).unwrap();
        println!("module time: {:?}", start.elapsed());

        let mut linker = Linker::new(&engine);

        linker
            .func_wrap(
                "hedge",
                "open",
                move |mut caller: Caller<'_, CallerState>, ptr: i32, len: i32| -> i32 {
                    let path = match caller.get_export("memory") {
                        Some(Extern::Memory(mem)) => {
                            let data = mem.data(&caller)
                                .get(ptr as u32 as usize..)
                                .and_then(|arr| arr.get(..len as u32 as usize));
                            let path = match data {
                                Some(data) => match str::from_utf8(data) {
                                    Ok(s) => s,
                                    Err(_) => {
                                        eprintln!("invalid utf-8");
                                        return -1;
                                    },
                                },
                                None => {
                                    eprintln!("pointer/length out of bounds");
                                    return -1;
                                },
                            };

                            path
                        }
                        _ => {
                            eprintln!("Failed to get memory");
                            return -1;
                        }
                    };

                    let file = match File::create(path) {
                        Ok(file) => file,
                        Err(_) => {
                            return -1;
                        },
                    };

                    let data = caller.data_mut();

                    let id = data.next_file_id;
                    data.next_file_id += 1;

                    data.files.insert(id, file);

                    return id;
                },
            )
            .unwrap();
        linker
            .func_wrap(
                "hedge",
                "write",
                move |mut caller: Caller<'_, CallerState>, fd: i32, buf_ptr: i32, buf_len: i32| -> i32 { 

                    let bytes = match caller.get_export("memory") {
                        Some(Extern::Memory(mem)) => {
                            let bytes = mem.data(&caller)
                                .get(buf_ptr as u32 as usize..)
                                .and_then(|arr| arr.get(..buf_len as u32 as usize));
                            match bytes {
                                Some(bytes) => bytes,
                                None => return -1,
                            }
                        }
                        _ => {
                            eprintln!("Failed to get memory");
                            return -1;
                        }
                    };

                    let data = caller.data();

                    let mut file = match data.files.get(&fd) {
                        Some(file) => file,
                        None => return -1,
                    };

                    let n_bytes = match file.write(bytes) {
                        Ok(n_bytes) => n_bytes as i32,
                        Err(_) => return -1,
                    };

                    return n_bytes;
                },
            )
            .unwrap();
        linker
            .func_wrap(
                "wasi_snapshot_preview1",
                "fd_close",
                move |_caller: Caller<'_, CallerState>, _fd: i32| {
                    eprintln!("fd_close not implemented");
                    0
                },
            )
            .unwrap();
        linker
            .func_wrap(
                "wasi_snapshot_preview1",
                "fd_seek",
                move |_caller: Caller<'_, CallerState>, _: i32, _: i64, _: i32, _: i32| {
                    eprintln!("fd_seek not implemented");
                    0
                },
            )
            .unwrap();
        linker
            .func_wrap(
                "wasi_snapshot_preview1",
                "fd_write",
                move |_caller: Caller<'_, CallerState>, _: i32, _: i32, _: i32, _: i32| {
                    eprintln!("fd_write not implemented");
                    0
                },
            )
            .unwrap();
        linker
            .func_wrap(
                "wasi_snapshot_preview1",
                "args_get",
                move |_caller: Caller<'_, CallerState>, _: i32, _: i32| {
                    eprintln!("args_get not implemented");
                    0
                },
            )
            .unwrap();
        linker
            .func_wrap(
                "wasi_snapshot_preview1",
                "args_sizes_get",
                move |_caller: Caller<'_, CallerState>, _: i32, _: i32| {
                    eprintln!("args_sizes_get not implemented");
                    0
                },
            )
            .unwrap();
        linker
            .func_wrap(
                "wasi_snapshot_preview1",
                "proc_exit",
                move |_caller: Caller<'_, CallerState>, _: i32| {
                    eprintln!("proc_exit not implemented");
                },
            )
            .unwrap();



        
        let mut store: Store<CallerState> = Store::new(&engine, CallerState::new());

        let instance = linker.instantiate(&mut store, &module).unwrap();

        let eri_init = instance
            .get_typed_func::<(), ()>(&mut store, "eri_init")
            .unwrap();
        let get_input_message_buffer_offset = instance
            .get_typed_func::<(), i32>(&mut store, "eri_get_in_msg_buf")
            .unwrap();
        let get_output_message_buffer_offset = instance
            .get_typed_func::<(), i32>(&mut store, "eri_get_out_msg_buf")
            .unwrap();
        let eri_update = instance
            .get_typed_func::<(), ()>(&mut store, "eri_update")
            .unwrap();

        let input_message_buffer_offset = get_input_message_buffer_offset.call(&mut store, ()).unwrap();
        let output_message_buffer_offset = get_output_message_buffer_offset.call(&mut store, ()).unwrap();

        eri_init.call(&mut store, ()).unwrap();

        let memory = instance.get_memory(&mut store, "memory").unwrap();


        Self{
            store,
            memory,
            input_message_buffer_offset: input_message_buffer_offset as usize,
            output_message_buffer_offset: output_message_buffer_offset as usize,
            eri_update,
        }
    }

    pub fn update(&mut self, messages: &Vec<InMessage>) -> Vec<OutMessage> {

        let in_msg_buf = &mut self.memory.data_mut(&mut self.store)[self.input_message_buffer_offset..];

        in_msg_buf[1..100].fill(42);

        encode_in_msgs(messages, in_msg_buf);


        self.eri_update.call(&mut self.store, ()).unwrap();


        let out_msg_buf = &mut self.memory.data_mut(&mut self.store)[self.output_message_buffer_offset..];

        let mut offset = 0;

        let mut out_messages = vec![];

        while out_msg_buf[offset] != 0 {
            let msg_type = u32::from_le_bytes((out_msg_buf[offset..offset+4]).try_into().unwrap());
            offset += 4;
            let msg_len = u32::from_le_bytes((out_msg_buf[offset..offset+4]).try_into().unwrap());
            offset += 4;

            let msg_type = OutMessageType::from(msg_type);

            match msg_type {
                OutMessageType::SetTree => {
                    out_messages.push(parse_tree(&out_msg_buf[offset..])); 
                },
                OutMessageType::Unknown => {
                    offset += msg_len as usize;
                }
            }
        }

        out_messages
    }
}

struct Tlv<'a> {
    typ: u32,
    len: u32,
    val: &'a [u8],
}

fn parse_tlv(buf: &[u8]) -> (Tlv<'_>, usize) {
    let mut off = 0;
    let typ = u32::from_le_bytes((buf[off..off+4]).try_into().unwrap());
    off += 4;
    let len = u32::from_le_bytes((buf[off..off+4]).try_into().unwrap());
    off += 4;

    let tlv = Tlv{
        typ: typ,
        len: len,
        val: &buf[off..(off+(len as usize))],
    };

    (tlv, off)
}

fn encode_in_msgs(msgs: &Vec<InMessage>, buf: &mut [u8]) {
    let mut off = 0;

    for msg in msgs {
        let consumed = encode_in_msg(&msg, &mut buf[off..]);
        off += consumed;
    }

    buf[off] = 0x00000000;
}

fn encode_in_msg(msg: &InMessage, buf: &mut [u8]) -> usize {
    let mut off = 0;

    match msg {
        InMessage::WidgetPressed { path, name } => {
            buf[off..off+4].copy_from_slice(&(InMessageType::WidgetPressed as u32).to_le_bytes());
            off += 4; 
            
            // reserve space to write len after we know its value
            let len_off = off;
            off += 4;

            off += encode_tlv(MessageAttrType::Path as u32, path.len() as u32, &path.as_bytes(), &mut buf[off..]);

            if let Some(name) = name {
                off += encode_tlv(MessageAttrType::Name as u32, name.len() as u32, &name.as_bytes(), &mut buf[off..]);
            }

            let len = off - len_off - 4;

            let len_slice = &mut buf[len_off..len_off+4];
            len_slice.copy_from_slice(&(len as u32).to_le_bytes());
        },
        InMessage::TextChanged { path, text } => {
            buf[off..off+4].copy_from_slice(&(InMessageType::TextChanged as u32).to_le_bytes());
            off += 4; 
            
            // reserve space to write len after we know its value
            let len_off = off;
            off += 4;

            off += encode_tlv(MessageAttrType::Path as u32, path.len() as u32, &path.as_bytes(), &mut buf[off..]);

            off += encode_tlv(MessageAttrType::Text as u32, text.len() as u32, &text.as_bytes(), &mut buf[off..]);

            let len = off - len_off - 4;

            let len_slice = &mut buf[len_off..len_off+4];
            len_slice.copy_from_slice(&(len as u32).to_le_bytes());

        }
    }

    off
}

fn encode_tlv(typ: u32, len: u32, val: &[u8], buf: &mut [u8]) -> usize {

    let mut off = 0;

    buf[off..off+4].copy_from_slice(&typ.to_le_bytes());
    off += 4;

    buf[off..off+4].copy_from_slice(&len.to_le_bytes());
    off += 4;

    buf[off..off+(len as usize)].copy_from_slice(val);
    off += len as usize;

    off
}

fn parse_tree(buf: &[u8]) -> OutMessage {

    let (root, _off) = parse_widget(buf, 0);

    OutMessage::SetTree{
        path: "/".to_string(),
        tree: root,
    }
}

fn parse_widget(buf: &[u8], depth: u32) -> (hedge::Widget, usize) {

    let mut off = 0;

    let (tlv, consumed) = parse_tlv(buf);
    off += consumed;

    //for _ in 0..depth {
    //    print!("  ");
    //}
    //println!("{:#x}, {}", tlv.typ, tlv.len);

    let mut text = String::new();
    let mut name = None;
    let mut children = vec![];

    let mut attr_buf = tlv.val;
    let mut attr_off = 0;

    while attr_buf.len() > 0 {
        let (attr_tlv, consumed) = parse_tlv(attr_buf);
        attr_off += consumed;

        let attr_type = WidgetAttrType::from(attr_tlv.typ);

        match attr_type {
            WidgetAttrType::Text => {
                text = parse_str(&attr_tlv.val[..attr_tlv.len as usize]);
            },
            WidgetAttrType::Name => {
                name = Some(parse_str(&attr_tlv.val[..attr_tlv.len as usize]));
            },
            WidgetAttrType::Children => {
                children = parse_children(attr_tlv.val, depth);
            },
            WidgetAttrType::BackgroundColor => {
            },
            WidgetAttrType::Unknown => {
            },
        }

        attr_off += attr_tlv.len as usize;

        attr_buf = &tlv.val[attr_off..];
    }

    off += tlv.len as usize;

    let widget_type = WidgetType::from(tlv.typ);

    let wid = match widget_type {
        WidgetType::Container => {
            hedge::Widget::Container{
                children: children,
            }
        },
        WidgetType::Button => {
            hedge::Widget::Button{
                name: name,
                text: text,
            }
        },
        WidgetType::Row => {
            hedge::Widget::Row{
                children: children,
            }
        },
        WidgetType::Column => {
            hedge::Widget::Column{
                children: children,
            }
        },
        WidgetType::Textbox => {
            hedge::Widget::Textbox{
                text: text,
            }
        },
        WidgetType::Unknown => {
            hedge::Widget::Label{
                text: "MISSING WIDGET".to_string(),
            }
        },
    };

    (wid, off)
}

fn parse_str(buf: &[u8]) -> String {
    std::str::from_utf8(buf).unwrap().to_string()
}

fn parse_children(buf: &[u8], depth: u32) -> Vec<hedge::Widget> {

    let mut off = 0;

    let mut children = vec![];

    while off < buf.len() {
        let (child, consumed) = parse_widget(&buf[off..], depth + 1);
        off += consumed;
        children.push(child);
    }

    children
}
