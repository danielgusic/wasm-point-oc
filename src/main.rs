use sci_rs::{
    SCIMessageType, SCITelegram,
    scip::{SCIPointLocation, SCIPointTargetLocation},
};

unsafe extern "C" {
    fn recv_msg(buf: *mut u8) -> usize;
    fn send_msg(msg: *const u8, len: usize);
    fn move_point(cmd: MovePointCmd) -> MovePointResult;
}

const SELF_ID: &'static str = "P01";

#[repr(C)]
enum MovePointCmd {
    Left,
    Right,
}

impl From<SCIPointTargetLocation> for MovePointCmd {
    fn from(value: SCIPointTargetLocation) -> Self {
        match value {
            SCIPointTargetLocation::PointLocationChangeToRight => Self::Right,
            SCIPointTargetLocation::PointLocationChangeToLeft => Self::Left,
        }
    }
}

#[repr(C)]
enum MovePointResult {
    EndPositionArrived,
    Trailed,
}

fn main() {
    let mut recv_buf = [0; 1024];
    let mut state = SCIPointLocation::PointLocationLeft;
    loop {
        let cmd_len = unsafe { recv_msg(recv_buf.as_mut_ptr()) };
        let telegram = SCITelegram::try_from(&recv_buf[..cmd_len]).unwrap();
        if telegram.message_type == SCIMessageType::scip_location_status() {
            let resp = SCITelegram::location_status(SELF_ID, &telegram.sender, state);
            let resp_bytes: Vec<u8> = resp.into();
            unsafe { send_msg(resp_bytes.as_ptr(), resp_bytes.len()) };
        }
        if telegram.message_type == SCIMessageType::scip_change_location() {
            let direction = SCIPointTargetLocation::try_from(telegram.payload.data[0]).unwrap();
            state = match unsafe { move_point(direction.into()) } {
                MovePointResult::EndPositionArrived => match direction {
                    SCIPointTargetLocation::PointLocationChangeToRight => {
                        SCIPointLocation::PointLocationRight
                    }
                    SCIPointTargetLocation::PointLocationChangeToLeft => {
                        SCIPointLocation::PointLocationLeft
                    }
                },
                MovePointResult::Trailed => SCIPointLocation::PointBumped,
            };
            let resp = SCITelegram::location_status(SELF_ID, &telegram.sender, state);
            let resp_bytes: Vec<u8> = resp.into();
            unsafe { send_msg(resp_bytes.as_ptr(), resp_bytes.len()) };
        }
    }
}
