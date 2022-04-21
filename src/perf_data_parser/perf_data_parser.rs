use super::event_mmap2::ReadMmap2EventExt;
use super::event_sample::{ReadSampleEventExt, Sample, SampleType};
use super::symbolicator::Symbolicator;
use bitflags::bitflags;
use std::error::Error;
use std::fs::File;
use std::io::{BufReader, Error as IOError, ErrorKind, Read, Seek, SeekFrom};
use std::path::Path;

pub fn convert_perf_data_to_wtf<P1: AsRef<Path>, P2: AsRef<Path> + Copy>(
    perf_data_path: P1,
    binary_profiled_path: P2,
) -> Result<(), Box<dyn Error>> {
    let mut file = BufReader::new(File::open(perf_data_path)?);

    let header = file.read_header()?;
    let attributes = file.read_attribute_section(&header)?;

    // TODO: Pass all attributes to read_data_section(), pick the correct one for each event
    let sample_type = attributes
        .get(0)
        .ok_or_else(|| IOError::new(ErrorKind::InvalidData, "No attributes found"))?
        .sample_type;
    file.read_data_section(&header, sample_type, binary_profiled_path, |sample| {
        // TODO: This should send sample to be processed on another thread
        dbg!(sample.stacktrace);
    })?;

    Ok(())
}

impl ReadExt for BufReader<File> {}
impl ReadMmap2EventExt for BufReader<File> {}
impl ReadSampleEventExt for BufReader<File> {}

trait ReadSectionExt: ReadExt + ReadMmap2EventExt + ReadSampleEventExt {
    fn read_attribute_section(&mut self, header: &Header) -> Result<Vec<Attribute>, IOError>;

    fn read_data_section<P: AsRef<Path> + Copy, F: FnMut(Sample)>(
        &mut self,
        header: &Header,
        sample_type: SampleType,
        binary_profiled_path: P,
        process_sample: F,
    ) -> Result<(), Box<dyn Error>>;
}

impl ReadSectionExt for BufReader<File> {
    fn read_attribute_section(&mut self, header: &Header) -> Result<Vec<Attribute>, IOError> {
        self.seek(SeekFrom::Start(header.attribute_section.offset))?;

        let mut attributes = Vec::new();
        let mut bytes_read = 0;
        while bytes_read < header.attribute_section.size {
            let _type = self.read_u32()?;
            let _size = self.read_u32()?;
            let _config = self.read_u64()?;
            let sample_period_or_frequency = self.read_u64()?;
            let sample_type = SampleType::from_bits_truncate(self.read_u64()?);
            // TODO: Parse the rest of the attribute data

            attributes.push(Attribute {
                frequency: sample_period_or_frequency, // TODO: Don't assume frequency
                sample_type,
            });

            self.seek_relative(header.attribute_size as i64 - 32)?;
            bytes_read += header.attribute_size;
        }

        Ok(attributes)
    }

    // TODO: How to generically structure specifying which events and fields the caller is interested in?
    // and then seek past the rest
    fn read_data_section<P: AsRef<Path> + Copy, F: FnMut(Sample)>(
        &mut self,
        header: &Header,
        sample_type: SampleType,
        binary_profiled_path: P,
        mut process_sample: F,
    ) -> Result<(), Box<dyn Error>> {
        self.seek(SeekFrom::Start(header.data_section.offset))?;

        let mut bytes_read = 0;
        let mut symbolicator = None;

        while bytes_read < header.data_section.size {
            let event_header = self.read_event_header()?;

            let mut event_bytes_read = 0;
            match event_header.event_type {
                // TODO: Parse the rest of the event types
                EventType::MMAP2 => {
                    // TODO: Cleanup this part
                    let (bytes, object_base_address) = self.read_mmap2_event(
                        binary_profiled_path
                            .as_ref()
                            .file_name()
                            .unwrap()
                            .to_str()
                            .unwrap(),
                    )?;
                    event_bytes_read = bytes;
                    if let Some(object_base_address) = object_base_address {
                        symbolicator = Some(Symbolicator::new(
                            binary_profiled_path,
                            object_base_address,
                        )?);
                    }
                }
                EventType::SAMPLE if symbolicator.is_some() => {
                    event_bytes_read = self.read_sample_event(
                        sample_type,
                        symbolicator.as_mut().unwrap(),
                        &mut process_sample,
                    )?;
                }
                _ => {}
            }

            self.seek_relative(event_header.event_size as i64 - 8 - event_bytes_read as i64)?;
            bytes_read += event_header.event_size as u64;
        }

        Ok(())
    }
}

