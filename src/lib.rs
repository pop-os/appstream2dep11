#[macro_use]
extern crate serde_derive;

use std::io::BufReader;
use std::fs::File;

use xml::reader::{self, EventReader, XmlEvent};

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(untagged)]
pub enum Data {
    Type(Option<String>),
    Id(Option<String>),
    Package(Option<String>),
    Name {
        #[serde(rename = "C")]
        c: Option<String>
    },
    Summary {
        #[serde(rename = "C")]
        c: Option<String>
    },
    Description {
        #[serde(rename = "C")]
        c: Option<String>
    },
    DeveloperName {
        #[serde(rename = "C")]
        c: Option<String>
    },
    Screenshots(Vec<Screenshot>),
    Url {
        homepage: Option<String>,
        bugtracker: Option<String>,
    },
    Icon {
        cached: Vec<InnerIcon>,
    },
    Provides {
        mimetypes: Vec<String>,
        binaries: Vec<String>,
    },
    ProjectLicense(Option<String>),
    Categories(Vec<String>),
    Keywords(Vec<String>),
}

impl Data {
    pub fn push_new_screenshoot(&mut self, thumbnails: Vec<String>, url: Option<String>, lang: Option<String>) {
        if let Data::Screenshots(ref mut vec) = self {
            let screenshot = Screenshot {
                thumbnails,
                source_image: SourceImage {
                    url,
                    lang,
                },
            };
            vec.push(screenshot);
        }
    }

