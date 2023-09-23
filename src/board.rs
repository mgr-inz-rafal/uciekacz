use colored::Colorize;

use crate::utils::Pos;

#[derive(Clone, PartialEq)]
pub(super) struct Board {
    // TODO: Could use some encapsulation
    pub(super) tiles: Vec<char>,
    pub(super) width: usize,
    pub(super) player_pos: Pos,
    pub(super) hunters_pos: Vec<Pos>,
    pub(super) exit_pos: Pos,
}

impl std::hash::Hash for Board {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.tiles.hash(state);
    }
}

impl Board {
    pub(super) fn new_test_01() -> Self {
        #[rustfmt::skip]
        let tiles = vec![
                '#', '#', '#', '#', '#', '#', '#', '#', '#', '#', '#', '#',
                '#', ' ', ' ', '@', '#', ' ', ' ', ' ', ' ', ' ', '=', '#',
                '#', ' ', ' ', ' ', '#', '#', ' ', ' ', ' ', ' ', ' ', '#',
                '#', ' ', ' ', ' ', '#', ' ', ' ', ' ', ' ', ' ', ' ', '#',
                '#', ' ', ' ', ' ', '#', '#', ' ', ' ', ' ', ' ', ' ', '#',
                '#', ' ', ' ', ' ', ' ', ' ', ' ', ' ', ' ', '=', ' ', '#',
                '#', ' ', ' ', ' ', ' ', ' ', ' ', ' ', '$', ' ', ' ', '#',
                '#', '#', '#', '#', '#', '#', '#', '#', '#', '#', '#', '#',
            ];
        let width = 12;
        let player_pos = *Self::find_chars(&tiles, '@', width)
            .first()
            .expect("should have player");
        let hunters_pos = Self::find_chars(&tiles, '=', width);
        let exit_pos = *Self::find_chars(&tiles, '$', width)
            .first()
            .expect("should have exit");

        // #[rustfmt::skip]
        // let tiles = vec![
        //         '#', '#', '#', '#',
        //         '#', ' ', '@', '#',
        //         '#', ' ', ' ', '#',
        //         '#', ' ', '$', '#',
        //         '#', ' ', ' ', '#',
        //         '#', ' ', ' ', '#',
        //         '#', '=', ' ', '#',
        //         '#', '#', '#', '#'
        //     ];
        // let width = 4;
        // let player_pos = find_char(&tiles, '@', width);
        // let hunter_pos = find_char(&tiles, '=', width);
        // let exit_pos = find_char(&tiles, '$', width);

        Self {
            tiles,
            width,
            player_pos,
            hunters_pos,
            exit_pos,
        }
    }

    pub(super) fn at(&self, pos: Pos) -> Option<&char> {
        let i = self.pos2index(pos);
        self.tiles.get(i)
    }

    pub(super) fn set_at(&mut self, pos: Pos, c: char) {
        let i = self.pos2index(pos);
        self.tiles[i] = c;
    }

    fn pos2index(&self, pos: Pos) -> usize {
        (pos.y * self.width as i32 + pos.x) as usize
    }

    fn find_chars(tiles: &[char], cc: char, width: usize) -> Vec<Pos> {
        tiles
            .iter()
            .enumerate()
            .filter(|(_, c)| **c == cc)
            .map(|(i, _)| {
                let y = i / width;
                let x = i - y * width;
                Pos::new(x as i32, y as i32)
            })
            .collect()
    }
}

impl std::fmt::Display for Board {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let chunks = self.tiles.chunks(self.width);
        for chunk in chunks {
            for c in chunk {
                write!(
                    f,
                    "{}",
                    match c {
                        '#' => "#".blue(),
                        '@' => "@".bright_green(),
                        '=' => "K".red(),
                        '$' => "$".bright_white(),
                        _ => " ".black(),
                    }
                )?;
            }
            writeln!(f)?;
        }
        Ok(())
    }
}
