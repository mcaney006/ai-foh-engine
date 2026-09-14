use crate::digico::{CommandSet, DigicoCommand};

#[derive(Debug, Clone)]
pub struct JournalLine {
    pub addr: String,
    pub note: String,
    pub bytes: usize,
}

pub fn preview(set: CommandSet, cmds: &[DigicoCommand]) -> Result<Vec<JournalLine>, crate::digico::ControlError> {
    let mut out = Vec::new();
    for cmd in cmds {
        let (addr, _) = cmd.address(set)?;
        let bytes = cmd.encode(set)?.len();
        out.push(JournalLine { addr, note: format!("{cmd:?}"), bytes });
    }
    Ok(out)
}

pub fn render(lines: &[JournalLine]) -> String {
    let mut s = String::from("# OSC dry-run — not sent\n");
    for l in lines {
        s.push_str(&format!("{}  ({} bytes)  {}\n", l.addr, l.bytes, l.note));
    }
    s
}