    pub fn push_into_icon<S: Into<String>>(&mut self, name: S, width: Option<usize>, height: Option<usize>) {
        if let Data::Icon { ref mut cached } = self {
            cached.push(InnerIcon {name: name.into(), width, height});
        }
    }
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct SourceImage {
    url: Option<String>,
    lang: Option<String>,
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct Screenshot {
    thumbnails: Vec<String>,
    #[serde(rename = "source-image")]
    source_image: SourceImage,
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct InnerIcon {
    name: String,
    width: Option<usize>,
    height: Option<usize>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Dep11 {
    #[serde(rename = "Type")]
    kind: Data,
    #[serde(rename = "ID")]
    id: Data,
    #[serde(rename = "Package")]
    package: Data,
    #[serde(rename = "Name")]
    name: Data,
    #[serde(rename = "Summary")]
    summary: Data,
    #[serde(rename = "Description")]
    description: Data,
    #[serde(rename = "DeveloperName")]
    developer_name: Data,
    #[serde(rename = "ProjectLicense")]
    project_license: Data,
    #[serde(rename = "Categories")]
    categories: Data,
    #[serde(rename = "Keywords")]
    keywords: Data,
    #[serde(rename = "Url")]
    url: Data,
    #[serde(rename = "Icon")]
    icon: Data,
    #[serde(rename = "Screenshots")]
    screenshots: Data,
    #[serde(rename = "Provides")]
    provides: Data,
}

impl Default for Dep11 {
    fn default() -> Self {
        Dep11 {
            kind: Data::Type(None),
            id: Data::Id(None),
            package: Data::Package(None),
            name: Data::Name { c: None },
            summary: Data::Summary { c: None },
            description: Data::Description { c: None },
            developer_name: Data::DeveloperName { c: None },
            project_license: Data::ProjectLicense(None),
            categories: Data::Categories(Vec::new()),
            keywords: Data::Keywords(Vec::new()),
            url: Data::Url { homepage: None, bugtracker: None, },
            icon: Data::Icon { cached: Vec::new() },
            screenshots: Data::Screenshots(Vec::new()),
            provides: Data::Provides { mimetypes: Vec::new(), binaries: Vec::new(), },
        }
    }
}

impl Dep11 {
    #[allow(clippy::cyclomatic_complexity)]
    pub fn new(path: &str) -> Self {
        let mut parser = EventReader::new(BufReader::new(File::open(path).unwrap())).into_iter();
        let mut dep11 = Dep11::default();

        while let Some(Ok(event)) = parser.next() {
            if let XmlEvent::StartElement { name, attributes, namespace } = event {
                match &*name.local_name {
                    "component" => {
                        if let Data::Type(ref mut kind) = dep11.kind {
                            *kind = attributes
                                .iter()
                                .find(|attribute| attribute.name.local_name == "type")
                                .map(|component| component.value.clone())
                        }
                    }
                    "id" => {
                        if let Data::Id(ref mut dep11_id) = dep11.id {
                            if let Some(Ok(XmlEvent::Characters(id))) = parser.next() {
                                *dep11_id = Some(id);
                            }
                        }
                    }
                    "summary" => {
                        if attributes.is_empty() {
                            if let Data::Summary { ref mut c } = dep11.summary {
                                if let Some(Ok(XmlEvent::Characters(summary))) = parser.next() {
                                    *c = Some(summary)
                                }
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
                                        if let Data::Description { ref mut c } = dep11.description {
                                            *c = Some(description.trim().into());
                                            break;
                                        }
                                    }
                                }
                            }
                        }
                    }
                    "project_license" => {
                        if let Some(Ok(XmlEvent::Characters(license))) = parser.next() {
                            if let Data::ProjectLicense(ref mut project_license) = dep11.project_license {
                                *project_license = Some(license)
                            }
                        }
                    }
                    "name" => {
                        if attributes.is_empty() {
                            if let Some(Ok(XmlEvent::Characters(name))) = parser.next() {
                                if let Data::Name { ref mut c } = dep11.name {
                                    *c = Some(name)
                                }
                            }
                        }
                    }
                    "url" => {
                        let maybe_homepage = namespace.get("homepage").map(|s| s.into());
                        let maybe_bugtracker = namespace.get("bugtracker").map(|s| s.into());
                        if let Data::Url { ref mut homepage, ref mut bugtracker } = dep11.url {
                            *homepage = maybe_homepage;
                            *bugtracker = maybe_bugtracker;
                        }
                    }
                    "screenshots" => {
                        let mut events = parser
                            .by_ref()
                            .take_while(|event| if let Ok(XmlEvent::EndElement { name }) = event {
                                name.local_name != "screenshots"
                            } else {
                                true
                            });

                        while let Some(Ok(event)) = events.next() {
                            if let XmlEvent::StartElement { name, .. } = event {
                                if name.local_name == "image" {
                                    if let Some(Ok(XmlEvent::Characters(screenshot))) = events.next() {
                                        dep11.screenshots.push_new_screenshoot(Vec::new(), screenshot.into(), String::from("C").into());
                                    }
                                }
                            }
                        }
                    }
                    "icon" => {
                        if let Some(Ok(XmlEvent::Characters(mut icon))) = parser.next() {
                            icon.push_str(".png");
                            dep11.icon.push_into_icon(icon, None, None);
                        }
                    }
                    "developer_name" => {
                        if let Some(Ok(XmlEvent::Characters(developer_name))) = parser.next() {
                            if let Data::DeveloperName { ref mut c } = dep11.developer_name {
                                *c = Some(developer_name)
                            }
                        }
                    }
                    "keywords" => {
                        if let Data::Keywords(ref mut keywords) = dep11.keywords {
                            collect_data_to(keywords, "keywords", "keyword", &mut parser);
                        }
                    }
                    "provides" => {
                        if let Data::Provides { ref mut binaries, .. } = dep11.provides {
                            collect_data_to(binaries, "provides", "binary", &mut parser);
                        }
                    }
                    "mimetypes" => {
                        if let Data::Provides { ref mut mimetypes, .. } = dep11.provides {
                            collect_data_to(mimetypes, "mimetypes", "mimetype", &mut parser);
                        }
                    }
                    "categories" => {
                        if let Data::Categories(ref mut categories) = dep11.keywords {
                            collect_data_to(categories, "categories", "category", &mut parser);
                        }
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

    pub fn checked_for_completion<F: FnMut(&str, &mut Data)>(&mut self, mut callback: F) {
        loop {
            if let Data::Type(None) = self.kind {
                callback("Please provide a `Type`.", &mut self.kind);
                continue;
            }
            if let Data::Id(None) = self.id {
                callback("Please provide an `Id`.", &mut self.id);
                continue;
            }
            if let Data::Package(None) = self.package {
                callback("Please provide a `Package`.", &mut self.package);
                continue;
            }
            if let Data::Summary { c: None } = self.summary {
                callback("Please provide a `Summary`.", &mut self.summary);
                continue;
            }
            if let Data::Description { c: None } = self.description {
                callback("Please provide a `Description`.", &mut self.description);
                continue;
            }
            if let Data::DeveloperName { c: None } = self.developer_name {
                callback("Please provide a `DeveloperName`.", &mut self.developer_name);
                continue;
            }
            if let Data::Categories(ref vec) = self.categories {
                if vec.is_empty() {
                    callback("Please provide `Categories`.", &mut self.categories);
                    continue;
                }
            }
            if let Data::Keywords(ref vec) = self.keywords {
                if vec.is_empty() {
                    callback("Please provide `Categories`.", &mut self.keywords);
                    continue;
                }
            }
            if let Data::Url { homepage: None, .. } = self.url {
                callback("Please provide `homepage`.", &mut self.url);
                continue;
            }
            if let Data::Icon { ref cached, .. } = self.icon {
                if cached.is_empty() {
                    callback("Please provide `Icon`(s).", &mut self.icon);
                    continue;
                }
            }
            if let Data::Screenshots(ref vec) = self.screenshots {
                if vec.is_empty() {
                    callback("Please provide `Screenshots`.", &mut self.screenshots);
                    continue;
                }
            }
            if let Data::Provides { ref mimetypes, ref binaries } = self.provides {
                if mimetypes.is_empty() {
                    callback("Please provide `mimetypes`.", &mut self.provides);
                    continue;
                }
                if binaries.is_empty() {
                    callback("Please provide `binaries`.", &mut self.provides);
                    continue;
                }
            }
            break;
        }
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
