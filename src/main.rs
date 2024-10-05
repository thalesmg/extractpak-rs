// https://github.com/timfel/monkey/blob/master/extractpak.c

use std::{env, error::Error, fs::File, io::{BufRead, BufReader}};

type dword = u32;

const MAGIC: &'static str = "KAPL";

#[derive(Debug, PartialEq)]
struct PakHeader<'a> {
    magic: &'a str,
    version: f32,
    index_start: dword,
    file_entries_start: dword,
    file_names_start: dword,
    data_start: dword,
    index_size: dword,
    file_entries_size: dword,
    file_names_size: dword,
    data_size: dword,
}

impl<'a> PakHeader<'a> {
    fn parse<R: BufRead>(reader: &mut R) -> Result<PakHeader<'a>, Box<dyn Error>> {
        const SZ: usize = size_of::<PakHeader>();
        dbg!(SZ);
        let mut buf = [0; SZ];
        reader.read_exact(&mut buf)?;
        let magic = std::str::from_utf8(&buf[0..4])?;
        debug_assert_eq!(magic, "KAPL");

        Ok(PakHeader {
            magic: MAGIC,
            version: f32::from_le_bytes(buf[4..8].try_into()?),
            index_start: u32::from_le_bytes(buf[8..12].try_into()?),
            file_entries_start: u32::from_le_bytes(buf[12..16].try_into()?),
            file_names_start: u32::from_le_bytes(buf[16..20].try_into()?),
            data_start: u32::from_le_bytes(buf[20..24].try_into()?),
            index_size: u32::from_le_bytes(buf[24..28].try_into()?),
            file_entries_size: u32::from_le_bytes(buf[28..32].try_into()?),
            file_names_size: u32::from_le_bytes(buf[32..36].try_into()?),
            data_size: u32::from_le_bytes(buf[36..40].try_into()?),
        })
    }
}

fn main() -> Result<(), Box<dyn Error>> {
    let args: Vec<String> = env::args().collect();
    dbg!(&args);
    let filepath = &args[1];
    let f = File::open(filepath)?;
    let mut reader = BufReader::new(f);
    let header = PakHeader::parse(&mut reader)?;
    dbg!(&header);
    Ok(())
}

#[cfg(test)]
mod test {
    use std::{f32, io::Cursor};

    use super::*;

    #[test]
    fn float_test() {
        let version = [0, 0, 0x80, 0x3f];
        assert_eq!(f32::from_le_bytes(version), 1.0);
    }

    #[test]
    fn header_test() {
        let mut header_raw = [
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
            PakHeader::parse(&mut cursor).unwrap(),
            PakHeader {
                magic: MAGIC,
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
