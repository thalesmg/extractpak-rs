// https://github.com/timfel/monkey/blob/master/extractpak.c

use std::{env, error::Error, fs::File, io::{BufRead, BufReader, Seek, SeekFrom, Write}, path::{Path, PathBuf}};
use std::fs;

type Dword = u32;

const MAGIC: &'static str = "KAPL";

#[derive(Debug, PartialEq)]
struct PakHeader {
    magic: Dword,
    version: f32,
    index_start: Dword,
    file_entries_start: Dword,
    file_names_start: Dword,
    data_start: Dword,
    index_size: Dword,
    file_entries_size: Dword,
    file_names_size: Dword,
    data_size: Dword,
}

impl PakHeader {
    fn read_parse<R: BufRead>(reader: &mut R) -> Result<Self, Box<dyn Error>> {
        const SZ: usize = size_of::<PakHeader>();
        let mut buf = [0; SZ];
        reader.read_exact(&mut buf)?;
        let magic = std::str::from_utf8(&buf[0..4])?;
        debug_assert_eq!(magic, MAGIC);
        Ok(PakHeader {
            magic: to_dword(&buf, 0)?,
            version: f32::from_le_bytes(buf[4..8].try_into()?),
            index_start: to_dword(&buf, 2)?,
            file_entries_start: to_dword(&buf, 3)?,
            file_names_start: to_dword(&buf, 4)?,
            data_start: to_dword(&buf, 5)?,
            index_size: to_dword(&buf, 6)?,
            file_entries_size: to_dword(&buf, 7)?,
            file_names_size: to_dword(&buf, 8)?,
            data_size: to_dword(&buf, 9)?,
        })
    }
}

#[derive(Debug, PartialEq)]
struct PakFileEntry {
    data_pos: Dword,
    filename_pos: Dword,
    data_size: Dword,
    data_size2: Dword,
    compressed: Dword,
}

impl PakFileEntry {
    fn read_parse<R: BufRead>(reader: &mut R) -> Result<Self, Box<dyn Error>> {
        const SZ: usize = size_of::<PakFileEntry>();
        let mut buf = [0; SZ];
        reader.read_exact(&mut buf)?;
        Ok(Self {
            data_pos: to_dword(&buf, 0)?,
            filename_pos: to_dword(&buf, 1)?,
            data_size: to_dword(&buf, 2)?,
            data_size2: to_dword(&buf, 3)?,
            compressed: to_dword(&buf, 4)?,
        })
    }
}

struct PixelFormat {
    size: Dword,
    flags: Dword,
    four_cc: Dword,
    rgb_bit_count: Dword,
    r_bit_mask: Dword,
    g_bit_mask: Dword,
    b_bit_mask: Dword,
    alpha_bit_mask: Dword,
}

struct Caps {
    caps1: Dword,
    caps2: Dword,
    ddsx: Dword,
    reserved: Dword,
}

struct DDSHeader {
    magic: Dword,
    size: Dword,
    flags: Dword,
    height: Dword,
    width: Dword,
    pitch_or_linear_size: Dword,
    depth: Dword,
    mip_map_count: Dword,
    reserved: [Dword; 11],
    // DDPixelFormat
    pixel_format: PixelFormat,
    caps: Caps,
    reserved2: Dword,
}

const DDSD_CAPS: Dword = 0x00000001;
const DDSD_HEIGHT: Dword = 0x00000002;
const DDSD_WIDTH: Dword = 0x00000004;
const DDSD_PIXELFORMAT: Dword = 0x00001000;
const DDSD_LINEARSIZE: Dword = 0x00080000;

const DDPF_FOURCC: Dword = 0x00000004;

const DDSCAPS_TEXTURE: Dword = 0x00001000;

fn to_dword(buf: &[u8], i: usize) -> Result<Dword, Box<dyn Error>> {
    Ok(u32::from_le_bytes(buf[(i*4)..((i+1)*4)].try_into()?))
}

impl DDSHeader {
    fn parse(dds_data: &[u8], dds_size: Dword) -> Result<Self, Box<dyn Error>> {
        const DDS_MAGIC: Dword = u32::from_le_bytes([b' ', b'S', b'D', b'D']);
        const DDS_FLAGS: Dword = DDSD_CAPS | DDSD_HEIGHT | DDSD_WIDTH | DDSD_PIXELFORMAT | DDSD_LINEARSIZE;
        let size = dds_size - 12;
        let pixel_format = PixelFormat {
            size: 32,
            flags: DDPF_FOURCC,
            four_cc: to_dword(&dds_data, 0)?,
            alpha_bit_mask: 0,
            r_bit_mask: 0,
            g_bit_mask: 0,
            b_bit_mask: 0,
            rgb_bit_count: 0,
        };
        let caps = Caps {
            caps1: DDSCAPS_TEXTURE,
            caps2: 0,
            ddsx: 0,
            reserved: 0,
        };
        Ok(Self {
            magic: DDS_MAGIC,
            size: 124,
            flags: DDS_FLAGS,
            height: to_dword(&dds_data, 2)?,
            width: to_dword(&dds_data, 1)?,
            pitch_or_linear_size: size,
            depth: 0,
            mip_map_count: 0,
            reserved: [0; 11],
            pixel_format,
            caps,
            reserved2: 0,
        })
    }

