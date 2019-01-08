//! *catar* format decoder.
//!
//! This module contain the code to decode *catar* archive files.

use failure::*;
use endian_trait::Endian;

use super::format_definition::*;
use crate::tools;

use std::io::{Read, Write, Seek, SeekFrom};
use std::path::{Path, PathBuf};

use std::os::unix::io::AsRawFd;
use std::os::unix::io::RawFd;
use std::os::unix::io::FromRawFd;
use std::os::unix::ffi::{OsStrExt, OsStringExt};
use std::ffi::{OsStr, OsString};

use nix::fcntl::OFlag;
use nix::sys::stat::Mode;
use nix::errno::Errno;
use nix::NixPath;

pub struct CaDirectoryEntry {
    start: u64,
    end: u64,
    pub filename: OsString,
    pub entry: CaFormatEntry,
}

// This one needs Read+Seek (we may want one without Seek?)
pub struct CaTarDecoder<'a, R: Read + Seek> {
    reader: &'a mut R,
    root_start: u64,
    root_end: u64,
}

const HEADER_SIZE: u64 = std::mem::size_of::<CaFormatHeader>() as u64;

impl <'a, R: Read + Seek> CaTarDecoder<'a, R> {

    pub fn new(reader: &'a mut R) -> Result<Self, Error> {

        let root_end = reader.seek(SeekFrom::End(0))?;

        Ok(Self {
            reader: reader,
            root_start: 0,
            root_end: root_end,
        })
    }

    pub fn root(&self) -> CaDirectoryEntry {
        CaDirectoryEntry {
            start: self.root_start,
            end: self.root_end,
            filename: OsString::new(), // Empty
            entry: CaFormatEntry {
                feature_flags: 0,
                mode: 0,
                flags: 0,
                uid: 0,
                gid: 0,
                mtime: 0,
            }
        }
    }

    fn read_item<T: Endian>(&mut self) -> Result<T, Error> {

        let mut result: T = unsafe { std::mem::uninitialized() };

        let buffer = unsafe { std::slice::from_raw_parts_mut(
            &mut result as *mut T as *mut u8,
            std::mem::size_of::<T>()
        )};

        self.reader.read_exact(buffer)?;

        Ok(result.from_le())
    }

    fn read_symlink(&mut self, size: u64) -> Result<PathBuf, Error> {
        if size < (HEADER_SIZE + 2) {
             bail!("dectected short symlink target.");
        }
        let target_len = size - HEADER_SIZE;

        if target_len > (libc::PATH_MAX as u64) {
            bail!("symlink target too long ({}).", target_len);
        }

        let mut buffer = vec![0u8; target_len as usize];
        self.reader.read_exact(&mut buffer)?;

        let last_byte = buffer.pop().unwrap();
        if last_byte != 0u8 {
            bail!("symlink target not nul terminated.");
        }

        Ok(PathBuf::from(std::ffi::OsString::from_vec(buffer)))
    }

    fn read_filename(&mut self, size: u64) -> Result<OsString, Error> {
        if size < (HEADER_SIZE + 2) {
            bail!("dectected short filename");
        }
        let name_len = size - HEADER_SIZE;

        if name_len > ((libc::FILENAME_MAX as u64) + 1) {
            bail!("filename too long ({}).", name_len);
        }

        let mut buffer = vec![0u8; name_len as usize];
        self.reader.read_exact(&mut buffer)?;

        let last_byte = buffer.pop().unwrap();
        if last_byte != 0u8 {
            bail!("filename entry not nul terminated.");
        }

        // fixme: check filename is relative (not starting with /)

        Ok(std::ffi::OsString::from_vec(buffer))
    }

