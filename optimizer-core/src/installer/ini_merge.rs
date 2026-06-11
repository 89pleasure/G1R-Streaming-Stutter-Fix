#[derive(Debug, Clone)]
struct IniDocument {
    lines: Vec<IniLine>,
    had_final_newline: bool,
}

#[derive(Debug, Clone)]
struct IniLine {
    raw: String,
    kind: IniLineKind,
}

#[derive(Debug, Clone)]
enum IniLineKind {
    Section {
        name: String,
    },
    KeyValue {
        section: Option<String>,
        key: String,
    },
    Other,
}

#[derive(Debug)]
struct PatchDocument {
    sections: Vec<PatchSection>,
}

#[derive(Debug)]
struct PatchSection {
    name: String,
    keys: Vec<PatchKey>,
}

#[derive(Debug)]
struct PatchKey {
    key: String,
    value: String,
    raw: String,
}

#[derive(Debug)]
struct ManagedBlock {
    comments: Vec<String>,
    sections: Vec<String>,
    keys: Vec<ManagedKey>,
}

#[derive(Debug)]
struct ManagedKey {
    section: String,
    key: String,
    value: String,
}

pub(super) fn merge_ini_content(existing: Option<&str>, planned: &str, managed: &str) -> String {
    let Some(existing) = existing else {
        return planned.to_string();
    };

    if existing.trim().is_empty() {
        return planned.to_string();
    }

    let line_ending = detect_line_ending(existing);
    let mut document = IniDocument::parse(existing);
    let patch = PatchDocument::parse(planned);
    let planned_blocks = ManagedBlock::parse_all(planned);
    let managed_patch = PatchDocument::parse(managed);
    let managed_blocks = ManagedBlock::parse_all(managed);
    document.remove_inactive_managed_settings(&patch, &managed_patch);
    document.remove_inactive_managed_scaffolding(&patch, &managed_blocks);
    document.apply_patch(&patch);
    document.ensure_block_comments(&planned_blocks);
    document.normalize_blank_lines();
    document.render(line_ending)
}

pub(super) fn has_external_ini_settings(existing: &str, planned: &str) -> bool {
    let document = IniDocument::parse(existing);
    let patch = PatchDocument::parse(planned);
    document.has_external_settings(&patch)
}

pub(super) fn has_managed_ini_settings(existing: &str, managed: &str) -> bool {
    let document = IniDocument::parse(existing);
    let patch = PatchDocument::parse(managed);
    document.has_managed_settings(&patch)
}

impl IniDocument {
    fn parse(content: &str) -> Self {
        let mut current_section = None;
        let mut lines = Vec::new();

        for raw in content.lines() {
            let raw = raw.strip_suffix('\r').unwrap_or(raw).to_string();
            let kind = if let Some(section) = parse_section_header(&raw) {
                current_section = Some(section.clone());
                IniLineKind::Section { name: section }
            } else if let Some((key, _value)) = parse_key_value(&raw) {
                IniLineKind::KeyValue {
                    section: current_section.clone(),
                    key,
                }
            } else {
                IniLineKind::Other
            };

            lines.push(IniLine { raw, kind });
        }

        Self {
            lines,
            had_final_newline: content.ends_with('\n'),
        }
    }

    fn apply_patch(&mut self, patch: &PatchDocument) {
        for section in &patch.sections {
            self.apply_section(section);
        }
    }

    fn has_external_settings(&self, patch: &PatchDocument) -> bool {
        self.lines.iter().any(|line| match &line.kind {
            IniLineKind::KeyValue { section, key } => !patch.contains_key(section.as_deref(), key),
            IniLineKind::Section { .. } | IniLineKind::Other => false,
        })
    }

    fn has_managed_settings(&self, patch: &PatchDocument) -> bool {
        self.lines.iter().any(|line| match &line.kind {
            IniLineKind::KeyValue { section, key } => patch.contains_key(section.as_deref(), key),
            IniLineKind::Section { .. } | IniLineKind::Other => false,
        })
    }

