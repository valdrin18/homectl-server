use dioxus::prelude::*;
use dioxus_router::Link;
use fermi::use_read;
use homectl_types::group::{FlattenedGroupConfig, GroupId};
use itertools::Itertools;
use palette::Hsv;

use crate::{
    app_state::{DEVICES_ATOM, GROUPS_ATOM},
    tile::Tile,
    util::{cmp_hsv, get_device_state_color, scale_hsv_value_to_display},
};

#[derive(Props, PartialEq)]
struct GroupRowProps {
    group_id: GroupId,
    name: String,
}

#[allow(non_snake_case)]
fn GroupRow(cx: Scope<GroupRowProps>) -> Element {
    let group_id = &cx.props.group_id;
    let name = &cx.props.name;

    let arrow_styles = r#"
        text-align: right;
        margin-right: 0.5rem;
        line-height: 1;
        font-size: 2rem;
    "#;

    let groups = use_read(&cx, GROUPS_ATOM);
    let devices = use_read(&cx, DEVICES_ATOM);

    let group_device_ids = groups
        .get(group_id)
        .map(|group| group.device_ids.clone())
        .unwrap_or_default();

    let group_devices = group_device_ids
        .iter()
        .filter_map(|id| devices.0.get(id))
        .map(|device| device.state.clone())
        .collect_vec();

    let group_colors: Vec<Hsv> = group_devices
        .iter()
        .filter_map(get_device_state_color)
        .map(scale_hsv_value_to_display)
        .sorted_by(cmp_hsv)
        .dedup()
        .collect();

    cx.render(rsx! {
        div {
            Link {
                to: "/groups/{group_id}",
                Tile {
                    full_width: true,
                    gradient: group_colors,
                    contents: cx.render(rsx! {
                        div {
                            flex: "1",
                            span {
                                padding: "0.5rem",
                                border_radius: "0.5rem",
                                background_color: "rgba(255, 255, 255, 0.5)",
                                "{name}"
                            }
                        }
                        div { style: "{arrow_styles}", ">" }
                    })
                }
            }
        }
    })
}

#[allow(non_snake_case)]
pub fn GroupList(cx: Scope) -> Element {
    let groups = use_read(&cx, GROUPS_ATOM);

    let groups: Vec<(GroupId, FlattenedGroupConfig)> = groups
        .iter()
        .map(|(group_id, config)| (group_id.clone(), config.clone()))
        .sorted_by(|a, b| a.1.name.cmp(&b.1.name))
        .collect();

    let groups = groups.iter().map(|(key, group)| {
        rsx! {
            GroupRow {
                key: "{key}",
                group_id: key.clone(),
                name: group.name.clone()
            }
        }
    });

    cx.render(rsx! {
        div {
            display: "flex",
            flex_direction: "column",
            margin: "1rem",
            gap: "1rem",
            groups
        }
    })
}