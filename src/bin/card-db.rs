use std::{
    fs::File,
    io::{self, BufRead, BufReader, BufWriter},
    str::FromStr,
};

use triple_triad::data::Card;

fn main() -> io::Result<()> {
    // TODO read from args
    let f_in = File::open("config/cards")?;
    let reader = BufReader::new(f_in);

    let f_out = File::create("config/cards.db")?;
    let mut writer = BufWriter::new(f_out);

    for (j, line) in reader.lines().enumerate() {
        let line = line?;

        match Card::from_str(&line) {
            Ok(card) => card.write_bytes(&mut writer)?,
            Err(e) => eprintln!("ERR: processing card at line {}: {:?}", j + 1, e),
        }
    }

    Ok(())
}

// TODO add magic number and header w/card count to file
