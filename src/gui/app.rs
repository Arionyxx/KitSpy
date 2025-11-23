use iced::widget::{button, column, container, row, scrollable, text, Column};
use iced::{executor, Application, Command, Element, Length, Settings, Theme, Color, Font, Subscription};
use std::path::PathBuf;
use std::sync::Arc;
use crate::core::loader;

pub fn run() -> iced::Result {
    App::run(Settings::default())
}

struct App {
    tree_data: Option<AssemblyData>,
    selected_method: Option<MethodData>,
    error_message: Option<String>,
}

#[derive(Debug, Clone)]
pub struct MethodData {
    pub name: String,
    pub il_code: Arc<String>,
}

#[derive(Debug, Clone)]
pub struct ClassData {
    pub name: String,
    pub methods: Vec<MethodData>,
    pub is_expanded: bool,
}

#[derive(Debug, Clone)]
pub struct NamespaceData {
    pub name: String,
    pub classes: Vec<ClassData>,
    pub is_expanded: bool,
}

#[derive(Debug, Clone)]
pub struct AssemblyData {
    pub name: String,
    pub namespaces: Vec<NamespaceData>,
}

#[derive(Debug, Clone)]
pub enum Message {
    OpenFile,
    FileSelected(Option<PathBuf>),
    FileLoaded(Result<AssemblyData, String>),
    ToggleNamespace(usize),
    ToggleClass(usize, usize),
    SelectMethod(usize, usize, usize),
}

impl Application for App {
    type Executor = executor::Default;
    type Message = Message;
    type Theme = Theme;
    type Flags = ();

    fn new(_flags: ()) -> (Self, Command<Message>) {
        (
            App {
                tree_data: None,
                selected_method: None,
                error_message: None,
            },
            Command::none(),
        )
    }

    fn title(&self) -> String {
        String::from("RustSpy")
    }

    fn update(&mut self, message: Message) -> Command<Message> {
        match message {
            Message::OpenFile => {
                 let future = async {
                    rfd::AsyncFileDialog::new().pick_file().await.map(|f| f.path().to_path_buf())
                };
                return Command::perform(future, Message::FileSelected);
            }
            Message::FileSelected(path_opt) => {
                if let Some(path) = path_opt {
                    return Command::perform(load_pe_async(path), Message::FileLoaded);
                }
            }
            Message::FileLoaded(res) => {
                match res {
                    Ok(data) => {
                        self.tree_data = Some(data);
                        self.error_message = None;
                        self.selected_method = None;
                    }
                    Err(e) => {
                        self.error_message = Some(e);
                    }
                }
            }
            Message::ToggleNamespace(idx) => {
                if let Some(data) = &mut self.tree_data {
                    if let Some(ns) = data.namespaces.get_mut(idx) {
                        ns.is_expanded = !ns.is_expanded;
                    }
                }
            }
            Message::ToggleClass(ns_idx, class_idx) => {
                 if let Some(data) = &mut self.tree_data {
                    if let Some(ns) = data.namespaces.get_mut(ns_idx) {
                         if let Some(cls) = ns.classes.get_mut(class_idx) {
                             cls.is_expanded = !cls.is_expanded;
                         }
                    }
                }
            }
            Message::SelectMethod(ns_idx, class_idx, method_idx) => {
                if let Some(data) = &self.tree_data {
                    if let Some(ns) = data.namespaces.get(ns_idx) {
                         if let Some(cls) = ns.classes.get(class_idx) {
                             if let Some(method) = cls.methods.get(method_idx) {
                                 self.selected_method = Some(method.clone());
                             }
                         }
                    }
                }
            }
        }
        Command::none()
    }

    fn view(&self) -> Element<Message> {
        let toolbar = row![
            button("Open PE").on_press(Message::OpenFile)
        ]
        .padding(10)
        .spacing(10);

        let sidebar_content = if let Some(data) = &self.tree_data {
            view_tree(data)
        } else {
            text("No file loaded").style(Color::from_rgb8(0xD4, 0xD4, 0xD4)).into()
        };

        let sidebar = container(scrollable(sidebar_content))
            .width(Length::FillPortion(1))
            .height(Length::Fill)
            .style(sidebar_style)
            .padding(10);

        let viewer_content = if let Some(method) = &self.selected_method {
            column![
                text(&method.name).size(20).style(Color::from_rgb8(0xD4, 0xD4, 0xD4)),
                text(&*method.il_code).font(Font::MONOSPACE).style(Color::from_rgb8(0xD4, 0xD4, 0xD4))
            ].spacing(10)
        } else {
            column![
                text("Select a method to view IL").style(Color::from_rgb8(0xD4, 0xD4, 0xD4))
            ]
        };

        let viewer = container(scrollable(viewer_content))
            .width(Length::FillPortion(3))
            .height(Length::Fill)
            .style(viewer_style)
            .padding(10);

        let main_content = row![sidebar, viewer].spacing(1);

        let mut content_col = column![toolbar, main_content];

        if let Some(err) = &self.error_message {
            let banner = container(text(err).style(Color::WHITE))
                .style(error_banner_style)
                .padding(10)
                .width(Length::Fill);
            content_col = content_col.push(banner);
        }

        container(content_col)
            .width(Length::Fill)
            .height(Length::Fill)
            .style(app_style)
            .into()
    }
    
