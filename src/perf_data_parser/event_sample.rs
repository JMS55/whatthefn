use super::perf_data_parser::{bit, ReadExt};
use super::symbolicator::Symbolicator;
use bitflags::bitflags;
use std::error::Error;

pub trait ReadSampleEventExt: ReadExt {
    fn read_sample_event<F: FnMut(Sample)>(
        &mut self,
        sample_type: SampleType,
        symbolicator: &mut Symbolicator,
        mut process_sample: F,
    ) -> Result<u64, Box<dyn Error>> {
        let mut tid = None;
        let mut timestamp = None;
        let mut callchain = None;

        // TODO: Correctly parse this event (some fields not listed yet, or incomplete)
        let mut bytes_read = 0;
        if sample_type.contains(SampleType::IDENTIFIER) {
            let _sample_id = self.read_u64()?;
            bytes_read += 8;
        }
        if sample_type.contains(SampleType::IP) {
            let _ip = self.read_u64()?;
            bytes_read += 8;
        }
        if sample_type.contains(SampleType::TID) {
            let _pid = self.read_u32()?;
            tid = Some(self.read_u32()?);
            bytes_read += 8;
        }
        if sample_type.contains(SampleType::TIME) {
            let time = self.read_u64()?;
            timestamp = Some(time);
            bytes_read += 8;
        }
        if sample_type.contains(SampleType::ADDR) {
            let _addr = self.read_u64()?;
            bytes_read += 8;
        }
        if sample_type.contains(SampleType::ID) {
            let _id = self.read_u64()?;
            bytes_read += 8;
        }
        if sample_type.contains(SampleType::STREAM_ID) {
            let _stream_id = self.read_u64()?;
            bytes_read += 8;
        }
        // TODO: Figure out how to enable this via perf record, dosen't seem to be on by default
        if sample_type.contains(SampleType::CPU) {
            let _cpu = self.read_u32()?;
            let _res = self.read_u32()?;
            bytes_read += 8;
        }
        if sample_type.contains(SampleType::PERIOD) {
            let _period = self.read_u64()?;
            bytes_read += 8;
        }
        if sample_type.contains(SampleType::READ) {
            todo!();
        }
        if sample_type.contains(SampleType::CALLCHAIN) {
            let nr = self.read_u64()?;
            let mut ips = Vec::new();
            for _ in 1..=nr {
                let ip = self.read_u64()?;
                ips.push(ip);
            }
            callchain = Some(ips);
            bytes_read += 8 * (nr + 1);
        }
        if sample_type.contains(SampleType::RAW) {
            let size = self.read_u32()?;
            let n = ((size as i32 + 7) & (-8)) as u32;
            for _ in 1..=n {
                let _data = self.read_u8()?;
            }
        }
        // TODO: Uncomment
        // if sample_type.contains(SampleType::BRANCH_STACK) {
        //     todo!();
        // }
        // if sample_type.contains(SampleType::REGS_USER) {
        //     todo!();
        // }
        // if sample_type.contains(SampleType::STACK_USER) {
        //     let size = self.read_u64()?;
        //     for _ in 1..=size {
        //         let _data = self.read_u8()?;
        //     }
        //     if size != 0 {
        //         let _dyn_size = self.read_u64()?;
        //     }
        //     bytes_read += 8 + size + if size != 0 { 8 } else { 0 };
        // }
        // if sample_type.contains(SampleType::WEIGHT) {
        //     todo!();
        // }
        // if sample_type.contains(SampleType::DATA_SRC) {
        //     todo!();
        // }
        // if sample_type.contains(SampleType::TRANSACTION) {
        //     todo!();
        // }
        // if sample_type.contains(SampleType::REGS_INTR) {
        //     todo!();
        // }
        // if sample_type.contains(SampleType::PHYS_ADDR) {
        //     let _phys_addr = self.read_u64()?;
        // }
        // if sample_type.contains(SampleType::CGROUP) {
        //     todo!();
        // }

        if tid.is_some() && timestamp.is_some() && callchain.is_some() {
            let stacktrace = callchain
                .unwrap()
                .into_iter()
                .map(|ip| symbolicator.lookup_symbol(ip))
                .collect::<Result<_, _>>()?;
            let sample = Sample {
                tid: tid.unwrap(),
                timestamp: timestamp.unwrap(),
                stacktrace,
            };
            (process_sample)(sample);
        }

        Ok(bytes_read)
    }
}

pub struct Sample {
    pub tid: u32,
    pub timestamp: u64,
    pub stacktrace: Box<[String]>,
}

bitflags! {
    pub struct SampleType: u64 {
        const IP = bit(0);
        const TID = bit(1);
        const TIME = bit(2);
        const ADDR = bit(3);
        const READ = bit(4);
        const CALLCHAIN = bit(5);
        const ID = bit(6);
        const CPU = bit(7);
        const PERIOD = bit(8);
        const STREAM_ID = bit(9);
        const RAW = bit(10);
        const BRANCH_STACK = bit(11);
        const REGS_USER = bit(12);
        const STACK_USER = bit(13);
        const WEIGHT = bit(14);
        const DATA_SRC = bit(15);
        const IDENTIFIER = bit(16);
        const TRANSACTION = bit(17);
        const REGS_INTR = bit(18);
        const PHYS_ADDR = bit(19);
        const AUX = bit(20);
        const CGROUP = bit(21);
        const DATA_PAGE_SIZE = bit(22);
        const CODE_PAGE_SIZE = bit(23);
        const WEIGHT_STRUCT = bit(24);
    }
}
