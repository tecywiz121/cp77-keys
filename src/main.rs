mod opt;

use std::borrow::Cow;
use std::collections::hash_map::{Entry, HashMap};
use std::fs::{File, OpenOptions};
use std::io::{BufReader, BufWriter};
use std::path::PathBuf;
use std::time::UNIX_EPOCH;

use structopt::StructOpt;

use xml::attribute::Attribute;
use xml::reader::{EventReader, XmlEvent as RdEvent};
use xml::writer::{EventWriter, XmlEvent as WrEvent};

const STEAM_PATH: &str = concat!(
    "C:\\Program Files (x86)\\Steam\\steamapps\\common\\Cyberpunk 2077\\r6\\",
    "config\\inputUserMappings.xml"
);

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let opt = opt::Opt::from_args();

    let file = File::open(opt.remap)?;
    let file = BufReader::new(file);
    let mappings: HashMap<String, String> = serde_json::from_reader(file)?;

    let ium_path = opt
        .user_mapping
        .unwrap_or_else(|| PathBuf::from(STEAM_PATH));

    let now = UNIX_EPOCH.elapsed()?.as_millis();
    let ium_filename = ium_path
        .file_name()
        .expect("no ium file name")
        .to_string_lossy();

    let ium_filename_bak = format!("{}.bak.{}", ium_filename, now);
    let ium_backup = ium_path.with_file_name(ium_filename_bak);

    let ium_filename_new = format!("{}.new.{}", ium_filename, now);
    let ium_new = ium_path.with_file_name(ium_filename_new);

    let output = OpenOptions::new()
        .create_new(true)
        .write(true)
        .open(&ium_new)?;
    let output = BufWriter::new(output);

    let input = File::open(&ium_path)?;
    let input = BufReader::new(input);

    let config = xml::reader::ParserConfig::new()
        .ignore_comments(false)
        .trim_whitespace(false);

    let mut reader = EventReader::new_with_config(input, config);
    let mut writer = EventWriter::new(output);

    let mut remaps: HashMap<String, String> =
        HashMap::with_capacity(mappings.len());

    let mut latest = None;

    loop {
        match reader.next()? {
            RdEvent::EndDocument => break,
            RdEvent::CData(s) => writer.write(WrEvent::CData(&s))?,
            RdEvent::Characters(s) => writer.write(WrEvent::Characters(&s))?,
            RdEvent::Comment(s) => writer.write(WrEvent::Comment(&s))?,
            RdEvent::ProcessingInstruction { name, data } => {
                writer.write(WrEvent::ProcessingInstruction {
                    name: &name,
                    data: data.as_ref().map(|s| s.as_str()),
                })?
            }
            RdEvent::StartDocument {
                version,
                encoding,
                standalone,
            } => writer.write(WrEvent::StartDocument {
                encoding: Some(&encoding),
                standalone,
                version,
            })?,
            RdEvent::StartElement {
                name,
                attributes,
                namespace,
            } => {
                match name.local_name.as_str() {
                    "buttonGroup" => {
                        latest = attributes
                            .iter()
                            .filter(|a| a.name.local_name == "id")
                            .next()
                            .map(|a| a.value.to_owned());
                    }
                    "mapping" => {
                        latest = attributes
                            .iter()
                            .filter(|a| a.name.local_name == "name")
                            .next()
                            .map(|a| a.value.to_owned());
                    }
                    _ => (),
                }

                let v: Vec<_> = if name.local_name == "button" {
                    let mut modified = Vec::with_capacity(attributes.len());

                    for a in attributes.iter() {
                        if a.name.local_name == "id" {
                            let old = &a.value;
                            let new = mappings.get(old).unwrap_or(old);

                            match remaps.entry(new.clone()) {
                                Entry::Occupied(o) => {
                                    if o.get() != old {
                                        eprintln!(
                                            "Original key {} double mapped to {} for {}",
                                            old,
                                            new,
                                            latest.as_ref().unwrap(),
                                        );
                                    }
                                }
                                Entry::Vacant(v) => {
                                    v.insert(old.clone());
                                }
                            }

                            let replacement = Attribute {
                                name: a.name.borrow(),
                                value: new,
                            };

                            modified.push(replacement);
                        } else {
                            modified.push(a.borrow());
                        }
                    }

                    modified
                } else {
                    attributes.iter().map(|a| a.borrow()).collect()
                };

                writer.write(WrEvent::StartElement {
                    name: name.borrow(),
                    namespace: Cow::Owned(namespace),
                    attributes: Cow::Owned(v),
                })?
            }
            RdEvent::EndElement { name } => {
                writer.write(WrEvent::EndElement {
                    name: Some(name.borrow()),
                })?
            }
            RdEvent::Whitespace(s) => writer.write(WrEvent::Characters(&s))?,
        }
    }

    std::fs::rename(&ium_path, &ium_backup)?;
    std::fs::rename(&ium_new, &ium_path)?;

    Ok(())
}
