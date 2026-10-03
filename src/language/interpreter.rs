use crate::language::config::Config;
use crate::language::glyphs::decode;
use scorched::{LogData, LogImportance, logf};
use std::{
    collections::hash_map::RandomState,
    hash::BuildHasher,
    io::{self, BufRead, Write},
    thread,
    time::Duration,
};

#[derive(Clone, Copy, PartialEq, Eq, Default)]
pub struct Grid(u16); // only low 9 bits used; cell `id` lives at bit (8 - id)

impl Grid {
    const ALL: u16 = 0b1_1111_1111;

    #[inline]
    fn bit(id: u8) -> u16 {
        1 << (8 - id) // cell 0 is the MSB — matches glyph encoding, see note below
    }

    pub fn get(self, id: u8) -> bool {
        self.0 & Self::bit(id) != 0
    }
    pub fn flip(&mut self, id: u8) {
        self.0 ^= Self::bit(id);
    }
    pub fn set(&mut self, id: u8, on: bool) {
        if on {
            self.0 |= Self::bit(id)
        } else {
            self.0 &= !Self::bit(id)
        }
    }
    pub fn set_all(&mut self, on: bool) {
        self.0 = if on { Self::ALL } else { 0 };
    }
    pub fn xor(&mut self, other: Grid) {
        self.0 ^= other.0;
    }
    pub fn mask(&mut self, other: Grid) {
        self.0 |= other.0;
    } // adjust to your mask semantics
    pub fn index(self) -> usize {
        self.0 as usize
    } // == the GLYPHS index, for printing
}

impl std::fmt::Display for Grid {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for row in 0..3 {
            for col in 0..3 {
                write!(f, "{}", self.get(row * 3 + col) as u8)?;
            }
            writeln!(f)?;
        }
        Ok(())
    }
}

pub fn interpret(preprocessed_code: &str, cfg: Config) {
    if !cfg.no_log && cfg.verbosity >= 2 {
        logf!(Info, "Interpreting script");
    }
    if let Err(error) = run(
        preprocessed_code,
        &cfg,
        &mut io::stdin().lock(),
        &mut io::stdout(),
    ) && !cfg.no_log
    {
        logf!(Error, "Interpreter error: {}", error);
    }
}

// Match control-flow blocks by character position, including skipped blocks.
fn block_ends(code: &[u8]) -> Result<Vec<usize>, String> {
    let mut ends = vec![usize::MAX; code.len()];
    let mut blocks = Vec::new();
    for (i, &character) in code.iter().enumerate() {
        match character {
            b'i' | b'w' => blocks.push(i),
            b'}' | b']' => {
                let open = blocks
                    .pop()
                    .ok_or_else(|| format!("Unmatched block end at {i}"))?;
                let expected = if code[open] == b'i' { b'}' } else { b']' };
                if character != expected {
                    return Err(format!("Mismatched block end at {i}"));
                }
                ends[open] = i;
                ends[i] = open;
            }
            _ => {}
        }
    }
    if !blocks.is_empty() {
        return Err("Unclosed control-flow block".to_owned());
    }
    Ok(ends)
}

fn condition(code: &[u8], start: usize, grid: Grid) -> bool {
    let mut i = start + 1;
    let mut group = true;
    let mut result = false;
    loop {
        let equal = grid.get(code[i] - b'0') == (code[i + 2] == b'1');
        group &= if code[i + 1] == b'=' { equal } else { !equal };
        i += 3;
        match code.get(i) {
            Some(b'&') => i += 1,
            Some(b'|') => {
                result |= group;
                group = true;
                i += 1;
            }
            _ => return result || group,
        }
    }
}