    fn write(buf: &mut [u8], dds_header: Self, filename: &Path, dds_size: Dword) {

    }
}

fn extract_file<R: BufRead + Seek>(
    reader: &mut R,
    basedir: &Path,
    filename: &str,
    entry: &PakFileEntry,
    header: &PakHeader
) -> Result<(), Box<dyn Error>> {
    let path = basedir.join(Path::new(filename));
    let dir = path.parent().ok_or("couldn't extract dirname")?;
    fs::create_dir_all(dir)?;
    let mut f = File::create(path)?;
    reader.seek(SeekFrom::Start((header.data_start + entry.data_pos).into()))?;
    let mut buf = Vec::with_capacity(entry.data_size.try_into()?);
    reader.read_exact(&mut buf)?;
    f.write_all(&buf)?;
    let path = basedir.join(Path::new(filename));
    if let Some("dxt") = path.extension().map(|os| os.to_str()).flatten() {
        let dds_header = DDSHeader::parse(&mut buf, entry.data_size)?;
        let filename: PathBuf = path.with_extension("dds");
        let dds_size = entry.data_size;
        DDSHeader::write(&mut buf, dds_header, filename.as_path(), dds_size);
    }
    Ok(())
}

fn main() -> Result<(), Box<dyn Error>> {
    let args: Vec<String> = env::args().collect();
    dbg!(&args);
    let filepath = &args[1];
    let f = File::open(filepath)?;
    let basedir = Path::new(filepath).parent().ok_or("couldn't get parent folder")?;
    let mut reader = BufReader::new(f);
    let header = PakHeader::read_parse(&mut reader)?;
    dbg!(&header);
    let num_entries = header.file_entries_size as usize / size_of::<PakHeader>();
    reader.seek(SeekFrom::Start(header.file_entries_start.into()))?;
    let mut entries = vec![];
    for _ in 0..num_entries {
        let entry = PakFileEntry::read_parse(&mut reader)?;
        entries.push(entry);
    }
    dbg!(&entries[0..3]);
    let mut filename_buf = vec![];
    for entry in entries {
        reader.seek(SeekFrom::Start((entry.filename_pos + header.file_names_start).into()))?;
        reader.read_until(0x0, &mut filename_buf)?;
        let filename: String = String::from_utf8(filename_buf)?;
        let filename: &str = filename.trim_end_matches('\0');
        dbg!(&filename);
        // extract_file(&mut reader, basedir, filename, &entry, &header)?;
        break
    }
    Ok(())
}

#[cfg(test)]
mod test {
    use std::{f32, io::Cursor};

    use super::*;

    #[test]
    fn pak_header_size_test() {
        assert_eq!(size_of::<PakHeader>(), 40);
    }

    #[test]
    fn pak_file_entry_size_test() {
        assert_eq!(size_of::<PakFileEntry>(), 20);
    }

    #[test]
    fn float_test() {
        let version = [0, 0, 0x80, 0x3f];
        assert_eq!(f32::from_le_bytes(version), 1.0);
    }

    #[test]
    fn header_test() {
        let header_raw = [
            0x4b, 0x41, 0x50, 0x4c,
            0x00, 0x00, 0x80, 0x3f,
            0x28, 0x00, 0x00, 0x00,
            0xc4, 0x28, 0x00, 0x00,
            0xd0, 0xf3, 0x00, 0x00,
            0x5c, 0xf1, 0x02, 0x00,
            0x9c, 0x28, 0x00, 0x00,
            0x0c, 0xcb, 0x00, 0x00,
            0x8c, 0xfd, 0x01, 0x00,
            0xcc, 0xe0, 0x10, 0x4a,
            0xd6, 0xb3, 0x27, 0x00,
            0x6f, 0x76, 0x28, 0x00,
            0xc6, 0xf6, 0x3c, 0x00,
            0xa6, 0xc2, 0xb7, 0x00,
        ];
        let mut cursor = Cursor::new(header_raw);
        assert_eq!(
            PakHeader::read_parse(&mut cursor).unwrap(),
            PakHeader {
                magic: 1280328011,
                version: 1.0,
                index_start: 40,
                file_entries_start: 10436,
                file_names_start: 62416,
                data_start: 192860,
                index_size: 10396,
                file_entries_size: 51980,
                file_names_size: 130444,
                data_size: 1242620108,
            }
        );
    }
}