    pub fn restore<F: Fn(&Path) -> Result<(), Error>>(
        &mut self,
        dir: &CaDirectoryEntry,
        callback: F,
    ) -> Result<(), Error> {

        let start = dir.start;

        self.reader.seek(SeekFrom::Start(start))?;

        let base = ".";

        let mut path = PathBuf::from(base);

        let dir = match nix::dir::Dir::open(&path, nix::fcntl::OFlag::O_DIRECTORY,  nix::sys::stat::Mode::empty()) {
            Ok(dir) => dir,
            Err(err) => bail!("unable to open base directory - {}", err),
        };

        let restore_dir = "restoretest";
        path.push(restore_dir);

        self.restore_sequential(&mut path, &OsString::from(restore_dir), &dir, &callback)?;

        Ok(())
    }

    pub fn restore_sequential<F: Fn(&Path) -> Result<(), Error>>(
        &mut self,
        path: &mut PathBuf, // user for error reporting
        filename: &OsStr,  // repeats path last component
        parent: &nix::dir::Dir,
        callback: &F,
    ) -> Result<(), Error> {

        let parent_fd = parent.as_raw_fd();

        // read ENTRY first
        let head: CaFormatHeader = self.read_item()?;
        check_ca_header::<CaFormatEntry>(&head, CA_FORMAT_ENTRY)?;
        let entry: CaFormatEntry = self.read_item()?;

        let mode = entry.mode as u32; //fixme: upper 32bits?

        if (mode & libc::S_IFMT) == libc::S_IFDIR {
            let dir = match dir_mkdirat(parent_fd, filename) {
                Ok(dir) => dir,
                Err(err) => bail!("unable to open directory {:?} - {}", path, err),
            };

            //fixme: restore permission, acls, xattr, ...

            loop {
                let head: CaFormatHeader = self.read_item()?;
                match head.htype {
                    CA_FORMAT_FILENAME => {
                        let name = self.read_filename(head.size)?;
                        path.push(&name);
                        println!("NAME: {:?}", path);
                        self.restore_sequential(path, &name, &dir, callback)?;
                        path.pop();
                    }
                    CA_FORMAT_GOODBYE => {
                        println!("Skip Goodbye");
                        if head.size < HEADER_SIZE { bail!("detected short goodbye table"); }
                        self.reader.seek(SeekFrom::Current((head.size - HEADER_SIZE) as i64))?;
                        return Ok(());
                    }
                    _ => {
                        bail!("got unknown header type inside directory entry {:016x}", head.htype);
                    }
                }
            }
        }

        if (mode & libc::S_IFMT) == libc::S_IFLNK {
            // fixme: create symlink
            //fixme: restore permission, acls, xattr, ...
            let head: CaFormatHeader = self.read_item()?;
            match head.htype {
                CA_FORMAT_SYMLINK => {
                    if ((mode & libc::S_IFMT) != libc::S_IFLNK) {
                        bail!("detected unexpected symlink item.");
                    }
                    let target = self.read_symlink(head.size)?;
                    println!("TARGET: {:?}", target);
                    if let Err(err) = symlinkat(&target, parent_fd, filename) {
                        bail!("create symlink {:?} failed - {}", path, err);
                    }
                }
                 _ => {
                     bail!("got unknown header type inside symlink entry {:016x}", head.htype);
                 }
            }
            return Ok(());
        }

        if (mode & libc::S_IFMT) == libc::S_IFREG {

            let mut read_buffer: [u8; 64*1024] = unsafe { std::mem::uninitialized() };

            let flags = OFlag::O_CREAT|OFlag::O_WRONLY|OFlag::O_EXCL;
            let open_mode =  Mode::from_bits_truncate(0o0600 | mode);

            let mut file = match file_openat(parent_fd, filename, flags, open_mode) {
                Ok(file) => file,
                Err(err) => bail!("open file {:?} failed - {}", path, err),
            };

            //fixme: restore permission, acls, xattr, ...

            let head: CaFormatHeader = self.read_item()?;
            match head.htype {
                CA_FORMAT_PAYLOAD => {
                     if head.size < HEADER_SIZE {
                        bail!("detected short payload");
                    }
                    let need = (head.size - HEADER_SIZE) as usize;
                    //self.reader.seek(SeekFrom::Current(need as i64))?;

                    // fixme:: create file

                    let mut done = 0;
                    while (done < need)  {
                        let todo = need - done;
                        let n = if todo > read_buffer.len() { read_buffer.len() } else { todo };
                        let data = &mut read_buffer[..n];
                        self.reader.read_exact(data)?;
                        file.write_all(data)?;
                        done += n;
                    }
                }
                _ => {
                    bail!("got unknown header type for file entry {:016x}", head.htype);
                }
            }

            return Ok(());
        }

        Ok(())
    }