pub trait ReadExt: Read + Seek {
    fn read_u8(&mut self) -> Result<u8, IOError> {
        let mut bytes = [0u8; 1];
        self.read_exact(&mut bytes)?;
        Ok(u8::from_ne_bytes(bytes))
    }

    fn read_u16(&mut self) -> Result<u16, IOError> {
        let mut bytes = [0u8; 2];
        self.read_exact(&mut bytes)?;
        Ok(u16::from_ne_bytes(bytes))
    }

    fn read_u32(&mut self) -> Result<u32, IOError> {
        let mut bytes = [0u8; 4];
        self.read_exact(&mut bytes)?;
        Ok(u32::from_ne_bytes(bytes))
    }

    fn read_u64(&mut self) -> Result<u64, IOError> {
        let mut bytes = [0u8; 8];
        self.read_exact(&mut bytes)?;
        Ok(u64::from_ne_bytes(bytes))
    }

    fn read_version(&mut self) -> Result<String, IOError> {
        let mut bytes = [0u8; 8];
        self.read_exact(&mut bytes)?;
        String::from_utf8(bytes.to_vec()).map_err(|e| IOError::new(ErrorKind::InvalidData, e))
    }

    fn read_section_info(&mut self) -> Result<SectionInfo, IOError> {
        let offset = self.read_u64()?;
        let size = self.read_u64()?;
        Ok(SectionInfo { offset, size })
    }

    fn read_extra_headers_present(&mut self) -> Result<ExtraHeadersPresent, IOError> {
        Ok(ExtraHeadersPresent::from_bits_truncate(self.read_u64()?))
    }

    fn read_header(&mut self) -> Result<Header, IOError> {
        let version = self.read_version()?;
        if version != "PERFILE2" {
            let error_message =
                format!("Invalid perf.data version: Got {version}, expected PERFILE2");
            return Err(IOError::new(ErrorKind::InvalidInput, error_message));
        }

        let _header_size = self.read_u64()?;
        let attribute_size = self.read_u64()?;
        let attribute_section = self.read_section_info()?;
        let data_section = self.read_section_info()?;
        let _event_types_section = self.read_section_info()?;
        let extra_headers_present = self.read_extra_headers_present()?;

        Ok(Header {
            attribute_size,
            attribute_section,
            data_section,
            extra_headers_present,
        })
    }

    fn read_event_header(&mut self) -> Result<EventHeader, IOError> {
        let event_type = self.read_u32()?.into();
        let _misc = self.read_u16()?;
        let event_size = self.read_u16()?;
        Ok(EventHeader {
            event_type,
            event_size,
        })
    }

    fn read_extra_headers(&mut self, header: &Header) -> Result<(), IOError> {
        // TODO
        Ok(())
    }
}

pub struct Header {
    attribute_size: u64,
    attribute_section: SectionInfo,
    data_section: SectionInfo,
    extra_headers_present: ExtraHeadersPresent,
}

pub struct SectionInfo {
    offset: u64,
    size: u64,
}