fn run(
    source: &str,
    cfg: &Config,
    input: &mut impl BufRead,
    output: &mut impl Write,
) -> Result<(), String> {
    let code = source.as_bytes();
    let ends = block_ends(code)?;
    let mut grid = Grid::default();
    let mut saves = [Grid::default(); 9];
    let mut queue = String::new();
    // RandomState supplies a randomly seeded hasher without another dependency.
    let random = RandomState::new();
    let mut draw = 0_u64;
    let mut value = |arg: u8| {
        if arg == b'r' {
            draw = draw.wrapping_add(1);
            random.hash_one(draw) & 1 != 0
        } else {
            arg == b'1'
        }
    };
    let mut pc = 0;
    while pc < code.len() {
        let i = pc;
        let width = match code[i] {
            b's' => 3,
            b'f' | b'a' | b'q' => 2,
            b'g' => {
                if code.get(i + 1) == Some(&b'g') {
                    2
                } else {
                    3
                }
            }
            b'i' | b'w' => 4,
            _ => 1,
        };
        if i + width > code.len() {
            return Err(format!("Incomplete instruction at {i}"));
        }
        let mut modified = false;
        match code[i] {
            b's' => {
                grid.set(code[i + 1] - b'0', value(code[i + 2]));
                modified = true;
            }
            b'f' => {
                grid.flip(code[i + 1] - b'0');
                modified = true;
            }
            b'a' => {
                if code[i + 1] == b'r' {
                    for id in 0..9 {
                        grid.set(id, value(b'r'));
                    }
                } else {
                    grid.set_all(value(code[i + 1]));
                }
                modified = true;
            }
            b'q' if code[i + 1] == b's' => {
                queue.push_str(decode(grid.index()).ok_or("Grid has no glyph")?);
            }
            b'q' => queue.clear(),
            b'p' => {
                if queue.is_empty() {
                    writeln!(
                        output,
                        "{}",
                        decode(grid.index()).ok_or("Grid has no glyph")?
                    )
                    .map_err(|e| e.to_string())?;
                } else {
                    writeln!(output, "{queue}").map_err(|e| e.to_string())?;
                    queue.clear();
                }
                output.flush().map_err(|e| e.to_string())?;
            }
            b'g' => match code[i + 1] {
                b'g' => {
                    output.flush().map_err(|e| e.to_string())?;
                    let mut line = String::new();
                    input.read_line(&mut line).map_err(|e| e.to_string())?;
                    let pattern = line.trim_end_matches(['\r', '\n']);
                    if pattern.len() != 9 || !pattern.bytes().all(|c| matches!(c, b'0' | b'1')) {
                        return Err("Grid input must be exactly nine binary digits".to_owned());
                    }
                    grid = Grid(u16::from_str_radix(pattern, 2).map_err(|e| e.to_string())?);
                    modified = true;
                }
                b's' => saves[(code[i + 2] - b'0') as usize] = grid,
                b'l' => {
                    grid = saves[(code[i + 2] - b'0') as usize];
                    modified = true;
                }
                b'm' => {
                    grid.mask(saves[(code[i + 2] - b'0') as usize]);
                    modified = true;
                }
                b'x' => {
                    let other = if code[i + 2] == b'c' {
                        grid
                    } else {
                        saves[(code[i + 2] - b'0') as usize]
                    };
                    grid.xor(other);
                    modified = true;
                }
                _ => return Err(format!("Invalid grid command at {i}")),
            },
            b'i' | b'w' => {
                if !condition(code, i, grid) {
                    pc = ends[pc] + 1;
                    continue;
                }
            }
            b']' => {
                pc = ends[pc];
                continue;
            }
            b'e' => {
                // Find the innermost enclosing loop, even inside an if block.
                let open = (0..pc)
                    .rev()
                    .find(|&j| code[j] == b'w' && ends[j] > pc)
                    .ok_or("Exit command is outside a while loop")?;
                pc = ends[open] + 1;
                continue;
            }
            b'b' | b'd' => {
                let mut end = i + 1;
                while code.get(end).is_some_and(u8::is_ascii_digit) {
                    end += 1;
                }
                let amount: u64 = source[i + 1..end]
                    .parse()
                    .map_err(|_| format!("Numeric argument too large at {i}"))?;
                if code[i] == b'b' {
                    let amount = usize::try_from(amount).map_err(|_| "Back argument too large")?;
                    pc = pc
                        .checked_sub(amount)
                        .ok_or("Back command jumps before script start")?;
                    // Nim subtracts from the command's character index; the
                    // common increment below then advances to the next character.
                } else {
                    thread::sleep(Duration::from_secs(amount));
                    pc = end - 1;
                }
            }
            b't' => break,
            _ => {}
        }
        if modified {
            echo_grid_mod(grid, cfg, output).map_err(|e| e.to_string())?;
        }
        if !matches!(code[i], b'b' | b'd') {
            pc += width - 1;
            if matches!(code[i], b'i' | b'w') {
                while matches!(code.get(pc + 1), Some(b'&' | b'|')) {
                    pc += 4;
                }
            }
        }
        pc += 1;
    }
    Ok(())
}