    fn read_directory_entry(&mut self, start: u64, end: u64) -> Result<CaDirectoryEntry, Error> {

        self.reader.seek(SeekFrom::Start(start))?;
        let mut buffer = [0u8; HEADER_SIZE as usize];
        self.reader.read_exact(&mut buffer)?;
        let head = tools::map_struct::<CaFormatHeader>(&buffer)?;

        if u64::from_le(head.htype) != CA_FORMAT_FILENAME {
            bail!("wrong filename header type for object [{}..{}]", start, end);
        }

        let mut name_len = u64::from_le(head.size);

        let entry_start = start + name_len;

        let filename = self.read_filename(name_len)?;

        let head: CaFormatHeader = self.read_item()?;
        check_ca_header::<CaFormatEntry>(&head, CA_FORMAT_ENTRY)?;
        let entry: CaFormatEntry = self.read_item()?;

        Ok(CaDirectoryEntry {
            start: entry_start,
            end: end,
            filename: filename,
            entry: CaFormatEntry {
                feature_flags: u64::from_le(entry.feature_flags),
                mode: u64::from_le(entry.mode),
                flags: u64::from_le(entry.flags),
                uid: u64::from_le(entry.uid),
                gid: u64::from_le(entry.gid),
                mtime: u64::from_le(entry.mtime),
            },
        })
    }

    pub fn list_dir(&mut self, dir: &CaDirectoryEntry) -> Result<Vec<CaDirectoryEntry>, Error> {

        const GOODBYE_ITEM_SIZE: u64 = std::mem::size_of::<CaFormatGoodbyeItem>() as u64;

        let start = dir.start;
        let end = dir.end;

        //println!("list_dir1: {} {}", start, end);

        if (end - start) < (HEADER_SIZE + GOODBYE_ITEM_SIZE) {
            bail!("detected short object [{}..{}]", start, end);
        }

        self.reader.seek(SeekFrom::Start(end - GOODBYE_ITEM_SIZE))?;
        let mut buffer = [0u8; GOODBYE_ITEM_SIZE as usize];
        self.reader.read_exact(&mut buffer)?;

        let item = tools::map_struct::<CaFormatGoodbyeItem>(&buffer)?;

        if u64::from_le(item.hash) != CA_FORMAT_GOODBYE_TAIL_MARKER {
            bail!("missing goodbye tail marker for object [{}..{}]", start, end);
        }

        let goodbye_table_size = u64::from_le(item.size);
        if goodbye_table_size < (HEADER_SIZE + GOODBYE_ITEM_SIZE) {
            bail!("short goodbye table size for object [{}..{}]", start, end);

        }
        let goodbye_inner_size = goodbye_table_size - HEADER_SIZE - GOODBYE_ITEM_SIZE;
        if (goodbye_inner_size % GOODBYE_ITEM_SIZE) != 0 {
            bail!("wrong goodbye inner table size for entry [{}..{}]", start, end);
        }

        let goodbye_start = end - goodbye_table_size;

        if u64::from_le(item.offset) != (goodbye_start - start) {
            println!("DEBUG: {} {}", u64::from_le(item.offset), goodbye_start - start);
            bail!("wrong offset in goodbye tail marker for entry [{}..{}]", start, end);
        }

        self.reader.seek(SeekFrom::Start(goodbye_start))?;
        let mut buffer = [0u8; HEADER_SIZE as usize];
        self.reader.read_exact(&mut buffer)?;
        let head = tools::map_struct::<CaFormatHeader>(&buffer)?;

        if u64::from_le(head.htype) != CA_FORMAT_GOODBYE {
            bail!("wrong goodbye table header type for entry [{}..{}]", start, end);
        }

        if u64::from_le(head.size) != goodbye_table_size {
            bail!("wrong goodbye table size for entry [{}..{}]", start, end);
        }

        let mut buffer = [0u8; GOODBYE_ITEM_SIZE as usize];

        let mut range_list = Vec::new();

        for i in 0..goodbye_inner_size/GOODBYE_ITEM_SIZE {
            self.reader.read_exact(&mut buffer)?;
            let item = tools::map_struct::<CaFormatGoodbyeItem>(&buffer)?;
            let item_offset = u64::from_le(item.offset);
            if item_offset > (goodbye_start - start) {
                bail!("goodbye entry {} offset out of range [{}..{}] {} {} {}",
                      i, start, end, item_offset, goodbye_start, start);
            }
            let item_start = goodbye_start - item_offset;
            let item_hash = u64::from_le(item.hash);
            let item_end = item_start + u64::from_le(item.size);
            if item_end > goodbye_start {
                bail!("goodbye entry {} end out of range [{}..{}]",
                      i, start, end);
            }

            range_list.push((item_start, item_end));
        }

        let mut result = vec![];

        for (item_start, item_end) in range_list {
            let entry = self.read_directory_entry(item_start, item_end)?;
            //println!("ENTRY: {} {} {:?}", item_start, item_end, entry.filename);
            result.push(entry);
        }

        Ok(result)
    }