bitflags! {
    pub struct ExtraHeadersPresent: u64 {
        const TRACING_DATA = bit(1);
        const BUILD_ID = bit(2);
        const HOSTNAME = bit(3);
        const OSRELEASE = bit(4);
        const VERSION = bit(5);
        const ARCH = bit(6);
        const NRCPUS = bit(7);
        const CPUDESC = bit(8);
        const CPUID = bit(9);
        const TOTAL_MEM = bit(10);
        const CMDLINE = bit(11);
        const EVENT_DESC = bit(12);
        const CPU_TOPOLOGY = bit(13);
        const NUMA_TOPOLOGY = bit(14);
        const BRANCH_STACK = bit(15);
        const PMU_MAPPINGS = bit(16);
        const GROUP_DESC = bit(17);
        const AUXTRACE = bit(18);
        const STAT = bit(19);
        const CACHE = bit(20);
        const SAMPLE_TIME = bit(21);
        const MEM_TOPOLOGY = bit(22);
        const CLOCKID = bit(23);
        const DIR_FORMAT = bit(24);
        const BPF_PROG_INFO = bit(25);
        const BPF_BTF = bit(26);
        const COMPRESSED = bit(27);
        const CPU_PMU_CAPS = bit(28);
        const CLOCK_DATA = bit(29);
        const HYBRID_TOPOLOGY = bit(30);
        const HYBRID_CPU_PMU_CAPS = bit(31);
    }
}

struct Attribute {
    frequency: u64,
    sample_type: SampleType,
}

pub struct EventHeader {
    event_type: EventType,
    event_size: u16,
}

#[allow(non_camel_case_types)]
#[derive(PartialEq, Eq)]
enum EventType {
    MMAP,
    LOST,
    COMM,
    EXIT,
    THROTTLE,
    UNTHROTTLE,
    FORK,
    READ,
    SAMPLE,
    MMAP2,
    AUX,
    ITRACE_START,
    LOST_SAMPLES,
    SWITCH,
    SWITCH_CPU_WIDE,
    NAMESPACES,
    KSYMBOL,
    BPF_EVENT,
    CGROUP,
    TEXT_POKE,
    AUX_OUTPUT_HW_ID,

    HEADER_ATTR,
    HEADER_EVENT_TYPE,
    HEADER_TRACING_DATA,
    HEADER_BUILD_ID,
    FINISHED_ROUND,
    ID_INDEX,
    AUXTRACE_INFO,
    AUXTRACE,
    AUXTRACE_ERROR,
    THREAD_MAP,
    CPU_MAP,
    STAT_CONFIG,
    STAT,
    STAT_ROUND,
    EVENT_UPDATE,
    TIME_CONV,
    HEADER_FEATURE,
    COMPRESSED,
}

impl From<u32> for EventType {
    fn from(n: u32) -> Self {
        use EventType::*;
        match n {
            1 => MMAP,
            2 => LOST,
            3 => COMM,
            4 => EXIT,
            5 => THROTTLE,
            6 => UNTHROTTLE,
            7 => FORK,
            8 => READ,
            9 => SAMPLE,
            10 => MMAP2,
            11 => AUX,
            12 => ITRACE_START,
            13 => LOST_SAMPLES,
            14 => SWITCH,
            15 => SWITCH_CPU_WIDE,
            16 => NAMESPACES,
            17 => KSYMBOL,
            18 => BPF_EVENT,
            19 => CGROUP,
            20 => TEXT_POKE,
            21 => AUX_OUTPUT_HW_ID,

            64 => HEADER_ATTR,
            65 => HEADER_EVENT_TYPE,
            66 => HEADER_TRACING_DATA,
            67 => HEADER_BUILD_ID,
            68 => FINISHED_ROUND,
            69 => ID_INDEX,
            70 => AUXTRACE_INFO,
            71 => AUXTRACE,
            72 => AUXTRACE_ERROR,
            73 => THREAD_MAP,
            74 => CPU_MAP,
            75 => STAT_CONFIG,
            76 => STAT,
            77 => STAT_ROUND,
            78 => EVENT_UPDATE,
            79 => TIME_CONV,
            80 => HEADER_FEATURE,
            81 => COMPRESSED,
            _ => unreachable!(),
        }
    }
}

pub const fn bit(n: u64) -> u64 {
    1 << n
}
