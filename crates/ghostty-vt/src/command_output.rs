use super::*;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CommandOutput {
    Text(String),
    Empty,
    NoCommand,
}

impl Terminal {
    /// Return the prompt, input, and output of the nth most recent completed command.
    ///
    /// This relies on OSC 133 semantic prompt markers, skips the newest
    /// (current) prompt, and treats `n = 0` as the latest completed command.
    pub fn command_output(&self, n: usize) -> Result<CommandOutput, Error> {
        let total_rows = self.total_rows()?;
        let cols = self.cols()?;
        let mut prompt_rows = Vec::new();
        for y in 0..total_rows {
            let Some(y) = u32::try_from(y).ok() else {
                break;
            };
            let grid_ref = self.grid_ref(ghostty_screen_point(0, y))?;
            if grid_ref_is_primary_prompt(&grid_ref)? {
                prompt_rows.push(y);
            }
        }

        let Some(prompt_idx) = prompt_rows.len().checked_sub(n.saturating_add(2)) else {
            return Ok(CommandOutput::NoCommand);
        };
        let start_row = prompt_rows[prompt_idx];
        let end_row_exclusive = prompt_rows[prompt_idx + 1];
        let mut last_text_cell = None;
        for y in start_row..end_row_exclusive {
            let mut grid_ref = self.grid_ref(ghostty_screen_point(0, y))?;
            for x in 0..cols {
                grid_ref.x = x;
                if grid_ref_has_text(&grid_ref)? {
                    last_text_cell = Some((x, y));
                }
            }
        }

        let Some(last_text_cell) = last_text_cell else {
            return Ok(CommandOutput::Empty);
        };
        let text = self.read_text_screen((0, start_row), last_text_cell, false)?;
        let text = text.trim().to_owned();
        if text.is_empty() {
            Ok(CommandOutput::Empty)
        } else {
            Ok(CommandOutput::Text(text))
        }
    }
}

fn grid_ref_has_text(grid_ref: &ffi::GhosttyGridRef) -> Result<bool, Error> {
    let mut raw = ffi::GhosttyCell::default();
    unsafe {
        ffi::ghostty_grid_ref_cell(grid_ref, &mut raw).into_result()?;
    }
    let mut has_text = false;
    unsafe {
        ffi::ghostty_cell_get(
            raw,
            ffi::GhosttyCellData_GHOSTTY_CELL_DATA_HAS_TEXT,
            (&mut has_text as *mut bool).cast(),
        )
        .into_result()?;
    }
    Ok(has_text)
}

fn grid_ref_is_primary_prompt(grid_ref: &ffi::GhosttyGridRef) -> Result<bool, Error> {
    let mut row = 0;
    unsafe {
        ffi::ghostty_grid_ref_row(grid_ref, &mut row).into_result()?;
    }
    let mut semantic_prompt = ffi::GhosttyRowSemanticPrompt_GHOSTTY_ROW_SEMANTIC_NONE;
    unsafe {
        ffi::ghostty_row_get(
            row,
            ffi::GhosttyRowData_GHOSTTY_ROW_DATA_SEMANTIC_PROMPT,
            (&mut semantic_prompt as *mut ffi::GhosttyRowSemanticPrompt).cast(),
        )
        .into_result()?;
    }
    Ok(semantic_prompt == ffi::GhosttyRowSemanticPrompt_GHOSTTY_ROW_SEMANTIC_PROMPT)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn command_output_uses_semantic_prompts_and_cycles_backwards() {
        let mut terminal = Terminal::new(20, 8, 4096).unwrap();
        terminal.write(
            b"\x1b]133;A\x07$ \x1b]133;B\x07first\r\n\x1b]133;C\x07one\r\n\x1b]133;D;0\x07\
              \x1b]133;A\x07$ \x1b]133;B\x07second\r\n\x1b]133;C\x07two\r\n\x1b]133;D;0\x07\
              \x1b]133;A\x07$ \x1b]133;B\x07",
        );
        assert_eq!(
            terminal.command_output(0).unwrap(),
            CommandOutput::Text("$ second\ntwo".to_string())
        );
        assert_eq!(
            terminal.command_output(1).unwrap(),
            CommandOutput::Text("$ first\none".to_string())
        );
        assert_eq!(
            terminal.command_output(2).unwrap(),
            CommandOutput::NoCommand
        );
    }
}