fn echo_grid_mod(grid: Grid, cfg: &Config, output: &mut impl Write) -> io::Result<()> {
    if cfg.echo_grid_mod {
        write!(output, "{grid}")?;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn output(code: &str) -> String {
        let mut output = Vec::new();
        run(code, &Config::default(), &mut io::empty(), &mut output).unwrap();
        String::from_utf8(output).unwrap()
    }

    #[test]
    fn cell_commands_and_termination() {
        assert_eq!(output("s71pf7ps61ptf8p"), "a\n \nc\n");
        assert_eq!(output("a1a0p"), " \n");
    }

    #[test]
    fn queue_print_consumes_and_clear_discards() {
        assert_eq!(output("f7qsf8qsppqsa0qcp"), "ab\nb\n \n");
    }

    #[test]
    fn saved_grids_include_cell_zero_and_mask_uses_or() {
        assert_eq!(output("s01gs8a0gl8gxcps71gs0a0s81gm0pgx0p"), " \nb\n\n\n");
        assert_eq!(output("gl8p"), " \n");
    }

    #[test]
    fn nested_false_blocks_are_skipped() {
        assert_eq!(output("i0=1i1=0f7p}w2=0f6p]}f8p"), "\n\n");
        assert_eq!(output("w0=1w1=0f7p]f6p]f7p"), "a\n");
    }

    #[test]
    fn conditions_compare_literals_and_and_precedes_or() {
        assert_eq!(
            output("i0!1f7p}i0=1|1=0&2=0f8p}i0=0|1=1&2=1p}"),
            "a\nb\nb\n"
        );
        assert_eq!(output("w0=0&1=0f7ps01]"), "a\n");
    }

    #[test]
    fn nested_loops_recheck_and_exit_through_if() {
        assert_eq!(output("w0=0w1=0f7ps11]s01]"), "a\n");
        assert_eq!(output("w0=0w1=0i2=0f7pe}]f8pe]p"), "a\nb\nb\n");
    }

    #[test]
    fn back_counts_characters_with_nim_increment() {
        assert_eq!(output("0f7pi7=1b8}p"), "a\n \n \n");
        assert_eq!(output("b0d0f7p"), "a\n");
        assert_eq!(output("f7pi7=1s70b10}p"), "a\n \n \n");
        // Multi-digit counts measure characters too.
        assert_eq!(output("0f7pd0i7=1b10}p"), "a\n \n \n");
    }

    #[test]
    fn give_accepts_binary_input_and_rejects_invalid_patterns() {
        let mut result = Vec::new();
        run(
            "ggp",
            &Config::default(),
            &mut &b"000000010\r\n"[..],
            &mut result,
        )
        .unwrap();
        assert_eq!(result, b"a\n");
        for input in ["00000002x\n", "00000010\n", ""] {
            assert!(
                run(
                    "gg",
                    &Config::default(),
                    &mut input.as_bytes(),
                    &mut Vec::new()
                )
                .is_err()
            );
        }
    }

    #[test]
    fn echo_shows_mutations_and_random_values_stay_binary() {
        let cfg = Config {
            echo_grid_mod: true,
            ..Config::default()
        };
        let mut result = Vec::new();
        run("s0rargs0gl0gm0gxc", &cfg, &mut io::empty(), &mut result).unwrap();
        let result = String::from_utf8(result).unwrap();
        assert_eq!(result.lines().count(), 15);
        assert!(result.chars().all(|c| matches!(c, '0' | '1' | '\n')));
        assert!(result.ends_with("000\n000\n000\n"));
    }

    #[test]
    fn invalid_runtime_operations_return_errors() {
        for code in [
            "b1",
            "e",
            "i0=0]",
            "w0=0",
            "f",
            "d18446744073709551616",
            "a1p",
        ] {
            assert!(
                run(code, &Config::default(), &mut io::empty(), &mut Vec::new()).is_err(),
                "{code}"
            );
        }
    }
}
