use std::collections::HashMap;
use std::path::PathBuf;
use std::time::SystemTime;
use std::fs::metadata;
use std::sync::RwLock;

use anyhow::{bail, Error, format_err};
use hyper::Method;
use handlebars::Handlebars;
use serde::Serialize;

use proxmox::api::{ApiMethod, Router, RpcEnvironmentType};

pub struct ApiConfig {
    basedir: PathBuf,
    router: &'static Router,
    aliases: HashMap<String, PathBuf>,
    env_type: RpcEnvironmentType,
    templates: RwLock<Handlebars<'static>>,
    template_files: RwLock<HashMap<String, (SystemTime, PathBuf)>>,
}

impl ApiConfig {

    pub fn new<B: Into<PathBuf>>(basedir: B, router: &'static Router, env_type: RpcEnvironmentType) -> Result<Self, Error> {
        Ok(Self {
            basedir: basedir.into(),
            router,
            aliases: HashMap::new(),
            env_type,
            templates: RwLock::new(Handlebars::new()),
            template_files: RwLock::new(HashMap::new()),
        })
    }

    pub fn find_method(
        &self,
        components: &[&str],
        method: Method,
        uri_param: &mut HashMap<String, String>,
    ) -> Option<&'static ApiMethod> {

        self.router.find_method(components, method, uri_param)
    }

    pub fn find_alias(&self, components: &[&str]) -> PathBuf {

        let mut prefix = String::new();
        let mut filename = self.basedir.clone();
        let comp_len = components.len();
        if comp_len >= 1 {
            prefix.push_str(components[0]);
            if let Some(subdir) = self.aliases.get(&prefix) {
                filename.push(subdir);
                for i in 1..comp_len { filename.push(components[i]) }
            } else {
                for i in 0..comp_len { filename.push(components[i]) }
            }
        }
        filename
    }

    pub fn add_alias<S, P>(&mut self, alias: S, path: P)
        where S: Into<String>,
              P: Into<PathBuf>,
    {
        self.aliases.insert(alias.into(), path.into());
    }

    pub fn env_type(&self) -> RpcEnvironmentType {
        self.env_type
    }

    pub fn register_template<P>(&self, name: &str, path: P) -> Result<(), Error>
    where
        P: Into<PathBuf>
    {
        if self.template_files.read().unwrap().contains_key(name) {
            bail!("template already registered");
        }

        let path: PathBuf = path.into();
        let metadata = metadata(&path)?;
        let mtime = metadata.modified()?;

        self.templates.write().unwrap().register_template_file(name, &path)?;
        self.template_files.write().unwrap().insert(name.to_string(), (mtime, path));

        Ok(())
    }

    /// Checks if the template was modified since the last rendering
    /// if yes, it loads a the new version of the template
    pub fn render_template<T>(&self, name: &str, data: &T) -> Result<String, Error>
    where
        T: Serialize,
    {
        let path;
        let mtime;
        {
            let template_files = self.template_files.read().unwrap();
            let (old_mtime, old_path) = template_files.get(name).ok_or_else(|| format_err!("template not found"))?;

            mtime = metadata(old_path)?.modified()?;
            if mtime <= *old_mtime {
                return self.templates.read().unwrap().render(name, data).map_err(|err| format_err!("{}", err));
            }
            path = old_path.to_path_buf();
        }

        {
            let mut template_files = self.template_files.write().unwrap();
            let mut templates = self.templates.write().unwrap();

            templates.register_template_file(name, &path)?;
            template_files.insert(name.to_string(), (mtime, path));

            templates.render(name, data).map_err(|err| format_err!("{}", err))
        }
    }
}
