use alloc::{string::String, vec::Vec};

pub enum AnsiEscape {
    Foreground(u8),
    Background(u8),
    Reset,
}

pub struct AnsiAdapter;

impl AnsiAdapter {
    pub fn parse(chars: &[u8]) -> (Vec<Option<AnsiEscape>>, usize) {
        let mut i = 0;
        let mut skip = 0;
        let mut vec = Vec::new();
        let mut light = false;

        'outer: loop {
            let mut end = false;
            let mut tmp = String::new();

            'inner: loop {
                match chars[i] {
                    b'm' => {
                        end = true;
                        break 'inner;
                    }
                    b';' => {
                        skip += 1;
                        i += 1;
                        break 'inner;
                    }
                    _ => {
                        tmp.push(char::from(chars[i]));
                        skip += 1;
                        i += 1;
                    }
                }
            }
            let num = tmp.parse::<u8>().unwrap();
            vec.push(match num {
                0 => Some(AnsiEscape::Reset),
                1 => {
                    light = true;
                    None
                }
                2..=8 => None,
                30..=37 => {
                    let color = if !light { num - 30 } else { num - 22 };
                    Some(AnsiEscape::Foreground(color))
                }
                40..=47 => {
                    let color = if !light { num - 40 } else { num - 32 };
                    Some(AnsiEscape::Background(color))
                }
                _ => None,
            });
            if end {
                break 'outer;
            }
        }

        (vec, skip)
    }
}