    pub fn print_filenames<W: std::io::Write>(
        &mut self,
        output: &mut W,
        prefix: &mut PathBuf,
        dir: &CaDirectoryEntry,
    ) -> Result<(), Error> {

        let mut list = self.list_dir(dir)?;

        list.sort_unstable_by(|a, b| a.filename.cmp(&b.filename));

        for item in &list {

            prefix.push(item.filename.clone());

            let mode = item.entry.mode as u32;

            let osstr: &OsStr =  prefix.as_ref();
            output.write(osstr.as_bytes())?;
            output.write(b"\n")?;

            if (mode & libc::S_IFMT) == libc::S_IFDIR {
                self.print_filenames(output, prefix, item)?;
            } else if (mode & libc::S_IFMT) == libc::S_IFREG {
            } else if (mode & libc::S_IFMT) == libc::S_IFLNK {
            } else {
                bail!("unknown item mode/type for {:?}", prefix);
            }

            prefix.pop();
        }

        Ok(())
    }
}

fn file_openat(parent: RawFd, filename: &OsStr, flags: OFlag, mode: Mode) -> Result<std::fs::File, Error> {

    let fd = filename.with_nix_path(|cstr| unsafe {
        nix::fcntl::openat(parent, cstr.as_ref(), flags, mode)
    })??;

    let file = unsafe { std::fs::File::from_raw_fd(fd) };

    Ok(file)
}

fn dir_mkdirat(parent: RawFd, filename: &OsStr) -> Result<nix::dir::Dir, Error> {

    // call mkdirat first
    let res = filename.with_nix_path(|cstr| unsafe {
        libc::mkdirat(parent, cstr.as_ptr(), libc::S_IRWXU)
    })?;
    Errno::result(res)?;

    let dir = nix::dir::Dir::openat(parent, filename, OFlag::O_DIRECTORY,  Mode::empty())?;

    Ok(dir)
}

fn symlinkat(target: &Path, parent: RawFd, linkname: &OsStr) -> Result<(), Error> {

    target.with_nix_path(|target| {
        linkname.with_nix_path(|linkname| {
            let res = unsafe { libc::symlinkat(target.as_ptr(), parent, linkname.as_ptr()) };
            Errno::result(res)?;
            Ok(())
        })?
    })?
}