    fn remove_inactive_managed_settings(
        &mut self,
        planned_patch: &PatchDocument,
        managed_patch: &PatchDocument,
    ) {
        self.lines.retain(|line| match &line.kind {
            IniLineKind::KeyValue { section, key } => {
                !managed_patch.contains_key(section.as_deref(), key)
                    || planned_patch.contains_key(section.as_deref(), key)
            }
            IniLineKind::Section { .. } | IniLineKind::Other => true,
        });
    }

    fn remove_inactive_managed_scaffolding(
        &mut self,
        planned_patch: &PatchDocument,
        managed_blocks: &[ManagedBlock],
    ) {
        let inactive_blocks = managed_blocks
            .iter()
            .filter(|block| !block.is_active(planned_patch))
            .collect::<Vec<_>>();
        if inactive_blocks.is_empty() {
            return;
        }

        let inactive_comments = inactive_blocks
            .iter()
            .flat_map(|block| block.comments.iter())
            .map(String::as_str)
            .collect::<Vec<_>>();
        self.lines.retain(|line| {
            !matches!(
                &line.kind,
                IniLineKind::Other if inactive_comments
                    .iter()
                    .any(|comment| line.raw.trim() == comment.trim())
            )
        });

        let inactive_sections = inactive_blocks
            .iter()
            .flat_map(|block| block.sections.iter())
            .map(String::as_str)
            .collect::<Vec<_>>();
        self.remove_empty_sections(&inactive_sections);
    }

    fn remove_empty_sections(&mut self, section_names: &[&str]) {
        let mut index = 0;
        while index < self.lines.len() {
            let should_remove = match &self.lines[index].kind {
                IniLineKind::Section { name } => {
                    section_names
                        .iter()
                        .any(|section_name| ini_name_eq(name, section_name))
                        && self.section_has_no_key_values(index)
                }
                IniLineKind::KeyValue { .. } | IniLineKind::Other => false,
            };

            if should_remove {
                self.lines.remove(index);
            } else {
                index += 1;
            }
        }
    }

    fn section_has_no_key_values(&self, section_index: usize) -> bool {
        for line in self.lines.iter().skip(section_index + 1) {
            match &line.kind {
                IniLineKind::Section { .. } => return true,
                IniLineKind::KeyValue { .. } => return false,
                IniLineKind::Other => {}
            }
        }

        true
    }

    fn ensure_block_comments(&mut self, planned_blocks: &[ManagedBlock]) {
        for block in planned_blocks {
            if block.comments.is_empty() || block.keys.is_empty() {
                continue;
            }

            let missing_comments = block
                .comments
                .iter()
                .filter(|comment| !self.contains_comment(comment))
                .collect::<Vec<_>>();
            if missing_comments.is_empty() {
                continue;
            }

            let Some(insert_index) = self.comment_insert_index(block) else {
                continue;
            };

            let needs_spacing = insert_index > 0
                && !self.lines[insert_index - 1].is_blank()
                && !matches!(
                    self.lines[insert_index - 1].kind,
                    IniLineKind::Section { .. }
                );
            let mut inserted_lines = Vec::new();
            if needs_spacing {
                inserted_lines.push(IniLine::other(String::new()));
            }
            inserted_lines.extend(
                missing_comments
                    .into_iter()
                    .map(|comment| IniLine::other(comment.clone())),
            );
            self.lines
                .splice(insert_index..insert_index, inserted_lines);
        }
    }

    fn contains_comment(&self, comment: &str) -> bool {
        self.lines.iter().any(|line| {
            matches!(&line.kind, IniLineKind::Other) && line.raw.trim() == comment.trim()
        })
    }

    fn comment_insert_index(&self, block: &ManagedBlock) -> Option<usize> {
        let key_index = self.first_key_index_for_block(block)?;
        let Some(section_index) = self.section_index_for_line(key_index) else {
            return Some(key_index);
        };
        let has_key_before_block = self.lines[section_index + 1..key_index]
            .iter()
            .any(|line| matches!(&line.kind, IniLineKind::KeyValue { .. }));

        if has_key_before_block {
            Some(key_index)
        } else {
            Some(section_index)
        }
    }

    fn first_key_index_for_block(&self, block: &ManagedBlock) -> Option<usize> {
        self.lines
            .iter()
            .enumerate()
            .find_map(|(index, line)| self.line_matches_block_key(line, block).then_some(index))
    }

