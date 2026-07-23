use chunkedge_binary::{Decode, Encode};

use crate::Packet;
use crate::packets::configuration::custom_report_details_s2c::CustomReportDetail;

#[derive(Clone, Debug, Encode, Decode, Packet)]
pub struct CustomReportDetailsS2c<'a> {
    pub details: Vec<CustomReportDetail<'a>>,
}
