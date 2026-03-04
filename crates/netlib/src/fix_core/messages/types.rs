/// `1` = Market, `2` = Limit, `3` = Stop, `4` = Stop Limit
#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OrdType {
    Market = 1,
    Limit = 2,
    Stop = 3,
    StopLimit = 4,
}

/// `1` = Buy, `2` = Sell
#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Side {
    Buy = 1,
    Sell = 2,
}

/// `0` = Open, `C` = Close
#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OpenClose {
    Open = b'0',
    Close = b'C',
}

/// `0` = Put, `1` = Call
#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PutOrCall {
    Put = 0,
    Call = 1,
}

/// `1` = Order Cancel Request, `2` = Order Cancel Replace Request
#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CxlRejResponseTo {
    OrderCancelRequest = 1,
    OrderCancelReplaceRequest = 2,
}

/// Status of order that was to have been canceled or modified.
#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OrdStatus {
    New = 0,
    PartiallyFilled = 1,
    Filled = 2,
    Canceled = 4,
    Rejected = 8,
}