    fn line_matches_block_key(&self, line: &IniLine, block: &ManagedBlock) -> bool {
        let IniLineKind::KeyValue {
            section: Some(section),
            key,
        } = &line.kind
        else {
            return false;
        };

        block.keys.iter().any(|block_key| {
            ini_name_eq(section, &block_key.section) && ini_name_eq(key, &block_key.key)
        })
    }

    fn section_index_for_line(&self, line_index: usize) -> Option<usize> {
        self.lines[..line_index]
            .iter()
            .rposition(|line| matches!(&line.kind, IniLineKind::Section { .. }))
    }

    fn normalize_blank_lines(&mut self) {
        let mut lines = Vec::with_capacity(self.lines.len());
        for line in self.lines.drain(..) {
            if line.is_blank() && (lines.is_empty() || lines.last().is_some_and(IniLine::is_blank))
            {
                continue;
            }
            lines.push(line);
        }

        while lines.last().is_some_and(IniLine::is_blank) {
            lines.pop();
        }

        self.lines = lines;
    }

    fn apply_section(&mut self, section: &PatchSection) {
        if self.first_section_index(&section.name).is_none() {
            self.append_section(section);
            return;
        }

        let mut missing_keys = Vec::new();
        for key in &section.keys {
            let matching_indices = self.key_indices_in_section(&section.name, &key.key);
            if let Some(first_index) = matching_indices.first().copied() {
                self.lines[first_index].raw =
                    replace_value_preserving_style(&self.lines[first_index].raw, &key.value);
                for index in matching_indices.iter().skip(1).rev() {
                    self.lines.remove(*index);
                }
            } else {
                missing_keys.push(key);
            }

            self.remove_top_level_key_duplicates(&key.key);
        }

        if missing_keys.is_empty() {
            return;
        }

        let insert_index = self.section_insert_index(&section.name);
        let inserted_lines = missing_keys
            .into_iter()
            .map(|key| IniLine::key_value(Some(section.name.clone()), key.raw.clone()))
            .collect::<Vec<_>>();
        self.lines
            .splice(insert_index..insert_index, inserted_lines);
    }

    fn append_section(&mut self, section: &PatchSection) {
        if !self.lines.is_empty() && !self.lines.last().is_some_and(IniLine::is_blank) {
            self.lines.push(IniLine::other(String::new()));
        }

        self.lines.push(IniLine::section(section.name.clone()));
        self.lines.extend(
            section
                .keys
                .iter()
                .map(|key| IniLine::key_value(Some(section.name.clone()), key.raw.clone())),
        );
    }

    fn first_section_index(&self, section_name: &str) -> Option<usize> {
        self.lines.iter().position(|line| match &line.kind {
            IniLineKind::Section { name } => ini_name_eq(name, section_name),
            IniLineKind::KeyValue { .. } | IniLineKind::Other => false,
        })
    }

    fn section_insert_index(&self, section_name: &str) -> usize {
        let Some(start_index) = self.first_section_index(section_name) else {
            return self.lines.len();
        };

        self.lines
            .iter()
            .enumerate()
            .skip(start_index + 1)
            .find_map(|(index, line)| match &line.kind {
                IniLineKind::Section { .. } => Some(index),
                IniLineKind::KeyValue { .. } | IniLineKind::Other => None,
            })
            .unwrap_or(self.lines.len())
    }

    fn key_indices_in_section(&self, section_name: &str, key_name: &str) -> Vec<usize> {
        self.lines
            .iter()
            .enumerate()
            .filter_map(|(index, line)| match &line.kind {
                IniLineKind::KeyValue {
                    section: Some(section),
                    key,
                } if ini_name_eq(section, section_name) && ini_name_eq(key, key_name) => {
                    Some(index)
                }
                IniLineKind::Section { .. } | IniLineKind::KeyValue { .. } | IniLineKind::Other => {
                    None
                }
            })
            .collect()
    }

    fn remove_top_level_key_duplicates(&mut self, key_name: &str) {
        self.lines.retain(|line| {
            !matches!(
                &line.kind,
                IniLineKind::KeyValue { section: None, key } if ini_name_eq(key, key_name)
            )
        });
    }

