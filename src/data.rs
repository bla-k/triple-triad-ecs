use std::{
    fs::File,
    io::{self, BufReader, Read, Write},
    str::FromStr,
};

#[derive(Debug)]
pub struct CardDb {
    pub elements: Vec<Element>,
    pub names: Vec<String>,
    pub stats: Vec<Stats>,
}

impl CardDb {
    const CARD_COUNT: usize = 110;

    fn from_reader<R: Read>(mut reader: R) -> io::Result<Self> {
        let mut elements = Vec::with_capacity(Self::CARD_COUNT);
        let mut names = Vec::with_capacity(Self::CARD_COUNT);
        let mut stats = Vec::with_capacity(Self::CARD_COUNT);

        let mut fixed_buf = [0u8; 4];

        loop {
            match reader.read_exact(&mut fixed_buf) {
                Ok(()) => {}
                Err(e) if e.kind() == io::ErrorKind::UnexpectedEof => break,
                Err(e) => return Err(e),
            }

            let [top_rgt, btm_lft, elem, str_len] = fixed_buf;

            let mut name_buf: Vec<u8> = vec![0u8; str_len as usize];
            reader.read_exact(&mut name_buf)?;

            elements.push(elem.into());
            names.push(
                String::from_utf8(name_buf)
                    .map_err(|e| io::Error::new(io::ErrorKind::InvalidInput, e))?,
            );
            stats.push([top_rgt, btm_lft].into());
        }

        Ok(CardDb {
            elements,
            names,
            stats,
        })
    }

    pub fn load(path: &str) -> io::Result<Self> {
        let f = File::open(path)?;
        Self::from_reader(BufReader::new(f))
    }
}

#[derive(Debug)]
pub struct Card {
    level: u8,
    name: String,
    stats: Stats,
    element: Element,
}

impl Card {
    const FIELD_SEPARATOR: char = ',';
}

impl Card {
    pub fn write_bytes<W: Write>(&self, writer: &mut W) -> io::Result<()> {
        let len = self.name.len() as u8;
        let name = self.name.as_bytes();
        let top_rgt = (self.stats.top << 4) | self.stats.rgt;
        let btm_lft = (self.stats.btm << 4) | self.stats.lft;
        let element = self.element as u8;

        writer.write_all(&[top_rgt, btm_lft, element, len])?;
        writer.write_all(name)?;

        Ok(())
    }
}

impl FromStr for Card {
    type Err = DataError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut parts = s.split(Self::FIELD_SEPARATOR);

        macro_rules! next_u8 {
            ($err:expr) => {
                parts
                    .next()
                    .ok_or($err)
                    .and_then(|s| s.parse::<u8>().map_err(|_| $err))
            };
        }

        let level = next_u8!(DataError::InvalidLevel)?;

        let name = parts.next().ok_or(DataError::InvalidName)?.to_string();

        let top = next_u8!(DataError::InvalidTopStat)?;
        let rgt = next_u8!(DataError::InvalidRightStat)?;
        let btm = next_u8!(DataError::InvalidBottomStat)?;
        let lft = next_u8!(DataError::InvalidLeftStat)?;

        let element = Element::from_str(parts.next().ok_or(DataError::InvalidElement)?)?;

        Ok(Card {
            level,
            name,
            stats: Stats { top, rgt, btm, lft },
            element,
        })
    }
}

#[derive(Debug)]
pub struct Stats {
    pub top: u8,
    pub rgt: u8,
    pub btm: u8,
    pub lft: u8,
}

impl From<[u8; 2]> for Stats {
    fn from(value: [u8; 2]) -> Self {
        let [top_rgt, btm_lft] = value;

        let top = top_rgt >> 4;
        let rgt = top_rgt & 0xF;
        let btm = btm_lft >> 4;
        let lft = btm_lft & 0xF;

        Stats { top, rgt, btm, lft }
    }
}

#[derive(Clone, Copy, Debug)]
#[repr(u8)]
pub enum Element {
    None = 0,
    Earth = 1,
    Fire = 2,
    Holy = 3,
    Ice = 4,
    Poison = 5,
    Thunder = 6,
    Water = 7,
    Wind = 8,
}

impl From<u8> for Element {
    fn from(value: u8) -> Self {
        match value {
            1 => Element::Earth,
            2 => Element::Fire,
            3 => Element::Holy,
            4 => Element::Ice,
            5 => Element::Poison,
            6 => Element::Thunder,
            7 => Element::Water,
            8 => Element::Wind,
            _ => Element::None,
        }
    }
}

impl FromStr for Element {
    type Err = DataError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let element = match s {
            "None" => Self::None,
            "Earth" => Self::Earth,
            "Fire" => Self::Fire,
            "Holy" => Self::Holy,
            "Ice" => Self::Ice,
            "Poison" => Self::Poison,
            "Thunder" => Self::Thunder,
            "Water" => Self::Water,
            "Wind" => Self::Wind,
            _ => return Err(DataError::InvalidElement),
        };

        Ok(element)
    }
}

#[derive(Clone, Copy, Debug)]
pub enum DataError {
    InvalidBottomStat,
    InvalidElement,
    InvalidLeftStat,
    InvalidLevel,
    InvalidName,
    InvalidRightStat,
    InvalidTopStat,
}