    fn theme(&self) -> Theme {
        Theme::Dark
    }

    fn subscription(&self) -> Subscription<Message> {
        Subscription::none()
    }
}

fn view_tree(data: &AssemblyData) -> Element<Message> {
    let mut col = Column::new().spacing(5);
    
    col = col.push(text(&data.name).size(16).style(Color::from_rgb8(0xD4, 0xD4, 0xD4)));

    for (ns_idx, ns) in data.namespaces.iter().enumerate() {
        let prefix = if ns.is_expanded { "▼ " } else { "▶ " };
        let ns_header = button(text(format!("{}{}", prefix, ns.name)))
            .on_press(Message::ToggleNamespace(ns_idx))
            .style(iced::theme::Button::Text);
        
        col = col.push(ns_header);

        if ns.is_expanded {
             for (cls_idx, cls) in ns.classes.iter().enumerate() {
                 let cls_prefix = if cls.is_expanded { "▼ " } else { "▶ " };
                 let cls_header = button(text(format!("{}{}", cls_prefix, cls.name)))
                    .on_press(Message::ToggleClass(ns_idx, cls_idx))
                    .style(iced::theme::Button::Text)
                    .padding([0, 0, 0, 20]); // Indent

                 col = col.push(cls_header);

                 if cls.is_expanded {
                      for (method_idx, method) in cls.methods.iter().enumerate() {
                           let method_btn = button(text(&method.name).font(Font::MONOSPACE))
                                .on_press(Message::SelectMethod(ns_idx, cls_idx, method_idx))
                                .style(iced::theme::Button::Text)
                                .padding([0, 0, 0, 40]); // Indent

                           col = col.push(method_btn);
                      }
                 }
             }
        }
    }

    col.into()
}

fn sidebar_style(theme: &Theme) -> container::Appearance {
    container::Appearance {
        background: Some(Color::from_rgb8(0x25, 0x25, 0x26).into()),
        text_color: Some(Color::from_rgb8(0xD4, 0xD4, 0xD4)),
        ..theme.extended_palette().background.base.container(theme)
    }
}

fn viewer_style(theme: &Theme) -> container::Appearance {
    container::Appearance {
        background: Some(Color::from_rgb8(0x1E, 0x1E, 0x1E).into()),
        text_color: Some(Color::from_rgb8(0xD4, 0xD4, 0xD4)),
        ..theme.extended_palette().background.base.container(theme)
    }
}

fn app_style(theme: &Theme) -> container::Appearance {
     container::Appearance {
        background: Some(Color::from_rgb8(0x1E, 0x1E, 0x1E).into()),
        ..theme.extended_palette().background.base.container(theme)
    }
}

fn error_banner_style(_theme: &Theme) -> container::Appearance {
    container::Appearance {
        background: Some(Color::from_rgb(0.8, 0.0, 0.0).into()),
        text_color: Some(Color::WHITE),
        ..Default::default()
    }
}

async fn load_pe_async(path: PathBuf) -> Result<AssemblyData, String> {
    let artifact = loader::load_pe(&path).map_err(|e| e.to_string())?;
    
    let filename = path.file_name().unwrap_or_default().to_string_lossy().to_string();
    
    // Create "Imports" namespace
    let mut import_classes = Vec::new();
    for import in artifact.imports {
        let methods = import.symbols.into_iter().map(|s| MethodData {
             name: s.clone(),
             il_code: Arc::new(format!("// Imported from {}\nextern {}", import.dll, s)),
        }).collect();
        
        import_classes.push(ClassData {
            name: import.dll,
            methods,
            is_expanded: false,
        });
    }
    
    let imports_ns = NamespaceData {
        name: "Imports".to_string(),
        classes: import_classes,
        is_expanded: true,
    };

    // Create "Sections" namespace
    let mut section_methods = Vec::new();
    for section in artifact.sections {
         section_methods.push(MethodData {
             name: section.name.clone(),
             il_code: Arc::new(format!("Section: {}\nVirtual Address: {:#x}\nVirtual Size: {:#x}", section.name, section.virtual_address, section.virtual_size)),
         });
    }
    
    let sections_class = ClassData {
        name: "Sections".to_string(),
        methods: section_methods,
        is_expanded: false,
    };
    
    let sections_ns = NamespaceData {
        name: "Metadata".to_string(),
        classes: vec![sections_class],
        is_expanded: true,
    };
    
    Ok(AssemblyData {
        name: filename,
        namespaces: vec![imports_ns, sections_ns],
    })
}
