use crate::card::Card;
use crate::shoe::CardShoe;
use std::path::Path;
use std::{fs, io, fmt};

pub struct FileShoe {
    cards: Vec<Card>,
    curr_pos: usize,
    stride: usize,
    ended: bool,
}

impl FileShoe {
    pub fn new(file_path: &Path) -> Result<FileShoe, io::Error> {
        let cards = fs::read(file_path)?.iter().map(|&n| {
            assert!(n > 0 && n < 11);
            Card(n)
        }).collect();

        Ok(FileShoe {
            cards,
            curr_pos: 0,
            stride: 1,
            ended: false,
        })
    }
}

impl Iterator for FileShoe {
    type Item = Card;

    fn next(&mut self) -> Option<Self::Item> {
        let pos = self.curr_pos;

        if self.ended {
            return None;
        }

        self.curr_pos += self.stride;
        if self.curr_pos >= self.cards.len() {
            self.stride += 1;
            if self.stride >= self.cards.len() / 10 {
                self.ended = true;
            }
            self.curr_pos -= self.cards.len();
        }

        Some(self.cards[pos])
    }
}

impl CardShoe for FileShoe {
    fn try_pick(&mut self) -> Option<Card> {
        Some(
            self.next()
                .expect("Reached maximum number of cards for this FileShoe")
        )
    }

    fn try_pick_first(&mut self, card: Card) -> Option<Card> {
        Some(
            self.find(|&c| c == card)
                .expect("Reached maximum number of cards for this FileShoe")
        )
    }

    fn reshuffle(&mut self) {
        unimplemented!("FileShoe cannot be reshuffled")
    }

    fn force_true_count(&mut self, _true_count: f32) {
        unimplemented!("Cannot force true count to a FileShoe")
    }

    fn adjust_running_count(&mut self, _rel_rc: i32) {
        unimplemented!("Cannot adjust running count on a FileShoe")
    }
}

impl fmt::Display for FileShoe {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "[")?;

        for (i, card) in self.cards.iter().enumerate() {
            if i > 0 {
                write!(f, ", ")?;
            }
            write!(f, "{}", card)?;
        }

        write!(f, "]")
    }
}

#[cfg(test)]
mod tests {
    use std::path::Path;
    use std::fs::File;
    use std::io::Write;
    use crate::card::Card;
    use crate::shoe::CardShoe;
    use crate::shoe::file_shoe::FileShoe;

    #[test]
    fn it_loads_a_file_shoe() {
        let path = Path::new("/tmp/test_cards");
        let mut file = File::create(path).expect("Couldn't create test file");
        file.write_all(&[1, 6, 8, 10, 4]).expect("Couldn't write test file");
        std::mem::drop(file);

        let mut shoe = FileShoe::new(path).expect("Couldn't create FileShoe");

        assert_eq!(shoe.pick(), Card(1));
        assert_eq!(shoe.pick(), Card(6));
        assert_eq!(shoe.pick(), Card(8));
        assert_eq!(shoe.pick(), Card(10));
        assert_eq!(shoe.pick(), Card(4));
    }
}
