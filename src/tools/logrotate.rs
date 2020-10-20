use std::path::{Path, PathBuf};
use std::fs::{File, rename};
use std::os::unix::io::FromRawFd;
use std::io::Read;

use anyhow::{bail, Error};
use nix::unistd;

use proxmox::tools::fs::{CreateOptions, make_tmp_file};

/// Used for rotating log files and iterating over them
pub struct LogRotate {
    base_path: PathBuf,
    compress: bool,
}

impl LogRotate {
    /// Creates a new instance if the path given is a valid file name
    /// (iow. does not end with ..)
    /// 'compress' decides if compresses files will be created on
    /// rotation, and if it will search '.zst' files when iterating
    pub fn new<P: AsRef<Path>>(path: P, compress: bool) -> Option<Self> {
        if path.as_ref().file_name().is_some() {
            Some(Self {
                base_path: path.as_ref().to_path_buf(),
                compress,
            })
        } else {
            None
        }
    }

    /// Returns an iterator over the logrotated file names that exist
    pub fn file_names(&self) -> LogRotateFileNames {
        LogRotateFileNames {
            base_path: self.base_path.clone(),
            count: 0,
            compress: self.compress
        }
    }

    /// Returns an iterator over the logrotated file handles
    pub fn files(&self) -> LogRotateFiles {
        LogRotateFiles {
            file_names: self.file_names(),
        }
    }

    fn compress(file: &PathBuf, options: &CreateOptions) -> Result<(), Error> {
        let mut source = File::open(file)?;
        let (fd, tmp_path) = make_tmp_file(file, options.clone())?;
        let target = unsafe { File::from_raw_fd(fd) };
        let mut encoder = match zstd::stream::write::Encoder::new(target, 0) {
            Ok(encoder) => encoder,
            Err(err) => {
                let _ = unistd::unlink(&tmp_path);
                bail!("creating zstd encoder failed - {}", err);
            }
        };

        if let Err(err) = std::io::copy(&mut source, &mut encoder) {
            let _ = unistd::unlink(&tmp_path);
            bail!("zstd encoding failed for file {:?} - {}", file, err);
        }

        if let Err(err) = encoder.finish() {
            let _ = unistd::unlink(&tmp_path);
            bail!("zstd finish failed for file {:?} - {}", file, err);
        }

        if let Err(err) = rename(&tmp_path, file) {
            let _ = unistd::unlink(&tmp_path);
            bail!("rename failed for file {:?} - {}", file, err);
        }
        Ok(())
    }

    /// Rotates the files up to 'max_files'
    /// if the 'compress' option was given it will compress the newest file
    ///
    /// e.g. rotates
    /// foo.2.zst => foo.3.zst
    /// foo.1.zst => foo.2.zst
    /// foo       => foo.1.zst
    ///           => foo
    pub fn do_rotate(&mut self, options: CreateOptions, max_files: Option<usize>) -> Result<(), Error> {
        let mut filenames: Vec<PathBuf> = self.file_names().collect();
        if filenames.is_empty() {
            return Ok(()); // no file means nothing to rotate
        }

        let mut next_filename = self.base_path.clone().canonicalize()?.into_os_string();

        if self.compress {
            next_filename.push(format!(".{}.zst", filenames.len()));
        } else {
            next_filename.push(format!(".{}", filenames.len()));
        }

        filenames.push(PathBuf::from(next_filename));
        let count = filenames.len();

        for i in (0..count-1).rev() {
            rename(&filenames[i], &filenames[i+1])?;
        }

        if self.compress {
            for i in 2..count-1 {
                if filenames[i].extension().unwrap_or(std::ffi::OsStr::new("")) != "zst" {
                    Self::compress(&filenames[i], &options)?;
                }
            }
        }

        if let Some(max_files) = max_files {
            for file in filenames.iter().skip(max_files) {
                if let Err(err) = unistd::unlink(file) {
                    eprintln!("could not remove {:?}: {}", &file, err);
                }
            }
        }

        Ok(())
    }

    pub fn rotate(
        &mut self,
        max_size: u64,
        options: Option<CreateOptions>,
        max_files: Option<usize>
    ) -> Result<bool, Error> {

        let options = match options {
            Some(options) => options,
            None => {
                let backup_user = crate::backup::backup_user()?;
                CreateOptions::new().owner(backup_user.uid).group(backup_user.gid)
            },
        };

        let metadata = match self.base_path.metadata() {
            Ok(metadata) => metadata,
            Err(err) if err.kind() == std::io::ErrorKind::NotFound => return Ok(false),
            Err(err) =>  bail!("unable to open task archive - {}", err),
        };

        if metadata.len() > max_size {
            self.do_rotate(options, max_files)?;
            Ok(true)
        } else {
            Ok(false)
        }
    }
}

/// Iterator over logrotated file names
pub struct LogRotateFileNames {
    base_path: PathBuf,
    count: usize,
    compress: bool,
}

impl Iterator for LogRotateFileNames {
    type Item = PathBuf;

    fn next(&mut self) -> Option<Self::Item> {
        if self.count > 0 {
            let mut path: std::ffi::OsString = self.base_path.clone().into();

            path.push(format!(".{}", self.count));
            self.count += 1;

            if Path::new(&path).is_file() {
                Some(path.into())
            } else if self.compress {
                path.push(".zst");
                if Path::new(&path).is_file() {
                    Some(path.into())
                } else {
                    None
                }
            } else {
                None
            }
        } else if self.base_path.is_file() {
            self.count += 1;
            Some(self.base_path.to_path_buf())
        } else {
            None
        }
    }
}

/// Iterator over logrotated files by returning a boxed reader
pub struct LogRotateFiles {
    file_names: LogRotateFileNames,
}

impl Iterator for LogRotateFiles {
    type Item = Box<dyn Read + Send>;

    fn next(&mut self) -> Option<Self::Item> {
        let filename = self.file_names.next()?;
        let file = File::open(&filename).ok()?;

        if filename.extension().unwrap_or(std::ffi::OsStr::new("")) == "zst" {
            let encoder = zstd::stream::read::Decoder::new(file).ok()?;
            return Some(Box::new(encoder));
        }

        Some(Box::new(file))
    }
}
