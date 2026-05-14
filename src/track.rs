use std::fmt::Write;

use unicode_width::UnicodeWidthStr;

use crate::dangos::{RefDango, Run};

pub const TRACK_LEN: usize = 32;

#[derive(Debug, Clone, Copy)]
pub enum PointType {
    Accelerate,
    Decelerate,
    Hole,
    Common,
}

pub type Map = [PointType; TRACK_LEN];
pub type Point = Vec<RefDango>;
pub type Track = [Point; TRACK_LEN];

/// 初始地图
pub fn init_map() -> Map {
    let mut points = [PointType::Common; TRACK_LEN];
    points[2] = PointType::Accelerate;
    points[5] = PointType::Hole;
    points[9] = PointType::Decelerate;
    points[10] = PointType::Accelerate;
    points[15] = PointType::Accelerate;
    points[19] = PointType::Hole;
    points[22] = PointType::Accelerate;
    points[27] = PointType::Decelerate;
    points
}

/// 初始团子赛道
pub fn init_track(dangos: &[RefDango]) -> Track {
    let mut track = [const { vec![] }; TRACK_LEN];
    track[0] = dangos.iter().rev().cloned().collect(); //堆叠顺序与行动顺序相反
    track
}

pub fn show_track(round: usize, track: &Track) {
    const ROW_NUM: usize = 8;
    const COL_NUM: usize = 4;
    const COL_WIDTH: usize = 42;
    const LINE_WIDTH: usize = COL_WIDTH * COL_NUM;
    const SEP_NUM: usize = (LINE_WIDTH - 4) / 2;

    let mut track_state = String::new();
    for row in 0..ROW_NUM {
        for col in 0..COL_NUM {
            let idx = row + col * 8;
            let point = &track[idx];
            let mut cell = format!("{:2}: ", idx + 1);
            for dango in point {
                write!(
                    &mut cell,
                    "{}({}) ",
                    dango.borrow().shortname(),
                    dango.borrow().get_arrive_count()
                )
                .unwrap();
            }
            let cell_width = UnicodeWidthStr::width(cell.as_str());
            write!(&mut track_state, "{cell}").unwrap();
            if cell_width < COL_WIDTH {
                write!(&mut track_state, "{}", " ".repeat(COL_WIDTH - cell_width)).unwrap();
            }
        }
        track_state.push('\n');
    }

    println!(
        "{} {:02} {}\n{}",
        "=".repeat(SEP_NUM),
        round,
        "=".repeat(SEP_NUM),
        track_state
    );
}
