// SPDX-FileCopyrightText: 2024 Joshua Goins <josh@redstrate.com>
// SPDX-License-Identifier: GPL-3.0-or-later

// TODO: finish up keyframe support
// TODO: figure out more node types

use std::io::Cursor;
use std::io::SeekFrom;

use crate::ByteSpan;
use crate::ReadableFile;
use crate::common::Platform;
use crate::common_file_operations::read_bool_from;
use crate::common_file_operations::read_string;
use crate::common_file_operations::write_bool_as;
use crate::common_file_operations::write_string;
use binrw::BinRead;
use binrw::binrw;

/// Where inside of the parent this node is aligned.
#[binrw]
#[brw(repr = u8)]
#[derive(Debug)]
pub enum AlignmentType {
    /// Aligned to the top left.
    TopLeft = 0x0,
    /// Aligned to the top.
    Top = 0x1,
    /// Aligned to the top right.
    TopRight = 0x2,
    /// Aligned to the left.
    Left = 0x3,
    /// Aligned in the center.
    Center = 0x4,
    /// Aligned to the right.
    Right = 0x5,
    /// Aligned to the bottom left.
    BottomLeft = 0x6,
    /// Aligned to the bottom.
    Bottom = 0x7,
    /// Aligned to the bottom right.
    BottomRight = 0x8,
}

#[binrw]
#[brw(repr = i32)]
#[derive(Debug, PartialEq, Eq, Clone, Copy)]
enum NodeType {
    Unk1 = 0x1,
    /// This node represents an image.
    Image = 0x2,
}

#[binrw]
#[br(import(node_type: NodeType))]
#[derive(Debug)]
enum NodeData {
    #[br(pre_assert(node_type == NodeType::Image))]
    Image {
        part_list_id: u32,
        part_id: u32,
        /// Whether the image should be horizontally flipped.
        #[br(map = read_bool_from::<u8>)]
        #[bw(map = write_bool_as::<u8>)]
        flip_horizontal: bool,
        /// Whether the image should be vertically flipped.
        #[br(map = read_bool_from::<u8>)]
        #[bw(map = write_bool_as::<u8>)]
        flip_vertical: bool,
        wrap: u8,
        unk1: u8,
    },
    Unknown,
}

/// A single widget.
#[binrw]
#[derive(Debug)]
pub struct WidgetNode {
    /// A integer identifier for this node.
    pub node_id: u32,
    /// If not zero, then it's the integer identifier of this node's parent.
    pub parent_id: i32,
    /// If not zero, then it's the integer identifier of the next sibling node.
    next_sibling_id: i32,
    /// If not zero, then it's the integer identifier of the previous sibling node.
    previous_sibling_id: i32,
    /// If not zero, then it's the integer identifier of this node's first child.
    child_node_id: i32,
    /// What kind of node this is.
    node_type: NodeType,
    node_offset: u16,
    tab_index: i16,
    unk1: [i32; 4],
    /// The X position, in pixels.
    pub x: i16,
    /// The Y position, in pixels.
    pub y: i16,
    /// Width, in pixels.
    pub width: u16,
    /// Height, in pixels.
    pub height: u16,
    rotation: f32,
    /// From 0.0 to 1.0, where 1.0 is "normal sized".
    pub scale_x: f32,
    /// From 0.0 to 1.0, where 1.0 is "normal sized".
    pub scale_y: f32,
    /// The X origin point (for rotation and scale?) in pixels.
    pub origin_x: i16,
    /// The Y origin point (for rotation and scale?) in pixels.
    pub origin_y: i16,
    priority: u16,
    unk2: u8,
    unk3: u8,
    /// From 0 to 100, where 100 is "normal color".
    pub multiply_red: i16,
    /// From 0 to 100, where 100 is "normal color".
    pub multiply_green: i16,
    /// From 0 to 100, where 100 is "normal color".
    pub multiply_blue: i16,
    /// From 0 to 100, where 0 is "normal color".
    pub add_red: i16,
    /// From 0 to 100, where 0 is "normal color".
    pub add_green: i16,
    /// From 0 to 100, where 0 is "normal color".
    pub add_blue: i16,
    /// From 0 to 255, where 255 is fully opaque.
    pub alpha: u8,
    clip_count: u8,
    /// ID of the associated timeline, see `Timeline`.
    pub timeline_id: u16,
    #[br(args(node_type))]
    data: NodeData,
}

/// Widget container containing nodes.
#[binrw]
#[derive(Debug)]
pub struct WidgetHeader {
    common: CommonHeader,

    unk1: u32, // TODO: probably the number of Widgets that each contain their own nodes
    unk2: i32,

    /// The integer ID of this widget.
    pub id: u32,
    /// Where this widget is aligned on the screen.
    pub alignment_type: AlignmentType,
    /// Whether this widget is themable.
    #[br(map = read_bool_from::<u8>)]
    #[bw(map = write_bool_as::<u8>)]
    pub supports_theming: bool,
    padding: [u8; 2],
    /// The widget's X position in pixels.
    pub x: i16,
    /// The widget's Y position in pixels.
    pub y: i16,
    node_count: u16,
    offset: u16,

