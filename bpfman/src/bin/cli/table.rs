// SPDX-License-Identifier: Apache-2.0
// Copyright Authors of bpfman

use bpfman::types::{ImagePullPolicy, Location, Program};
use comfy_table::{Cell, Color, Table};
use hex::encode_upper;
pub(crate) struct ProgTable(Table);

impl ProgTable {
    pub(crate) fn new_program(program: &Program) -> Result<Self, anyhow::Error> {
        let mut table = Table::new();

        table.load_preset(comfy_table::presets::NOTHING);
        table.set_header(vec![
            Cell::new("Bpfman State")
                .add_attribute(comfy_table::Attribute::Bold)
                .add_attribute(comfy_table::Attribute::Underlined)
                .fg(Color::Green),
        ]);

        let data = program.get_data();

        let name = data.get_name()?;
        if name.is_empty() {
            table.add_row(vec!["Name:", "None"]);
        } else {
            table.add_row(vec!["Name:", &name]);
        }

        match data.get_location()? {
            Location::Image(i) => {
                table.add_row(vec!["Image URL:", &i.image_url]);
                table.add_row(vec![
                    "Pull Policy:",
                    &format! { "{}", TryInto::<ImagePullPolicy>::try_into(i.image_pull_policy)?},
                ]);
            }
            Location::File(p) => {
                table.add_row(vec!["Path:", &p]);
            }
        };

        let global_data = data.get_global_data()?;
        if global_data.is_empty() {
            table.add_row(vec!["Global:", "None"]);
        } else {
            let mut first = true;
            for (key, value) in global_data {
                let data = &format! {"{key}={}", encode_upper(value)};
                if first {
                    first = false;
                    table.add_row(vec!["Global:", data]);
                } else {
                    table.add_row(vec!["", data]);
                }
            }
        }

        let metadata = data.get_metadata()?;
        if metadata.is_empty() {
            table.add_row(vec!["Metadata:", "None"]);
        } else {
            let mut first = true;
            for (key, value) in metadata.clone() {
                let data = &format! {"{key}={value}"};
                if first {
                    first = false;
                    table.add_row(vec!["Metadata:", data]);
                } else {
                    table.add_row(vec!["", data]);
                }
            }
        }

        if let Some(map_pin_path) = data.get_map_pin_path()? {
            table.add_row(vec![
                "Map Pin Path:",
                map_pin_path
                    .to_str()
                    .expect("map_pin_path is not valid Unicode"),
            ]);
        } else {
            table.add_row(vec!["Map Pin Path:", "None"]);
        }

        match data.get_map_owner_id()? {
            Some(id) => table.add_row(vec!["Map Owner ID:", &id.to_string()]),
            None => table.add_row(vec!["Map Owner ID:", "None"]),
        };

        let map_used_by = data.get_maps_used_by()?;
        if map_used_by.is_empty() {
            table.add_row(vec!["Maps Used By:", "None"]);
        } else {
            let mut first = true;
            for prog_id in map_used_by {
                if first {
                    first = false;
                    table.add_row(vec!["Maps Used By:", &prog_id.to_string()]);
                } else {
                    table.add_row(vec!["", &prog_id.to_string()]);
                }
            }
        };
        let links = data.get_link_ids()?;
        if links.is_empty() {
            table.add_row(vec!["Links:", "None"]);
        } else {
            let mut first = true;
            for link in links {
                let data = &format! {"{link}"};
                if first {
                    first = false;
                    table.add_row(vec!["Links:", data]);
                } else {
                    table.add_row(vec!["", data]);
                }
            }
        }
        Ok(ProgTable(table))
    }

    pub(crate) fn new_kernel_info(r: &Program) -> Result<Self, anyhow::Error> {
        let mut table = Table::new();

        table.load_preset(comfy_table::presets::NOTHING);
        table.set_header(vec![
            Cell::new("Kernel State")
                .add_attribute(comfy_table::Attribute::Bold)
                .add_attribute(comfy_table::Attribute::Underlined)
                .fg(Color::Green),
        ]);

        let p = r.get_data();
        let name = p.get_kernel_name()?;

        let rows = vec![
            vec!["Program ID:".to_string(), p.get_id()?.to_string()],
            vec!["Name:".to_string(), name],
            vec!["Type:".to_string(), format!("{}", r.kind())],
            vec!["Loaded At:".to_string(), p.get_kernel_loaded_at()?],
            vec!["Tag:".to_string(), p.get_kernel_tag()?],
            vec![
                "GPL Compatible:".to_string(),
                p.get_kernel_gpl_compatible()?.to_string(),
            ],
            vec![
                "Map IDs:".to_string(),
                format!("{:?}", p.get_kernel_map_ids()?),
            ],
            vec!["BTF ID:".to_string(), p.get_kernel_btf_id()?.to_string()],
            vec![
                "Size Translated (bytes):".to_string(),
                p.get_kernel_bytes_xlated()?.to_string(),
            ],
            vec!["JITted:".to_string(), p.get_kernel_jited()?.to_string()],
            vec![
                "Size JITted:".to_string(),
                p.get_kernel_bytes_jited()?.to_string(),
            ],
            vec![
                "Kernel Allocated Memory (bytes):".to_string(),
                p.get_kernel_bytes_memlock()?.to_string(),
            ],
            vec![
                "Verified Instruction Count:".to_string(),
                p.get_kernel_verified_insns()?.to_string(),
            ],
        ];
        table.add_rows(rows);
        Ok(ProgTable(table))
    }

    pub(crate) fn new_list() -> Self {
        let mut table = Table::new();

        table.load_preset(comfy_table::presets::NOTHING);
        table.set_header(vec!["Program ID", "Name", "Type", "Load Time", "Links"]);
        ProgTable(table)
    }

    pub(crate) fn add_row_list(
        &mut self,
        id: String,
        name: String,
        type_: String,
        load_time: String,
        links: String,
    ) {
        self.0.add_row(vec![id, name, type_, load_time, links]);
    }

    pub(crate) fn add_response_prog(&mut self, r: Program) -> anyhow::Result<()> {
        let data = r.get_data();

        self.add_row_list(
            data.get_id()?.to_string(),
            data.get_kernel_name()?,
            r.kind().to_string(),
            data.get_kernel_loaded_at()?,
            data.get_link_ids()?
                .into_iter()
                .map(|m| m.to_string())
                .collect::<Vec<String>>()
                .join(", "),
        );

        Ok(())
    }

    pub(crate) fn print(&self) {
        println!("{self}\n")
    }
}

impl std::fmt::Display for ProgTable {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}
