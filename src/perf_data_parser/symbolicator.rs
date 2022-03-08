use addr2line::gimli::{EndianReader, RunTimeEndian};
use addr2line::object::{File as ObjectFile, Object, ObjectSection};
use addr2line::Context;
use std::error::Error;
use std::fs;
use std::io::{Error as IOError, ErrorKind};
use std::path::Path;
use std::rc::Rc;

pub struct Symbolicator {
    context: Context<EndianReader<RunTimeEndian, Rc<[u8]>>>,
    object_text_section_address: u64,
}

impl Symbolicator {
    pub fn new<P: AsRef<Path>>(
        object_path: P,
        object_base_address: u64,
    ) -> Result<Self, Box<dyn Error>> {
        let object_bytes = fs::read(object_path)?;
        let object_file = ObjectFile::parse(object_bytes.as_slice())?;
        let context = Context::new(&object_file)?;

        let object_text_section = object_file
            .section_by_name(".text")
            .ok_or(IOError::new(ErrorKind::NotFound, ".text section not found"))?;
        dbg!(object_text_section.address());
        let object_text_section_address =
            object_base_address + object_text_section.address() - 0x7D1E40; // TODO

        Ok(Self {
            context,
            object_text_section_address,
        })
    }

    // TODO: Is it possible to hand out &str?
    pub fn lookup_symbol(&mut self, instruction_pointer: u64) -> Result<String, Box<dyn Error>> {
        let mut frames = self
            .context
            .find_frames(instruction_pointer - self.object_text_section_address)?;
        // TODO: Correctly handle inline functions, see addr2line docs
        while let Some(frame) = frames.next()? {
            return Ok(frame.function.unwrap().demangle()?.to_string());
        }
        Ok("[unknown]".to_string())
    }
}

// a37910
