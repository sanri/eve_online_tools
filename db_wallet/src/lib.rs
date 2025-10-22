pub mod entities;
mod m20220101_000001_create_table;

pub use sea_orm_migration::prelude::*;
use serde::{Deserialize, Serialize};
use strum::{AsRefStr, EnumIter, EnumString, FromRepr, IntoStaticStr};

pub struct Migrator;

#[async_trait::async_trait]
impl MigratorTrait for Migrator {
    fn migrations() -> Vec<Box<dyn MigrationTrait>> {
        vec![Box::new(m20220101_000001_create_table::Migration)]
    }
}

#[derive(
    Serialize,
    Deserialize,
    EnumString,
    AsRefStr,
    IntoStaticStr,
    EnumIter,
    FromRepr,
    Copy,
    Clone,
    PartialEq,
    Eq,
    Ord,
    PartialOrd,
)]
#[serde(rename_all = "snake_case")]
#[strum(serialize_all = "snake_case")]
#[repr(i32)]
pub enum JournalRefType {
    PlayerTrading = 1,
    MarketTransaction = 2,
    GmCashTransfer = 3,
    MissionReward = 7,
    CloneActivation = 8,
    Inheritance = 9,
    PlayerDonation = 10,
    CorporationPayment = 11,
    DockingFee = 12,
    OfficeRentalFee = 13,
    FactorySlotRentalFee = 14,
    RepairBill = 15,
    Bounty = 16,
    BountyPrize = 17,
    Insurance = 19,
    MissionExpiration = 20,
    MissionCompletion = 21,
    Shares = 22,
    CourierMissionEscrow = 23,
    MissionCost = 24,
    AgentMiscellaneous = 25,
    LpStore = 26,
    AgentLocationServices = 27,
    AgentDonation = 28,
    AgentSecurityServices = 29,
    AgentMissionCollateralPaid = 30,
    AgentMissionCollateralRefunded = 31,
    AgentsPreward = 32,
    AgentMissionReward = 33,
    AgentMissionTimeBonusReward = 34,
    Cspa = 35,
    Cspaofflinerefund = 36,
    CorporationAccountWithdrawal = 37,
    CorporationDividendPayment = 38,
    CorporationRegistrationFee = 39,
    CorporationLogoChangeCost = 40,
    ReleaseOfImpoundedProperty = 41,
    MarketEscrow = 42,
    AgentServicesRendered = 43,
    MarketFinePaid = 44,
    CorporationLiquidation = 45,
    BrokersFee = 46,
    CorporationBulkPayment = 47,
    AllianceRegistrationFee = 48,
    WarFee = 49,
    AllianceMaintainanceFee = 50,
    ContrabandFine = 51,
    CloneTransfer = 52,
    AccelerationGateFee = 53,
    TransactionTax = 54,
    JumpCloneInstallationFee = 55,
    Manufacturing = 56,
    ResearchingTechnology = 57,
    ResearchingTimeProductivity = 58,
    ResearchingMaterialProductivity = 59,
    Copying = 60,
    ReverseEngineering = 62,
    ContractAuctionBid = 63,
    ContractAuctionBidRefund = 64,
    ContractCollateral = 65,
    ContractRewardRefund = 66,
    ContractAuctionSold = 67,
    ContractReward = 68,
    ContractCollateralRefund = 69,
    ContractCollateralPayout = 70,
    ContractPrice = 71,
    ContractBrokersFee = 72,
    ContractSalesTax = 73,
    ContractDeposit = 74,
    ContractDepositSalesTax = 75,
    ContractAuctionBidCorp = 77,
    ContractCollateralDepositedCorp = 78,
    ContractPricePaymentCorp = 79,
    ContractBrokersFeeCorp = 80,
    ContractDepositCorp = 81,
    ContractDepositRefund = 82,
    ContractRewardDeposited = 83,
    ContractRewardDepositedCorp = 84,
    BountyPrizes = 85,
    AdvertisementListingFee = 86,
    MedalCreation = 87,
    MedalIssued = 88,
    DnaModificationFee = 90,
    SovereignityBill = 91,
    BountyPrizeCorporationTax = 92,
    AgentMissionRewardCorporationTax = 93,
    AgentMissionTimeBonusRewardCorporationTax = 94,
    UpkeepAdjustmentFee = 95,
    PlanetaryImportTax = 96,
    PlanetaryExportTax = 97,
    PlanetaryConstruction = 98,
    CorporateRewardPayout = 99,
    BountySurcharge = 101,
    ContractReversal = 102,
    CorporateRewardTax = 103,
    StorePurchase = 106,
    StorePurchaseRefund = 107,
    DatacoreFee = 112,
    WarFeeSurrender = 113,
    WarAllyContract = 114,
    BountyReimbursement = 115,
    KillRightFee = 116,
    SecurityProcessingFee = 117,
    IndustryJobTax = 120,
    InfrastructureHubMaintenance = 122,
    AssetSafetyRecoveryTax = 123,
    OpportunityReward = 124,
    ProjectDiscoveryReward = 125,
    ProjectDiscoveryTax = 126,
    ReprocessingTax = 127,
    JumpCloneActivationFee = 128,
    OperationBonus = 129,
    ResourceWarsReward = 131,
    DuelWagerEscrow = 132,
    DuelWagerPayment = 133,
    DuelWagerRefund = 134,
    Reaction = 135,
    ExternalTradeFreeze = 136,
    ExternalTradeThaw = 137,
    ExternalTradeDelivery = 138,
    SeasonChallengeReward = 139,
    SkillPurchase = 141,
    ItemTraderPayment = 142,
    FluxTicketSale = 143,
    FluxPayout = 144,
    FluxTax = 145,
    FluxTicketRepayment = 146,
    RedeemedIskToken = 147,
    DailyChallengeReward = 148,
    MarketProviderTax = 149,
    EssEscrowTransfer = 155,
    MilestoneRewardPayment = 156,
    UnderConstruction = 166,
    AllignmentBasedGateToll = 168,
    ProjectPayouts = 170,
    InsurgencyCorruptionContributionReward = 172,
    InsurgencySuppressionContributionReward = 173,
    DailyGoalPayouts = 174,
    DailyGoalPayoutsTax = 175,
    CosmeticMarketComponentItemPurchase = 178,
    CosmeticMarketSkinSaleBrokerFee = 179,
    CosmeticMarketSkinPurchase = 180,
    CosmeticMarketSkinSale = 181,
    CosmeticMarketSkinSaleTax = 182,
    CosmeticMarketSkinTransaction = 183,
    SkyhookClaimFee = 184,
    AirCareerProgramReward = 185,
    FreelanceJobsDurationFee = 186,
    FreelanceJobsBroadcastingFee = 187,
    FreelanceJobsRewardEscrow = 188,
    FreelanceJobsReward = 189,
    FreelanceJobsEscrowRefund = 190,
    FreelanceJobsRewardCorporationTax = 191,
    GmPlexFeeRefund = 192,
}

