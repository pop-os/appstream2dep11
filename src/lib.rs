#[macro_use]
extern crate serde_derive;

use std::io::BufReader;
use std::fs::File;

use xml::reader::{self, EventReader, XmlEvent};

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
struct Name {
    #[serde(rename = "C")]
    c: Option<String>,
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
struct Summary {
    #[serde(rename = "C")]
    c: Option<String>,
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
struct Description {
    #[serde(rename = "C")]
    c: Option<String>,
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
struct DeveloperName {
    #[serde(rename = "C")]
    c: Option<String>,
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
struct SourceImage {
    url: Option<String>,
    lang: Option<String>,
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
struct Screenshot {
    thumbnails: Vec<String>,
    #[serde(rename = "source-image")]
    source_image: SourceImage,
}

impl Screenshot {
    fn new(thumbnails: Vec<String>, url: Option<String>, lang: Option<String>) -> Self {
        Screenshot {
            thumbnails,
            source_image: SourceImage {
                url,
                lang,
            },
        }
    }
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct Url {
    homepage: Option<String>,
    bugtracker: Option<String>,
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
struct InnerIcon {
    name: String,
    width: Option<usize>,
    height: Option<usize>,
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct Icon {
    cached: Vec<InnerIcon>,
}

impl Icon {
    fn push<S: Into<String>>(&mut self, name: S, width: Option<usize>, height: Option<usize>) {
        self.cached.push(InnerIcon {name: name.into(), width, height});
    }
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct Provides {
    mimetypes: Vec<String>,
    binaries: Vec<String>,
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct Dep11 {
    #[serde(rename = "Type")]
    kind: Option<String>,
    #[serde(rename = "ID")]
    id: Option<String>,
    #[serde(rename = "Package")]
    package: Option<String>,
    #[serde(rename = "Name")]
    name: Name,
    #[serde(rename = "Summary")]
    summary: Summary,
    #[serde(rename = "Description")]
    description: Description,
    #[serde(rename = "DeveloperName")]
    developer_name: DeveloperName,
    #[serde(rename = "ProjectLicense")]
    project_license: Option<String>,
    #[serde(rename = "Categories")]
    categories: Vec<String>,
    #[serde(rename = "Keywords")]
    keywords: Vec<String>,
    #[serde(rename = "Url")]
    url: Url,
    #[serde(rename = "Icon")]
    icon: Icon,
    #[serde(rename = "Screenshots")]
    screenshots: Vec<Screenshot>,
    #[serde(rename = "Provides")]
    provides: Provides,
}

impl Dep11 {
    pub fn new(path: &str) -> Self {
        let mut parser = EventReader::new(BufReader::new(File::open(path).unwrap())).into_iter();
        let mut dep11 = Dep11::default();

        while let Some(Ok(event)) = parser.next() {
            if let XmlEvent::StartElement { name, attributes, namespace } = event {
                match &*name.local_name {
                    "component" => {
                        dep11.kind = attributes
                            .iter()
                            .find(|attribute| attribute.name.local_name == "type")
                            .map(|component| component.value.clone())
                    }
                    "id" => {
                        if let Some(Ok(XmlEvent::Characters(id))) = parser.next() {
                            dep11.id = Some(id);
                        }
                    }
                    "summary" => {
                        if attributes.is_empty() {
                            if let Some(Ok(XmlEvent::Characters(summary))) = parser.next() {
                                dep11.summary.c = Some(summary)
                            }
                        }
                    }
                    "description" => {
                        let mut events = parser
                            .by_ref()
                            .take_while(|event| if let Ok(XmlEvent::EndElement { name }) = event {
                                name.local_name != "description"
                            } else {
                                true
                            });
                        while let Some(Ok(event)) = events.next() {
                            if let XmlEvent::StartElement { name, attributes, .. } = event {
                                if name.local_name == "p" && attributes.is_empty() {
                                    if let Some(Ok(XmlEvent::Characters(description))) = events.next() {
                                        dep11.description.c = Some(description.trim().into());
                                        break;
                                    }
                                }
                            }
                        }
                    }
                    "project_license" => {
                        if let Some(Ok(XmlEvent::Characters(license))) = parser.next() {
                            dep11.project_license = Some(license)
                        }
                    }
                    "name" => {
                        if attributes.is_empty() {
                            if let Some(Ok(XmlEvent::Characters(name))) = parser.next() {
                                dep11.name.c = Some(name)
                            }
                        }
                    }
                    "url" => {
                        dep11.url.homepage = namespace.get("homepage").map(|s| s.into());
                        dep11.url.bugtracker = namespace.get("bugtracker").map(|s| s.into());
                    }
                    "screenshots" => {
                        let mut events = parser
                            .by_ref()
                            .take_while(|event| if let Ok(XmlEvent::EndElement { name }) = event {
                                name.local_name != "screenshots"
                            } else {
                                true
                            });

                        let mut screenshots = Vec::new();
                        while let Some(Ok(event)) = events.next() {
                            if let XmlEvent::StartElement { name, .. } = event {
                                if name.local_name == "image" {
                                    if let Some(Ok(XmlEvent::Characters(screenshot))) = events.next() {
                                        screenshots.push(Screenshot::new(Vec::new(), screenshot.into(), String::from("C").into()))
                                    }
                                }
                            }
                        }
                        dep11.screenshots = screenshots;
                    }
                    "icon" => {
                        if let Some(Ok(XmlEvent::Characters(mut icon))) = parser.next() {
                            icon.push_str(".png");
                            dep11.icon.push(icon, None, None);
                        }
                    }
                    "developer_name" => {
                        if let Some(Ok(XmlEvent::Characters(developer_name))) = parser.next() {
                            dep11.developer_name.c = Some(developer_name)
                        }
                    }
                    "keywords" => {
                        collect_data_to(&mut dep11.keywords, "keywords", "keyword", &mut parser);
                    }
                    "provides" => {
                        collect_data_to(&mut dep11.provides.binaries, "provides", "binary", &mut parser);
                    }
                    "mimetypes" => {
                        collect_data_to(&mut dep11.provides.mimetypes, "mimetypes", "mimetype", &mut parser);
                    }
                    "categories" => {
                        collect_data_to(&mut dep11.categories, "categories", "category", &mut parser);
                    }
                    _ => (),
                }
            }
        }
        dep11
    }

    pub fn to_string(&self) -> Result<String, String> {
        serde_yaml::to_string(self)
            .map_err(|why| format!("{}", why))
    }
}

fn collect_data_to(data: &mut Vec<String>, source_name: &str, target_name: &str, parser: &mut impl Iterator<Item = reader::Result<XmlEvent>>) {
    let mut events = parser
        .by_ref()
        .take_while(|event| if let Ok(XmlEvent::EndElement { name }) = event {
            name.local_name != source_name
        } else {
            true
        });

    while let Some(Ok(event)) = events.next() {
        if let XmlEvent::StartElement { name, .. } = event {
            if name.local_name == target_name {
                if let Some(Ok(XmlEvent::Characters(binary))) = events.next() {
                    data.push(binary);
                }
            }
        }
    }
}
