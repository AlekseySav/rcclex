/// configuration for regex compilation
pub struct Config {
    /// charset for '.'
    pub dot_charset: Charset,
    /// charset for \a-\z \A-\Z (except A, Z, x, X)
    pub esc_charset: HashMap<u8, Charset>,
    /// prepend '(' with '\A', append ')' with '\Z'
    pub auto_groups: bool,
}

pub type Result<T> = std::result::Result<T, Error>;

/// errors, that can happen during regex compilation
#[derive(Debug, Clone, PartialEq)]
pub enum Error {
    Config,
    Charset,
    Escape,
    Repeat,
    Overflow,
    Balance,
    Group,
    Union,
    Empty,
    Postfix,
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            Self::Config => write!(f, "invalid regex configuration"),
            Self::Charset => write!(f, "bad charset syntax"),
            Self::Escape => write!(f, "invalid escape sequence"),
            Self::Repeat => write!(f, "bad repeat syntax"),
            Self::Overflow => write!(f, "repeat number or hexadecimal char value exceeds 255"),
            Self::Balance => write!(f, "bad () balance"),
            Self::Group => write!(f, "attempted to define empty expr as a group"),
            Self::Union => write!(f, "invalid usage of '|' or bad () balance"),
            Self::Empty => write!(f, "empty expression or sub-expression"),
            Self::Postfix => write!(f, "invalid usage of postfix operator"),
        }
    }
}

impl Config {
    /// create default configuration
    /// auto-goups are disabled
    /// '.' matches on any character
    /// default perl-regex charsets are defined (\t \n \s \S \d \D \w \W)
    pub fn default() -> Config {
        let w = charset!([b'A', b'Z'], [b'a', b'z'], [b'0', b'9']; b'_');
        Config {
            dot_charset: Charset::ALL,
            esc_charset: HashMap::from([
                (b't', charset!(b'\t')),
                (b'n', charset!(b'\n')),
                (b's', charset!(b' ', b'\t', b'\r', b'\n')),
                (b'S', charset!(b' ', b'\t', b'\r', b'\n').inv()),
                (b'd', charset!([b'0', b'9'])),
                (b'D', charset!([b'0', b'9']).inv()),
                (b'w', w.clone()),
                (b'W', w.inv()),
            ]),
            auto_groups: false,
        }
    }

    /// check if config is valid
    pub fn is_valid(&self) -> bool {
        for (c, _) in self.esc_charset.iter() {
            match c {
                b'B'..=b'W' | b'Y' | b'a'..=b'w' | b'y' | b'z' => (),
                _ => return false,
            }
        }
        return true;
    }
}