    /// The nodes of this widget.
    #[br(count = node_count)]
    pub nodes: Vec<WidgetNode>,
}

#[binrw]
#[derive(Debug)]
struct TimelineKeyFrame {
    time: u32,
    offset: u16,
    interpolation: u8,
    unk1: u8,
    acceleration: f32,
    decelration: f32,
}

#[binrw]
#[derive(Debug)]
struct TimelineKeyGroup {
    usage: u16,
    key_group_type: u16,
    offset: u16,
    keyframe_count: u16,
    #[br(count = 0)]
    keyframes: Vec<TimelineKeyFrame>,
}

/// Represents a single frame of animation.
#[binrw]
#[derive(Debug)]
pub struct TimelineFrame {
    // TODO: lol what? why is this called TimelineFrame then?!
    /// The frame to start at.
    pub start_frame: u32,
    /// The frame to end on.
    pub end_frame: u32,
    offset: u32,
    keygroup_count: u32,
    #[br(count = keygroup_count)]
    keygroups: Vec<TimelineKeyGroup>,
}

/// Represents an animated timeline.
#[binrw]
#[derive(Debug)]
pub struct Timeline {
    /// Integer identifier for this timeline.
    id: u32,
    offset: u32,

    num_frames_1: u16,
    num_frames_2: u16,

    /// The frames of this timeline.
    #[br(count = num_frames_1 + num_frames_2)]
    pub frames: Vec<TimelineFrame>,
}

/// Contains timelines.
#[binrw]
#[derive(Debug)]
pub struct TimelineHeader {
    common: CommonHeader,

    /// The number of timelines.
    timeline_count: u32,
    unk2: i32,

    /// The contained timelines.
    #[br(count = timeline_count)]
    pub timelines: Vec<Timeline>,
}

/// Element that may contain a timeline or a widget.
#[binrw]
#[derive(Debug)]
pub struct AtkHeader {
    common: CommonHeader,

    /// Offset from the start of this `AtkHeader`, in bytes.
    asset_list_offset: u32,
    /// Offset from the start of this `AtkHeader`, in bytes.
    part_list_offset: u32,
    /// Offset from the start of this `AtkHeader`, in bytes.
    component_list_offset: u32,
    /// Offset from the start of this `AtkHeader`, in bytes.
    timeline_list_offset: u32,
    /// Offset from the start of this `AtkHeader`, in bytes.
    widget_offset: u32,
    /// Offset from the start of this `AtkHeader`, in bytes.
    rewrite_data_offset: u32,
    /// The number of available timelines.
    timeline_count: u32,

    /// The contained timeline.
    #[br(if(timeline_list_offset > 0))]
    #[br(restore_position, seek_before = SeekFrom::Current(timeline_list_offset as i64 - ATK_HEADER_SIZE as i64))]
    pub timeline: Option<TimelineHeader>,

    /// The contained widget.
    #[br(if(widget_offset > 0))]
    #[br(restore_position, seek_before = SeekFrom::Current(widget_offset as i64 - ATK_HEADER_SIZE as i64))]
    pub widget: Option<WidgetHeader>,
}

const ATK_HEADER_SIZE: usize = 36;

/// The common header for all ULD nodes.
#[binrw]
#[derive(Debug)]
struct CommonHeader {
    #[br(count = 4)]
    #[bw(pad_size_to = 4)]
    #[br(map = read_string)]
    #[bw(map = write_string)]
    identifier: String,

    // TODO: convert to integer automatically
    #[br(count = 4)]
    #[bw(pad_size_to = 4)]
    #[br(map = read_string)]
    #[bw(map = write_string)]
    version: String,
}

#[binrw]
#[derive(Debug)]
struct UldHeader {
    common: CommonHeader,

    /// Offset from the root of the file, in bytes.
    component_offset: u32,
    /// Offset from the root of the file, in bytes.
    widget_offset: u32,
}

/// UI layout definition file, usually with the `.ulb` file extension.
///
/// Does what it says: lays out UI elements.
#[binrw]
#[derive(Debug)]
pub struct Uld {
    header: UldHeader,

    // TODO: what is the difference between a component and a widget?
    /// The component portion of this ULD.
    #[br(restore_position)]
    #[br(seek_before = SeekFrom::Start(header.component_offset as u64))]
    pub component: AtkHeader,

    /// The widget portion of this ULD.
    #[br(restore_position)]
    #[br(seek_before = SeekFrom::Start(header.widget_offset as u64))]
    pub widget: AtkHeader,
}

impl ReadableFile for Uld {
    fn from_existing(platform: Platform, buffer: ByteSpan) -> Option<Self> {
        let mut cursor = Cursor::new(buffer);
        Uld::read_options(&mut cursor, platform.endianness(), ()).ok()
    }
}
#[cfg(test)]
mod tests {
    use crate::pass_random_invalid;

    use super::*;

    #[test]
    fn test_invalid() {
        pass_random_invalid::<Uld>();
    }
}
