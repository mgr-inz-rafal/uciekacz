use colored::Colorize;

use crate::utils::Pos;

#[derive(Clone, PartialEq)]
pub(super) struct Board {
    // TODO: Could use some encapsulation
    pub(super) tiles: Vec<char>,
    pub(super) width: usize,
    pub(super) player_pos: Pos,
    pub(super) hunter_pos: Pos,
    pub(super) exit_pos: Pos,
}

impl Board {
    pub(super) fn new_test_01() -> Self {
        #[rustfmt::skip]
        let tiles = vec![
                '#', '#', '#', '#', '#', '#', '#', '#', '#', '#', '#', '#',
                '#', ' ', ' ', '@', ' ', ' ', ' ', ' ', ' ', ' ', '=', '#',
                '#', ' ', ' ', ' ', '#', '#', ' ', ' ', ' ', ' ', ' ', '#',
                '#', ' ', ' ', ' ', '#', ' ', ' ', ' ', ' ', ' ', ' ', '#',
                '#', ' ', ' ', ' ', '#', '#', ' ', ' ', ' ', ' ', ' ', '#',
                '#', ' ', ' ', ' ', ' ', ' ', ' ', ' ', ' ', ' ', ' ', '#',
                '#', ' ', ' ', ' ', ' ', ' ', ' ', ' ', '$', ' ', ' ', '#',
                '#', '#', '#', '#', '#', '#', '#', '#', '#', '#', '#', '#',
            ];
        let width = 12;
        let player_pos = Self::find_char(&tiles, '@', width);
        let hunter_pos = Self::find_char(&tiles, '=', width);
        let exit_pos = Self::find_char(&tiles, '$', width);

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
            hunter_pos,
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

    fn find_char(tiles: &[char], cc: char, width: usize) -> Pos {
        tiles
            .iter()
            .enumerate()
            .find(|(_, c)| **c == cc)
            .map(|(i, _)| {
                let y = i / width;
                let x = i - y * width;
                Pos::new(x as i32, y as i32)
            })
            .expect("no {c}")
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