    fn render(&self, line_ending: &str) -> String {
        let mut content = self
            .lines
            .iter()
            .map(|line| line.raw.as_str())
            .collect::<Vec<_>>()
            .join(line_ending);

        if self.had_final_newline {
            content.push_str(line_ending);
        }

        content
    }
}

impl IniLine {
    fn section(name: String) -> Self {
        Self {
            raw: format!("[{name}]"),
            kind: IniLineKind::Section { name },
        }
    }

    fn key_value(section: Option<String>, raw: String) -> Self {
        let key = parse_key_value(&raw)
            .map(|(key, _value)| key)
            .unwrap_or_default();
        Self {
            raw,
            kind: IniLineKind::KeyValue { section, key },
        }
    }

    fn other(raw: String) -> Self {
        Self {
            raw,
            kind: IniLineKind::Other,
        }
    }

    fn is_blank(&self) -> bool {
        self.raw.trim().is_empty()
    }
}

impl PatchDocument {
    fn parse(content: &str) -> Self {
        let mut sections = Vec::new();
        let mut current_section = None;

        for raw in content.lines() {
            let raw = raw.strip_suffix('\r').unwrap_or(raw).to_string();
            if let Some(section) = parse_section_header(&raw) {
                upsert_patch_section(&mut sections, &section);
                current_section = Some(section);
                continue;
            }

            let Some(section) = current_section.as_ref() else {
                continue;
            };
            let Some((key, value)) = parse_key_value(&raw) else {
                continue;
            };

            let section = upsert_patch_section(&mut sections, section);
            if let Some(existing_key) = section
                .keys
                .iter_mut()
                .find(|existing_key| ini_name_eq(&existing_key.key, &key))
            {
                *existing_key = PatchKey { key, value, raw };
            } else {
                section.keys.push(PatchKey { key, value, raw });
            }
        }

        Self { sections }
    }

    fn contains_key(&self, section_name: Option<&str>, key_name: &str) -> bool {
        match section_name {
            Some(section_name) => self.sections.iter().any(|section| {
                ini_name_eq(&section.name, section_name)
                    && section
                        .keys
                        .iter()
                        .any(|key| ini_name_eq(&key.key, key_name))
            }),
            None => self.sections.iter().any(|section| {
                section
                    .keys
                    .iter()
                    .any(|key| ini_name_eq(&key.key, key_name))
            }),
        }
    }

    fn contains_key_value(&self, section_name: &str, key_name: &str, value: &str) -> bool {
        self.sections.iter().any(|section| {
            ini_name_eq(&section.name, section_name)
                && section
                    .keys
                    .iter()
                    .any(|key| ini_name_eq(&key.key, key_name) && key.value == value)
        })
    }
}

impl ManagedBlock {
    fn parse_all(content: &str) -> Vec<Self> {
        let mut blocks = Vec::new();
        let mut block_lines = Vec::new();
        let mut current_section = None;

        for raw in content.lines() {
            let raw = raw.strip_suffix('\r').unwrap_or(raw).to_string();
            if raw.trim().is_empty() {
                Self::push_block(&mut blocks, &mut block_lines, &mut current_section);
            } else {
                block_lines.push(raw);
            }
        }
        Self::push_block(&mut blocks, &mut block_lines, &mut current_section);

        blocks
    }

    fn push_block(
        blocks: &mut Vec<Self>,
        block_lines: &mut Vec<String>,
        current_section: &mut Option<String>,
    ) {
        if block_lines.is_empty() {
            return;
        }

        if let Some(block) = Self::parse(block_lines, current_section) {
            blocks.push(block);
        }
        block_lines.clear();
    }

