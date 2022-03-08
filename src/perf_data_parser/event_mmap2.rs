use super::perf_data_parser::ReadExt;
use bitflags::bitflags;
use std::io::Error as IOError;

pub trait ReadMmap2EventExt: ReadExt {
    fn read_mmap2_event(&mut self, object_path: &str) -> Result<(u64, Option<u64>), IOError> {
        let mut bytes_read = 65;
        let mut object_base_address = None;

        // TODO: properly parse some of these
        let _pid = self.read_u32()?;
        let _tid = self.read_u32()?;
        let addr = self.read_u64()?;
        let _len = self.read_u64()?;
        let _pgoff = self.read_u64()?;
        let _maj = self.read_u32()?;
        let _min = self.read_u32()?;
        let _ino = self.read_u64()?;
        let _ino_generation = self.read_u64()?;
        let prot = MemoryProtection::from_bits_truncate(self.read_u32()?);
        let _flags = self.read_u32()?;
        let mut filename = String::new();
        loop {
            let c = self.read_u8()?;
            if c != 0 {
                filename.push(c as char);
                bytes_read += 1;
            } else {
                break;
            }
        }
        // TODO: should I use the filename, or whole path?
        if filename.contains(object_path) && prot.contains(MemoryProtection::PROT_EXEC) {
            object_base_address = Some(addr);
        }
        // let _sample_id = todo!();
        // struct sample_id {
        //     { u32 pid, tid; }   /* if PERF_SAMPLE_TID set */
        //     { u64 time;     }   /* if PERF_SAMPLE_TIME set */
        //     { u64 id;       }   /* if PERF_SAMPLE_ID set */
        //     { u64 stream_id;}   /* if PERF_SAMPLE_STREAM_ID set  */
        //     { u32 cpu, res; }   /* if PERF_SAMPLE_CPU set */
        //     { u64 id;       }   /* if PERF_SAMPLE_IDENTIFIER set */
        // };
        Ok((bytes_read, object_base_address))
    }
}

bitflags! {
    struct MemoryProtection: u32 {
        const PROT_READ = 0x1;
        const PROT_WRITE = 0x2;
        const PROT_EXEC = 0x4;
    }
}
