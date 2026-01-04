//! OpenAPI/Swagger configuration
//!
//! Generates OpenAPI documentation for the MMO API

use utoipa::{OpenApi, openapi::security::{HttpAuthScheme, HttpBuilder, SecurityScheme}};

/// Security addon for adding JWT bearer authentication scheme
struct SecurityAddon;

impl utoipa::Modify for SecurityAddon {
    fn modify(&self, openapi: &mut utoipa::openapi::OpenApi) {
        if let Some(components) = openapi.components.as_mut() {
            components.add_security_scheme(
                "bearer_auth",
                SecurityScheme::Http(
                    HttpBuilder::new()
                        .scheme(HttpAuthScheme::Bearer)
                        .bearer_format("JWT")
                        .build(),
                ),
            )
        }
    }
}

/// OpenAPI documentation
#[derive(OpenApi)]
#[openapi(
    info(
        title = "MMO API",
        version = "1.0.0",
        description = "Production-ready Rust API server with JWT authentication, MongoDB, and Redis",
        contact(
            name = "MMO Team",
            email = "support@mmo.example.com"
        ),
        license(
            name = "MIT",
        )
    ),
    servers(
        (url = "http://localhost:8080", description = "Local development server"),
        (url = "https://api.mmo.example.com", description = "Production server")
    ),
    tags(
        (name = "Auth", description = "Authentication endpoints"),
        (name = "User Wallet", description = "User-facing wallet endpoints (V3)"),
        (name = "Wallet - Internal", description = "Internal wallet endpoints for service-to-service communication"),
        (name = "Admin - Wallet Management", description = "Admin wallet management endpoints (V3)"),
        (name = "Permissions", description = "Permission and role management endpoints")
    ),
    paths(
        // Auth endpoints
        crate::modules::auth::handler::register,
        crate::modules::auth::handler::login,
        crate::modules::auth::handler::refresh_token,
        crate::modules::auth::handler::logout,
        crate::modules::auth::handler::get_me,
        crate::modules::auth::handler::change_password,

        // User Wallet endpoints (V3 - Public APIs)
        crate::modules::wallet::handler::initiate_deposit,
        crate::modules::wallet::handler::deposit_webhook,
        crate::modules::wallet::handler::get_deposit_status,
        crate::modules::wallet::handler::get_deposit_history,

        // Internal Wallet endpoints (Service-to-Service)
        crate::modules::wallet::handler::get_balance,
        crate::modules::wallet::handler::create_wallet,
        crate::modules::wallet::handler::create_auto_deposit,
        crate::modules::wallet::handler::manual_deposit,
        crate::modules::wallet::handler::create_withdrawal,
        crate::modules::wallet::handler::validate_withdrawal,
        crate::modules::wallet::handler::approve_withdrawal,
        crate::modules::wallet::handler::reject_withdrawal,
        crate::modules::wallet::handler::complete_bank_transfer,
        crate::modules::wallet::handler::create_purchase,
        crate::modules::wallet::handler::early_release_escrow,
        crate::modules::wallet::handler::create_dispute,
        crate::modules::wallet::handler::resolve_dispute_refund,
        crate::modules::wallet::handler::resolve_dispute_release,
        crate::modules::wallet::handler::manual_debit,
        crate::modules::wallet::handler::freeze_wallet,
        crate::modules::wallet::handler::unfreeze_wallet,
        crate::modules::wallet::handler::set_shop_commission,
        crate::modules::wallet::handler::get_admin_logs,
        crate::modules::wallet::handler::get_transaction_history,
        crate::modules::wallet::handler::process_auto_releases,

        // Admin Wallet endpoints (V3 - Public APIs)
        crate::modules::wallet::handler::admin_manual_deposit,
        crate::modules::wallet::handler::admin_get_deposits_history,
        crate::modules::wallet::handler::get_dashboard_stats,
        crate::modules::wallet::handler::trigger_reconciliation,
        crate::modules::wallet::handler::start_cron_jobs,
        crate::modules::wallet::handler::stop_cron_jobs,

        // Permission endpoints
        crate::modules::permissions::handler::list_permissions,
        crate::modules::permissions::handler::create_role,
        crate::modules::permissions::handler::list_roles,
        crate::modules::permissions::handler::update_role_permissions,
        crate::modules::permissions::handler::delete_role,
        crate::modules::permissions::handler::assign_role,
    ),
    components(
        schemas(
            // Auth schemas
            crate::modules::auth::dto::RegisterRequest,
            crate::modules::auth::dto::LoginRequest,
            crate::modules::auth::dto::RefreshTokenRequest,
            crate::modules::auth::dto::LogoutRequest,
            crate::modules::auth::dto::ChangePasswordRequest,
            crate::modules::auth::dto::AuthResponse,
            crate::modules::auth::dto::UserResponse,

            // Core schemas
            crate::core::errors::ApiError,
            crate::core::errors::ErrorResponse,
            crate::core::response::MessageResponse,

            // Wallet DTOs
            crate::modules::wallet::dto::WalletBalanceResponse,
            crate::modules::wallet::dto::WalletInfoResponse,
            crate::modules::wallet::dto::AutoDepositRequest,
            crate::modules::wallet::dto::ManualDepositRequest,
            crate::modules::wallet::dto::DepositResponse,
            crate::modules::wallet::dto::WithdrawalRequest,
            crate::modules::wallet::dto::WithdrawalResponse,
            crate::modules::wallet::dto::BankInfo,
            crate::modules::wallet::dto::PurchaseRequest,
            crate::modules::wallet::dto::PurchaseResponse,
            crate::modules::wallet::dto::EarlyReleaseRequest,
            crate::modules::wallet::dto::EscrowInfoResponse,
            crate::modules::wallet::dto::RefundRequest,
            crate::modules::wallet::dto::SellerCancelRequest,
            crate::modules::wallet::dto::AdminDebitRequest,
            crate::modules::wallet::dto::AdminFreezeRequest,
            crate::modules::wallet::dto::AdminUnfreezeRequest,
            crate::modules::wallet::dto::AdminWithdrawalDecisionRequest,
            crate::modules::wallet::dto::SetCommissionRateRequest,
            crate::modules::wallet::dto::TransactionHistoryQuery,
            crate::modules::wallet::dto::TransactionResponse,
            crate::modules::wallet::dto::TransactionListResponse,
            crate::modules::wallet::dto::SnapshotResponse,
            crate::modules::wallet::dto::ReconciliationReportResponse,
            crate::modules::wallet::dto::ReconciliationCheck,
            crate::modules::wallet::dto::AdminDashboardStats,
            crate::modules::wallet::dto::PendingWithdrawalItem,
            crate::modules::wallet::dto::SuccessResponse,
            crate::modules::wallet::dto::CreateWalletRequest,
            crate::modules::wallet::dto::DisputeRequest,
            crate::modules::wallet::dto::ResolveDisputeRequest,
            crate::modules::wallet::dto::RejectWithdrawalRequest,
            crate::modules::wallet::dto::CompleteBankTransferRequest,
            crate::modules::wallet::dto::AdminLogQuery,

            // V3 Deposit DTOs
            crate::modules::wallet::dto::DepositInitiateRequest,
            crate::modules::wallet::dto::DepositInitiateResponse,
            crate::modules::wallet::dto::PaymentWebhookPayload,
            crate::modules::wallet::dto::DepositStatusResponse,
            crate::modules::wallet::dto::DepositHistoryResponse,
            crate::modules::wallet::dto::DepositHistoryQuery,
            crate::modules::wallet::dto::AdminManualDepositRequest,
            crate::modules::wallet::dto::AdminDepositHistoryQuery,

            // Admin Dashboard DTOs
            crate::modules::wallet::dto::DashboardStatsResponse,
            crate::modules::wallet::dto::ReconciliationResponse,
            crate::modules::wallet::dto::ReconciliationDiscrepancy,

            // Note: TransactionHistoryResponse and AdminLogResponse excluded (contain domain types without ToSchema)
            crate::modules::wallet::dto::ProcessAutoReleaseResponse,

            // Wallet Domain types (only enums and simple structs with ToSchema)
            crate::modules::wallet::domain::WalletType,
            crate::modules::wallet::domain::WalletStatus,
            crate::modules::wallet::domain::TransactionType,
            crate::modules::wallet::domain::Direction,
            crate::modules::wallet::domain::BalanceType,
            crate::modules::wallet::domain::TransactionStatus,
            crate::modules::wallet::domain::ReferenceType,
            crate::modules::wallet::domain::WithdrawalStatus,
            crate::modules::wallet::domain::ValidationResult,
            crate::modules::wallet::domain::CheckResult,
            crate::modules::wallet::domain::ValidationError,
            crate::modules::wallet::domain::Severity,
            crate::modules::wallet::domain::SnapshotStatus,
            crate::modules::wallet::domain::EscrowStatus,
            crate::modules::wallet::domain::ReleaseType,
            crate::modules::wallet::domain::AdminOperation,
            crate::modules::wallet::domain::TargetType,
            crate::modules::wallet::domain::DepositStatus,

            // Permission DTOs
            crate::modules::permissions::dto::PermissionResponse,
            crate::modules::permissions::dto::CreateRoleRequest,
            crate::modules::permissions::dto::UpdateRolePermissionsRequest,
            crate::modules::permissions::dto::AssignUserRoleRequest,
            crate::modules::permissions::dto::RoleResponse,
            crate::modules::permissions::dto::UserPermissionsResponse,
        ),
    ),
    modifiers(&SecurityAddon)
)]
pub struct ApiDoc;
