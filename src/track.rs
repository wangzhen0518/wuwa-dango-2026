use crate::dangos::RefDango;

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
