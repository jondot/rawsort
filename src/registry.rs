use crate::exif::Exif;
use std::collections::HashMap;
use walkdir::DirEntry;

pub struct Registry {
    map: HashMap<String, (String, Box<dyn Fn(&Exif, &DirEntry) -> String>)>,
}

impl Registry {
    pub fn new() -> Registry {
        Registry {
            map: HashMap::new(),
        }
    }
    pub fn add<T>(&mut self, name: String, desc: String, func: T)
    where
        T: Fn(&Exif, &DirEntry) -> String + 'static,
    {
        self.map.insert(name, (desc, Box::new(func)));
    }
    pub fn describe(&self) -> Vec<(&String, &String)> {
        return self
            .map
            .keys()
            .map(|k| {
                let (desc, _) = self.map.get(k).unwrap();
                (k, desc)
            })
            .collect();
    }
    pub fn format(&self, fmt: &str, ent: &DirEntry) -> Result<String, String> {
        let path = ent.path();
        let exif = Exif::from_path(&path);
        let fmtstring = fmt.to_string();
        exif.map(|exif_ok| {
            self.map.keys().fold(fmtstring, |acc, k| {
                let (_, val) = &self.map.get(k).unwrap();
                acc.replace(k, &val(&exif_ok, ent))
            })
        })
    }
}
