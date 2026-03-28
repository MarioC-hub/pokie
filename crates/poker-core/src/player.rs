use core::fmt;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum Player {
    Oop,
    Ip,
}

impl Player {
    pub const ALL: [Player; 2] = [Player::Oop, Player::Ip];

    pub const fn opponent(self) -> Self {
        match self {
            Player::Oop => Player::Ip,
            Player::Ip => Player::Oop,
        }
    }
}

impl fmt::Display for Player {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            Player::Oop => "oop",
            Player::Ip => "ip",
        })
    }
}