impl JournalRefType {
    pub fn zh_str(&self) -> &'static str {
        match self {
            JournalRefType::PlayerDonation => "玩家捐助",    // 10
            JournalRefType::OfficeRentalFee => "办公室租金", // 13
            JournalRefType::AgentMissionReward => "代理人任务奖励", // 33
            JournalRefType::AgentMissionTimeBonusReward => "代理人任务时间加成奖励", // 34
            JournalRefType::CorporationAccountWithdrawal => "军团账户支取", // 37
            JournalRefType::CorporationDividendPayment => "军团奖励支付", // 38
            JournalRefType::BountyPrizes => "追击赏金",      // 85
            JournalRefType::ProjectDiscoveryReward => "探索计划奖励", // 125
            JournalRefType::EssEscrowTransfer => "事件监测装置保证金支付", // 155
            JournalRefType::DailyGoalPayouts => "每日目标奖励", // 174
            _ => self.into(),
        }
    }
}

#[derive(Serialize, Deserialize, EnumString, AsRefStr, EnumIter, FromRepr, Copy, Clone)]
#[serde(rename_all = "snake_case")]
#[strum(serialize_all = "snake_case")]
#[repr(i32)]
pub enum ContextIdType {
    StructureId = 1,
    StationId = 2,
    MarketTransactionId = 3,
    CharacterId = 4,
    CorporationId = 5,
    AllianceId = 6,
    EveSystem = 7,
    IndustryJobId = 8,
    ContractId = 9,
    PlanetId = 10,
    SystemId = 11,
    TypeId = 12,
}
