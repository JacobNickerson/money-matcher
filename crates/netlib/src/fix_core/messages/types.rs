/// Tag 40 - OrdType
/// `1` = Market
/// `2` = Limit
/// `3` = Stop
/// `4` = Stop Limit
#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OrdType {
    Market = b'1',
    Limit = b'2',
    Stop = b'3',
    StopLimit = b'4',
}

/// Tag 54 - Side
/// `1` = Buy
/// `2` = Sell
#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Side {
    Buy = b'1',
    Sell = b'2',
}

/// Tag 77 - OpenClose
/// `0` = Open
/// `C` = Close
#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OpenClose {
    Open = b'0',
    Close = b'C',
}

/// Tag 201 - PutOrCall
/// `0` = Put
/// `1` = Call
#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PutOrCall {
    Put = b'0',
    Call = b'1',
}

/// Tag 434 - CxlRejResponseTo
/// `1` = Order Cancel Request
/// `2` = Order Cancel Replace Request
#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CxlRejResponseTo {
    OrderCancelRequest = b'1',
    OrderCancelReplaceRequest = b'2',
}

/// Tag 39 - OrdStatus
/// `0` = New
/// `1` = Partially Filled
/// `2` = Filled
/// `3` = Done For Day
/// `4` = Canceled
/// `5` = Replaced
/// `6` = Pending Cancel
/// `8` = Rejected
/// `E` = Pending Replace
#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OrdStatus {
    New = b'0',
    PartiallyFilled = b'1',
    Filled = b'2',
    DoneForDay = b'3',
    Canceled = b'4',
    Replaced = b'5',
    PendingCancel = b'6',
    Rejected = b'8',
    PendingReplace = b'E',
}

/// Tag 20 - ExecTransType
/// `0` = New
/// `1` = Cancel
/// `2` = Correct
/// `3` = Status
#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ExecTransType {
    New = b'0',
    Cancel = b'1',
    Correct = b'2',
    Status = b'3',
}

/// Tag 150 - ExecType
/// `0` = New
/// `1` = Partially Filled
/// `2` = Filled
/// `3` = Done For Day
/// `4` = Canceled
/// `5` = Replace
/// `6` = Pending Cancel
/// `8` = Rejected
/// `D` = Restated
/// `E` = Pending Replace
#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ExecType {
    New = b'0',
    PartiallyFilled = b'1',
    Filled = b'2',
    DoneForDay = b'3',
    Canceled = b'4',
    Replace = b'5',
    PendingCancel = b'6',
    Rejected = b'8',
    Restated = b'D',
    PendingReplace = b'E',
}

/// Tag 204 - CustomerOrFirm
/// `0` = Customer
/// `1` = Proprietary Firm
/// `2` = Broker/Dealer Firm
/// `3` = Broker/Dealer Customer
/// `4` = ISE Market Maker
/// `5` = Far Market Maker
/// `6` = Retail Customer
/// `7` = Proprietary Customer
/// `8` = Customer Professional
/// `9` = Joint Back Office
#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CustomerOrFirm {
    Customer = b'0',
    ProprietaryFirm = b'1',
    BrokerDealerFirm = b'2',
    BrokerDealerCustomer = b'3',
    IseMarketMaker = b'4',
    FarMarketMaker = b'5',
    RetailCustomer = b'6',
    ProprietaryCustomer = b'7',
    CustomerProfessional = b'8',
    JointBackOffice = b'9',
}
