use codec::{Decode, Encode};
use gstd::{prelude::*, TypeInfo};

#[derive(Debug, Clone, Encode, Decode, TypeInfo, Hash)]
pub enum RunStatus {
    /// Indicates that `Run` is in bidding stage.
    Created,

    /// Indicates that `Run` is canceled.
    Canceled,

    /// Indicates that `Run` is in progress.
    InProgress,

    /// Indicates that `Run` is finished.
    Finished { horse_name: String, run_id: u128 },
}
