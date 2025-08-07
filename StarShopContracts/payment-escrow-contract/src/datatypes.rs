use soroban_sdk::{
    Address,
    contracterror, contracttype, String
};


#[derive(Clone)]
#[contracttype]
pub enum DataKey {
    Escrow(String),
    Balance(Address),
    Allowance(AllowanceDataKey),
    Arbitrator,
    DisputedPayments,
    ResolvedDisputes,
    SellerRegId(Address),
    PaymentCounter,
}

// Error definitions
#[contracterror]
#[derive(Copy, Clone, Debug, Eq, PartialEq, PartialOrd, Ord)]
#[repr(u32)]
pub enum PaymentEscrowError {
    NotInitialized = 1,
    AlreadyInitialized = 2,
    UnauthorizedAccess = 3,
    InsufficientFunds = 4,
    TransferFailed = 5,
    InvalidAmount = 6,
    CannotPaySelf = 7,
    DepositPaymentFailed = 8,
    NotFound = 9,
    NotDelivered = 10,
    NotCompleted = 11,
    NotValid = 12,
    DisputePeriodExpired = 13,
    AlreadyDisputed = 14,
    NotArbitrator = 15,
    NotExpired = 16,
    NotSeller = 17,
    ArbitratorAlreadyExists = 18,
    PaymentDisputed = 19,
}

// Status Enum
#[contracttype]
#[derive(Clone, PartialEq, Debug)]
pub enum PaymentStatus {
    Pending,      // Funds held in contract
    Delivered,    // Buyer has confirmed delivery
    Completed,    // Funds released to seller
    Disputed,     // Funds locked, awaiting resolution
    Refunded,     // Funds returned to buyer
    Expired,      // Auto-refunded due to timeout
}



#[contracttype]
#[derive(Clone, PartialEq)]
pub enum DisputeDecision {
    RefundBuyer = 0,
    PaySeller = 1,
}


#[contracttype]
#[derive(Clone)]
pub struct DisputeEvent {
    pub order_id: u128,
    pub initiator: Address,
    pub reason: String,
}


#[contracttype]
#[derive(Clone)]
pub struct DisputeResolvedEvent {
    pub order_id: u128,
    pub resolution: DisputeDecision,
    pub admin: Address,
}


#[contracttype]
#[derive(Clone, Debug)]
pub struct Payment {
    pub id: u128,
    pub buyer: Address,
    pub seller: Address,
    pub amount: i128,
    pub token: Address,
    pub status: PaymentStatus,
    pub created_at: u64,
    pub expiry: u64,
    pub dispute_deadline: u64,
    pub description: String,
}



#[contracttype]
#[derive(Clone)]
pub struct AllowanceValue {
    pub amount: i128,
    pub expiration_ledger: u32,
}

#[contracttype]
#[derive(Clone)]
pub struct AllowanceDataKey {
    pub from: Address,
    pub spender: Address,
}

#[contracttype]
#[derive(Clone)]
pub struct DeliveryDetails {
    pub payment_id: u128,
    pub buyer: Address,
    pub seller: Address,
    pub status: PaymentStatus,
    pub created_at: u64,
    pub expiry: u64,
    pub description: String,
}