    fn parse(lines: &[String], inherited_section: &mut Option<String>) -> Option<Self> {
        let mut comments: Vec<String> = Vec::new();
        let mut sections: Vec<String> = Vec::new();
        let mut keys: Vec<ManagedKey> = Vec::new();
        let mut current_section = inherited_section.clone();

        for raw in lines {
            if is_comment_line(raw) {
                comments.push(raw.clone());
                continue;
            }

            if let Some(section) = parse_section_header(raw) {
                if !sections
                    .iter()
                    .any(|existing_section| ini_name_eq(existing_section, &section))
                {
                    sections.push(section.clone());
                }
                current_section = Some(section);
                continue;
            }

            let Some(section) = current_section.as_ref() else {
                continue;
            };
            let Some((key, value)) = parse_key_value(raw) else {
                continue;
            };
            if !sections
                .iter()
                .any(|existing_section| ini_name_eq(existing_section, section))
            {
                sections.push(section.clone());
            }
            keys.push(ManagedKey {
                section: section.clone(),
                key,
                value,
            });
        }

        *inherited_section = current_section;
        (!sections.is_empty() || !keys.is_empty()).then_some(Self {
            comments,
            sections,
            keys,
        })
    }

    fn is_active(&self, planned_patch: &PatchDocument) -> bool {
        self.keys
            .iter()
            .any(|key| planned_patch.contains_key_value(&key.section, &key.key, &key.value))
    }
}

fn upsert_patch_section<'a>(
    sections: &'a mut Vec<PatchSection>,
    section_name: &str,
) -> &'a mut PatchSection {
    if let Some(index) = sections
        .iter()
        .position(|section| ini_name_eq(&section.name, section_name))
    {
        return &mut sections[index];
    }

    sections.push(PatchSection {
        name: section_name.to_string(),
        keys: Vec::new(),
    });
    let index = sections.len() - 1;
    &mut sections[index]
}

fn parse_section_header(line: &str) -> Option<String> {
    let trimmed = line.trim();
    if !trimmed.starts_with('[') {
        return None;
    }

    let close_index = trimmed.find(']')?;
    let suffix = trimmed[close_index + 1..].trim();
    if !suffix.is_empty() && !suffix.starts_with(';') && !suffix.starts_with('#') {
        return None;
    }

    let name = trimmed[1..close_index].trim();
    if name.is_empty() {
        None
    } else {
        Some(name.to_string())
    }
}

fn parse_key_value(line: &str) -> Option<(String, String)> {
    let trimmed = line.trim_start();
    if trimmed.is_empty() || trimmed.starts_with(';') || trimmed.starts_with('#') {
        return None;
    }

    let equals_index = line.find('=')?;
    let key = line[..equals_index].trim();
    if key.is_empty() {
        return None;
    }

    let value = extract_value(&line[equals_index + 1..]);
    Some((key.to_string(), value))
}

fn is_comment_line(line: &str) -> bool {
    let trimmed = line.trim_start();
    trimmed.starts_with(';') || trimmed.starts_with('#')
}

fn replace_value_preserving_style(line: &str, value: &str) -> String {
    let Some(equals_index) = line.find('=') else {
        return line.to_string();
    };
    let prefix = &line[..=equals_index];
    let suffix = &line[equals_index + 1..];
    let leading_len = suffix
        .char_indices()
        .find_map(|(index, character)| (!character.is_whitespace()).then_some(index))
        .unwrap_or(suffix.len());
    let leading = &suffix[..leading_len];
    let comment_suffix = find_inline_comment_start(suffix).map_or("", |comment_index| {
        let mut suffix_index = comment_index;
        while suffix_index > leading_len {
            let previous = suffix[..suffix_index].chars().next_back();
            if !previous.is_some_and(char::is_whitespace) {
                break;
            }
            suffix_index -= previous.map(char::len_utf8).unwrap_or_default();
        }
        &suffix[suffix_index..]
    });

    format!("{prefix}{leading}{value}{comment_suffix}")
}

fn extract_value(value_with_comment: &str) -> String {
    let value_end =
        find_inline_comment_start(value_with_comment).unwrap_or(value_with_comment.len());
    value_with_comment[..value_end].trim().to_string()
}

fn find_inline_comment_start(value: &str) -> Option<usize> {
    let mut previous_was_whitespace = false;
    for (index, character) in value.char_indices() {
        if matches!(character, ';' | '#') && previous_was_whitespace {
            return Some(index);
        }
        previous_was_whitespace = character.is_whitespace();
    }
    None
}

fn detect_line_ending(content: &str) -> &'static str {
    if content.contains("\r\n") {
        "\r\n"
    } else {
        "\n"
    }
}

fn ini_name_eq(left: &str, right: &str) -> bool {
    left.eq_ignore_ascii_case(right)
}
