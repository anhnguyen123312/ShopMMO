//! Permission constants - hardcoded actions for authorization
//!
//! Permissions are defined as compile-time constants using Enum.
//! This provides type-safety and IDE autocomplete support.
//!
//! Format: RESOURCE:ACTION:SCOPE (e.g., product:create:own, wallet:read:admin)
//! Similar to AWS IAM permission model with added scope granularity.
//!
//! Scopes:
//! - `own` - User can only perform action on their own resources
//! - `all` - User can perform action on any resource (typically admin)
//! - `admin` - Admin-only operations
//! - No scope suffix means the action applies universally

use std::collections::HashSet;

/// Permission enum - type-safe permission definitions
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Permission {
    // ========================================================================
    // PRODUCT PERMISSIONS
    // ========================================================================
    ProductCreateOwn,  // product:create:own - Seller creates own product
    ProductReadOwn,    // product:read:own - Read own products
    ProductReadAll,    // product:read:all - Read all products (admin/public)
    ProductUpdateOwn,  // product:update:own - Update own product
    ProductUpdateAll,  // product:update:all - Update any product (admin)
    ProductDeleteOwn,  // product:delete:own - Delete own product
    ProductDeleteAll,  // product:delete:all - Delete any product (admin)
    ProductListOwn,    // product:list:own - List own products
    ProductListAll,    // product:list:all - List all products
    ProductPublish,    // product:publish - Publish product (make visible)
    ProductUnpublish,  // product:unpublish - Unpublish product
    ProductBulkUpdate, // product:bulk_update - Bulk update products
    ProductImport,     // product:import - Import products from file
    ProductExport,     // product:export - Export products to file

    // ========================================================================
    // ORDER PERMISSIONS
    // ========================================================================
    OrderCreateOwn, // order:create:own - Buyer creates order
    OrderReadOwn,   // order:read:own - Read own orders (buyer/seller)
    OrderReadAll,   // order:read:all - Read all orders (admin)
    OrderUpdateOwn, // order:update:own - Update own order
    OrderUpdateAll, // order:update:all - Update any order (admin)
    OrderListOwn,   // order:list:own - List own orders
    OrderListAll,   // order:list:all - List all orders (admin)
    OrderCancel,    // order:cancel - Cancel order
    OrderConfirm,   // order:confirm - Seller confirms order
    OrderShip,      // order:ship - Mark order as shipped
    OrderComplete,  // order:complete - Mark order as complete
    OrderRefund,    // order:refund - Process refund (admin)

    // ========================================================================
    // WALLET PERMISSIONS
    // ========================================================================
    // Basic wallet operations
    WalletReadOwn, // wallet:read:own - Read own wallet balance
    WalletReadAll, // wallet:read:all - Read all wallets (admin)
    WalletListAll, // wallet:list:all - List all wallets (admin)

    // Deposit operations
    WalletDepositOwn,     // wallet:deposit:own - User initiates deposit
    WalletDepositManual,  // wallet:deposit:manual - Admin manual deposit
    WalletDepositAuto,    // wallet:deposit:auto - System auto deposit
    WalletDepositWebhook, // wallet:deposit:webhook - Process payment webhook

    // Withdrawal operations
    WalletWithdrawOwn,      // wallet:withdraw:own - User requests withdrawal
    WalletWithdrawValidate, // wallet:withdraw:validate - Admin validates withdrawal
    WalletWithdrawApprove,  // wallet:withdraw:approve - Admin approves withdrawal
    WalletWithdrawReject,   // wallet:withdraw:reject - Admin rejects withdrawal
    WalletWithdrawComplete, // wallet:withdraw:complete - Admin completes transfer

    // Escrow operations
    WalletEscrowCreate,       // wallet:escrow:create - Create escrow (purchase)
    WalletEscrowRelease,      // wallet:escrow:release - Release escrow to seller
    WalletEscrowRefund,       // wallet:escrow:refund - Refund escrow to buyer
    WalletEscrowEarlyRelease, // wallet:escrow:early_release - Buyer early release
    WalletEscrowAutoRelease,  // wallet:escrow:auto_release - System auto-release job

    // Admin wallet operations
    WalletFreeze,        // wallet:freeze - Freeze wallet (admin)
    WalletUnfreeze,      // wallet:unfreeze - Unfreeze wallet (admin)
    WalletDebitAdmin,    // wallet:debit:admin - Admin manual debit
    WalletCreditAdmin,   // wallet:credit:admin - Admin manual credit
    WalletSetCommission, // wallet:set_commission - Set shop commission rate
    WalletReconcile,     // wallet:reconcile - Trigger reconciliation
    WalletDashboard,     // wallet:dashboard - View admin dashboard
    WalletLogs,          // wallet:logs - View admin operation logs
    WalletCronManage,    // wallet:cron:manage - Start/stop cron jobs

    // ========================================================================
    // DISPUTE PERMISSIONS
    // ========================================================================
    DisputeCreateOwn,      // dispute:create:own - Buyer creates dispute
    DisputeReadOwn,        // dispute:read:own - Read own disputes
    DisputeReadAll,        // dispute:read:all - Read all disputes (admin)
    DisputeListOwn,        // dispute:list:own - List own disputes
    DisputeListAll,        // dispute:list:all - List all disputes (admin)
    DisputeRespondSeller,  // dispute:respond:seller - Seller responds to dispute
    DisputeRespondBuyer,   // dispute:respond:buyer - Buyer responds to dispute
    DisputeEscalate,       // dispute:escalate - Escalate to admin
    DisputeResolveRefund,  // dispute:resolve:refund - Admin resolves with refund
    DisputeResolveRelease, // dispute:resolve:release - Admin resolves with release
    DisputePartialRefund,  // dispute:partial_refund - Admin partial refund
    DisputeExtendDeadline, // dispute:extend_deadline - Admin extends deadline
    DisputeAutoEscalate,   // dispute:auto_escalate - System auto-escalate job

    // ========================================================================
    // USER PERMISSIONS
    // ========================================================================
    UserCreate,        // user:create - Create user (registration)
    UserReadOwn,       // user:read:own - Read own profile
    UserReadAll,       // user:read:all - Read any user (admin)
    UserUpdateOwn,     // user:update:own - Update own profile
    UserUpdateAll,     // user:update:all - Update any user (admin)
    UserDeleteOwn,     // user:delete:own - Delete own account
    UserDeleteAll,     // user:delete:all - Delete any user (admin)
    UserListAll,       // user:list:all - List all users (admin)
    UserSuspend,       // user:suspend - Suspend user (admin)
    UserActivate,      // user:activate - Activate user (admin)
    UserVerifyEmail,   // user:verify_email - Verify email
    UserResetPassword, // user:reset_password - Reset password
    UserAssignRoles,   // user:assign_roles - Assign roles (admin)
    UserViewRoles,     // user:view_roles - View user roles (admin)

    // ========================================================================
    // ROLE PERMISSIONS
    // ========================================================================
    RoleCreate,            // role:create - Create role (admin)
    RoleRead,              // role:read - Read roles
    RoleUpdate,            // role:update - Update role (admin)
    RoleDelete,            // role:delete - Delete role (admin)
    RoleList,              // role:list - List all roles
    RoleAssignPermissions, // role:assign_permissions - Assign permissions to role

    // ========================================================================
    // SHOP PERMISSIONS
    // ========================================================================
    ShopCreateOwn,      // shop:create:own - Vendor creates shop
    ShopReadOwn,        // shop:read:own - Read own shop
    ShopReadAll,        // shop:read:all - Read any shop
    ShopUpdateOwn,      // shop:update:own - Update own shop
    ShopUpdateAll,      // shop:update:all - Update any shop (admin)
    ShopDeleteOwn,      // shop:delete:own - Delete own shop
    ShopDeleteAll,      // shop:delete:all - Delete any shop (admin)
    ShopListAll,        // shop:list:all - List all shops
    ShopVerifyTelegram, // shop:verify:telegram - Verify telegram
    ShopSuspend,        // shop:suspend - Suspend shop (admin)
    ShopActivate,       // shop:activate - Activate shop (admin)
    ShopSetCommission,  // shop:set_commission - Set commission (admin)
    ShopUploadLogo,     // shop:upload:logo - Upload shop logo
    ShopUploadBanner,   // shop:upload:banner - Upload shop banner
    ShopUpdatePolicies, // shop:update:policies - Update shop policies
    ShopViewStats,      // shop:view:stats - View shop statistics (admin)

    // ========================================================================
    // CATEGORY PERMISSIONS
    // ========================================================================
    CategoryCreate,  // category:create - Create category (admin)
    CategoryRead,    // category:read - Read categories (public)
    CategoryUpdate,  // category:update - Update category (admin)
    CategoryDelete,  // category:delete - Delete category (admin)
    CategoryList,    // category:list - List categories (public)
    CategoryReorder, // category:reorder - Reorder categories (admin)
    CategoryTree,    // category:tree - Get category tree (public)

    // ========================================================================
    // ADMIN PERMISSIONS (Super admin only)
    // ========================================================================
    AdminFull,         // admin:full - Full admin access
    AdminRead,         // admin:read - Read admin resources
    AdminWrite,        // admin:write - Write admin resources
    AdminSystemConfig, // admin:system:config - System configuration
    AdminAuditLogs,    // admin:audit:logs - View audit logs
}

impl Permission {
    pub fn as_str(&self) -> &'static str {
        match self {
            // Product
            Self::ProductCreateOwn => "product:create:own",
            Self::ProductReadOwn => "product:read:own",
            Self::ProductReadAll => "product:read:all",
            Self::ProductUpdateOwn => "product:update:own",
            Self::ProductUpdateAll => "product:update:all",
            Self::ProductDeleteOwn => "product:delete:own",
            Self::ProductDeleteAll => "product:delete:all",
            Self::ProductListOwn => "product:list:own",
            Self::ProductListAll => "product:list:all",
            Self::ProductPublish => "product:publish",
            Self::ProductUnpublish => "product:unpublish",
            Self::ProductBulkUpdate => "product:bulk_update",
            Self::ProductImport => "product:import",
            Self::ProductExport => "product:export",

            // Order
            Self::OrderCreateOwn => "order:create:own",
            Self::OrderReadOwn => "order:read:own",
            Self::OrderReadAll => "order:read:all",
            Self::OrderUpdateOwn => "order:update:own",
            Self::OrderUpdateAll => "order:update:all",
            Self::OrderListOwn => "order:list:own",
            Self::OrderListAll => "order:list:all",
            Self::OrderCancel => "order:cancel",
            Self::OrderConfirm => "order:confirm",
            Self::OrderShip => "order:ship",
            Self::OrderComplete => "order:complete",
            Self::OrderRefund => "order:refund",

            // Wallet - Basic
            Self::WalletReadOwn => "wallet:read:own",
            Self::WalletReadAll => "wallet:read:all",
            Self::WalletListAll => "wallet:list:all",

            // Wallet - Deposit
            Self::WalletDepositOwn => "wallet:deposit:own",
            Self::WalletDepositManual => "wallet:deposit:manual",
            Self::WalletDepositAuto => "wallet:deposit:auto",
            Self::WalletDepositWebhook => "wallet:deposit:webhook",

            // Wallet - Withdrawal
            Self::WalletWithdrawOwn => "wallet:withdraw:own",
            Self::WalletWithdrawValidate => "wallet:withdraw:validate",
            Self::WalletWithdrawApprove => "wallet:withdraw:approve",
            Self::WalletWithdrawReject => "wallet:withdraw:reject",
            Self::WalletWithdrawComplete => "wallet:withdraw:complete",

            // Wallet - Escrow
            Self::WalletEscrowCreate => "wallet:escrow:create",
            Self::WalletEscrowRelease => "wallet:escrow:release",
            Self::WalletEscrowRefund => "wallet:escrow:refund",
            Self::WalletEscrowEarlyRelease => "wallet:escrow:early_release",
            Self::WalletEscrowAutoRelease => "wallet:escrow:auto_release",

            // Wallet - Admin
            Self::WalletFreeze => "wallet:freeze",
            Self::WalletUnfreeze => "wallet:unfreeze",
            Self::WalletDebitAdmin => "wallet:debit:admin",
            Self::WalletCreditAdmin => "wallet:credit:admin",
            Self::WalletSetCommission => "wallet:set_commission",
            Self::WalletReconcile => "wallet:reconcile",
            Self::WalletDashboard => "wallet:dashboard",
            Self::WalletLogs => "wallet:logs",
            Self::WalletCronManage => "wallet:cron:manage",

            // Dispute
            Self::DisputeCreateOwn => "dispute:create:own",
            Self::DisputeReadOwn => "dispute:read:own",
            Self::DisputeReadAll => "dispute:read:all",
            Self::DisputeListOwn => "dispute:list:own",
            Self::DisputeListAll => "dispute:list:all",
            Self::DisputeRespondSeller => "dispute:respond:seller",
            Self::DisputeRespondBuyer => "dispute:respond:buyer",
            Self::DisputeEscalate => "dispute:escalate",
            Self::DisputeResolveRefund => "dispute:resolve:refund",
            Self::DisputeResolveRelease => "dispute:resolve:release",
            Self::DisputePartialRefund => "dispute:partial_refund",
            Self::DisputeExtendDeadline => "dispute:extend_deadline",
            Self::DisputeAutoEscalate => "dispute:auto_escalate",

            // User
            Self::UserCreate => "user:create",
            Self::UserReadOwn => "user:read:own",
            Self::UserReadAll => "user:read:all",
            Self::UserUpdateOwn => "user:update:own",
            Self::UserUpdateAll => "user:update:all",
            Self::UserDeleteOwn => "user:delete:own",
            Self::UserDeleteAll => "user:delete:all",
            Self::UserListAll => "user:list:all",
            Self::UserSuspend => "user:suspend",
            Self::UserActivate => "user:activate",
            Self::UserVerifyEmail => "user:verify_email",
            Self::UserResetPassword => "user:reset_password",
            Self::UserAssignRoles => "user:assign_roles",
            Self::UserViewRoles => "user:view_roles",

            // Role
            Self::RoleCreate => "role:create",
            Self::RoleRead => "role:read",
            Self::RoleUpdate => "role:update",
            Self::RoleDelete => "role:delete",
            Self::RoleList => "role:list",
            Self::RoleAssignPermissions => "role:assign_permissions",

            // Shop
            Self::ShopCreateOwn => "shop:create:own",
            Self::ShopReadOwn => "shop:read:own",
            Self::ShopReadAll => "shop:read:all",
            Self::ShopUpdateOwn => "shop:update:own",
            Self::ShopUpdateAll => "shop:update:all",
            Self::ShopDeleteOwn => "shop:delete:own",
            Self::ShopDeleteAll => "shop:delete:all",
            Self::ShopListAll => "shop:list:all",
            Self::ShopVerifyTelegram => "shop:verify:telegram",
            Self::ShopSuspend => "shop:suspend",
            Self::ShopActivate => "shop:activate",
            Self::ShopSetCommission => "shop:set_commission",
            Self::ShopUploadLogo => "shop:upload:logo",
            Self::ShopUploadBanner => "shop:upload:banner",
            Self::ShopUpdatePolicies => "shop:update:policies",
            Self::ShopViewStats => "shop:view:stats",

            // Category
            Self::CategoryCreate => "category:create",
            Self::CategoryRead => "category:read",
            Self::CategoryUpdate => "category:update",
            Self::CategoryDelete => "category:delete",
            Self::CategoryList => "category:list",
            Self::CategoryReorder => "category:reorder",
            Self::CategoryTree => "category:tree",

            // Admin
            Self::AdminFull => "admin:full",
            Self::AdminRead => "admin:read",
            Self::AdminWrite => "admin:write",
            Self::AdminSystemConfig => "admin:system:config",
            Self::AdminAuditLogs => "admin:audit:logs",
        }
    }

    pub fn from_str(s: &str) -> Option<Self> {
        match s {
            // Product
            "product:create:own" => Some(Self::ProductCreateOwn),
            "product:read:own" => Some(Self::ProductReadOwn),
            "product:read:all" => Some(Self::ProductReadAll),
            "product:update:own" => Some(Self::ProductUpdateOwn),
            "product:update:all" => Some(Self::ProductUpdateAll),
            "product:delete:own" => Some(Self::ProductDeleteOwn),
            "product:delete:all" => Some(Self::ProductDeleteAll),
            "product:list:own" => Some(Self::ProductListOwn),
            "product:list:all" => Some(Self::ProductListAll),
            "product:publish" => Some(Self::ProductPublish),
            "product:unpublish" => Some(Self::ProductUnpublish),
            "product:bulk_update" => Some(Self::ProductBulkUpdate),
            "product:import" => Some(Self::ProductImport),
            "product:export" => Some(Self::ProductExport),

            // Order
            "order:create:own" => Some(Self::OrderCreateOwn),
            "order:read:own" => Some(Self::OrderReadOwn),
            "order:read:all" => Some(Self::OrderReadAll),
            "order:update:own" => Some(Self::OrderUpdateOwn),
            "order:update:all" => Some(Self::OrderUpdateAll),
            "order:list:own" => Some(Self::OrderListOwn),
            "order:list:all" => Some(Self::OrderListAll),
            "order:cancel" => Some(Self::OrderCancel),
            "order:confirm" => Some(Self::OrderConfirm),
            "order:ship" => Some(Self::OrderShip),
            "order:complete" => Some(Self::OrderComplete),
            "order:refund" => Some(Self::OrderRefund),

            // Wallet - Basic
            "wallet:read:own" => Some(Self::WalletReadOwn),
            "wallet:read:all" => Some(Self::WalletReadAll),
            "wallet:list:all" => Some(Self::WalletListAll),

            // Wallet - Deposit
            "wallet:deposit:own" => Some(Self::WalletDepositOwn),
            "wallet:deposit:manual" => Some(Self::WalletDepositManual),
            "wallet:deposit:auto" => Some(Self::WalletDepositAuto),
            "wallet:deposit:webhook" => Some(Self::WalletDepositWebhook),

            // Wallet - Withdrawal
            "wallet:withdraw:own" => Some(Self::WalletWithdrawOwn),
            "wallet:withdraw:validate" => Some(Self::WalletWithdrawValidate),
            "wallet:withdraw:approve" => Some(Self::WalletWithdrawApprove),
            "wallet:withdraw:reject" => Some(Self::WalletWithdrawReject),
            "wallet:withdraw:complete" => Some(Self::WalletWithdrawComplete),

            // Wallet - Escrow
            "wallet:escrow:create" => Some(Self::WalletEscrowCreate),
            "wallet:escrow:release" => Some(Self::WalletEscrowRelease),
            "wallet:escrow:refund" => Some(Self::WalletEscrowRefund),
            "wallet:escrow:early_release" => Some(Self::WalletEscrowEarlyRelease),
            "wallet:escrow:auto_release" => Some(Self::WalletEscrowAutoRelease),

            // Wallet - Admin
            "wallet:freeze" => Some(Self::WalletFreeze),
            "wallet:unfreeze" => Some(Self::WalletUnfreeze),
            "wallet:debit:admin" => Some(Self::WalletDebitAdmin),
            "wallet:credit:admin" => Some(Self::WalletCreditAdmin),
            "wallet:set_commission" => Some(Self::WalletSetCommission),
            "wallet:reconcile" => Some(Self::WalletReconcile),
            "wallet:dashboard" => Some(Self::WalletDashboard),
            "wallet:logs" => Some(Self::WalletLogs),
            "wallet:cron:manage" => Some(Self::WalletCronManage),

            // Dispute
            "dispute:create:own" => Some(Self::DisputeCreateOwn),
            "dispute:read:own" => Some(Self::DisputeReadOwn),
            "dispute:read:all" => Some(Self::DisputeReadAll),
            "dispute:list:own" => Some(Self::DisputeListOwn),
            "dispute:list:all" => Some(Self::DisputeListAll),
            "dispute:respond:seller" => Some(Self::DisputeRespondSeller),
            "dispute:respond:buyer" => Some(Self::DisputeRespondBuyer),
            "dispute:escalate" => Some(Self::DisputeEscalate),
            "dispute:resolve:refund" => Some(Self::DisputeResolveRefund),
            "dispute:resolve:release" => Some(Self::DisputeResolveRelease),
            "dispute:partial_refund" => Some(Self::DisputePartialRefund),
            "dispute:extend_deadline" => Some(Self::DisputeExtendDeadline),
            "dispute:auto_escalate" => Some(Self::DisputeAutoEscalate),

            // User
            "user:create" => Some(Self::UserCreate),
            "user:read:own" => Some(Self::UserReadOwn),
            "user:read:all" => Some(Self::UserReadAll),
            "user:update:own" => Some(Self::UserUpdateOwn),
            "user:update:all" => Some(Self::UserUpdateAll),
            "user:delete:own" => Some(Self::UserDeleteOwn),
            "user:delete:all" => Some(Self::UserDeleteAll),
            "user:list:all" => Some(Self::UserListAll),
            "user:suspend" => Some(Self::UserSuspend),
            "user:activate" => Some(Self::UserActivate),
            "user:verify_email" => Some(Self::UserVerifyEmail),
            "user:reset_password" => Some(Self::UserResetPassword),
            "user:assign_roles" => Some(Self::UserAssignRoles),
            "user:view_roles" => Some(Self::UserViewRoles),

            // Role
            "role:create" => Some(Self::RoleCreate),
            "role:read" => Some(Self::RoleRead),
            "role:update" => Some(Self::RoleUpdate),
            "role:delete" => Some(Self::RoleDelete),
            "role:list" => Some(Self::RoleList),
            "role:assign_permissions" => Some(Self::RoleAssignPermissions),

            // Shop
            "shop:create:own" => Some(Self::ShopCreateOwn),
            "shop:read:own" => Some(Self::ShopReadOwn),
            "shop:read:all" => Some(Self::ShopReadAll),
            "shop:update:own" => Some(Self::ShopUpdateOwn),
            "shop:update:all" => Some(Self::ShopUpdateAll),
            "shop:delete:own" => Some(Self::ShopDeleteOwn),
            "shop:delete:all" => Some(Self::ShopDeleteAll),
            "shop:list:all" => Some(Self::ShopListAll),
            "shop:verify:telegram" => Some(Self::ShopVerifyTelegram),
            "shop:suspend" => Some(Self::ShopSuspend),
            "shop:activate" => Some(Self::ShopActivate),
            "shop:set_commission" => Some(Self::ShopSetCommission),
            "shop:upload:logo" => Some(Self::ShopUploadLogo),
            "shop:upload:banner" => Some(Self::ShopUploadBanner),
            "shop:update:policies" => Some(Self::ShopUpdatePolicies),
            "shop:view:stats" => Some(Self::ShopViewStats),

            // Category
            "category:create" => Some(Self::CategoryCreate),
            "category:read" => Some(Self::CategoryRead),
            "category:update" => Some(Self::CategoryUpdate),
            "category:delete" => Some(Self::CategoryDelete),
            "category:list" => Some(Self::CategoryList),
            "category:reorder" => Some(Self::CategoryReorder),
            "category:tree" => Some(Self::CategoryTree),

            // Admin
            "admin:full" => Some(Self::AdminFull),
            "admin:read" => Some(Self::AdminRead),
            "admin:write" => Some(Self::AdminWrite),
            "admin:system:config" => Some(Self::AdminSystemConfig),
            "admin:audit:logs" => Some(Self::AdminAuditLogs),

            _ => None,
        }
    }

    pub fn resource(&self) -> &'static str {
        self.as_str().split(':').next().unwrap_or("")
    }

    pub fn action(&self) -> &'static str {
        self.as_str().split(':').nth(1).unwrap_or("")
    }

    pub fn scope(&self) -> Option<&'static str> {
        self.as_str().split(':').nth(2)
    }
}

pub fn all_permissions() -> Vec<&'static str> {
    vec![
        // Product
        Permission::ProductCreateOwn.as_str(),
        Permission::ProductReadOwn.as_str(),
        Permission::ProductReadAll.as_str(),
        Permission::ProductUpdateOwn.as_str(),
        Permission::ProductUpdateAll.as_str(),
        Permission::ProductDeleteOwn.as_str(),
        Permission::ProductDeleteAll.as_str(),
        Permission::ProductListOwn.as_str(),
        Permission::ProductListAll.as_str(),
        Permission::ProductPublish.as_str(),
        Permission::ProductUnpublish.as_str(),
        Permission::ProductBulkUpdate.as_str(),
        Permission::ProductImport.as_str(),
        Permission::ProductExport.as_str(),
        // Order
        Permission::OrderCreateOwn.as_str(),
        Permission::OrderReadOwn.as_str(),
        Permission::OrderReadAll.as_str(),
        Permission::OrderUpdateOwn.as_str(),
        Permission::OrderUpdateAll.as_str(),
        Permission::OrderListOwn.as_str(),
        Permission::OrderListAll.as_str(),
        Permission::OrderCancel.as_str(),
        Permission::OrderConfirm.as_str(),
        Permission::OrderShip.as_str(),
        Permission::OrderComplete.as_str(),
        Permission::OrderRefund.as_str(),
        // Wallet
        Permission::WalletReadOwn.as_str(),
        Permission::WalletReadAll.as_str(),
        Permission::WalletListAll.as_str(),
        Permission::WalletDepositOwn.as_str(),
        Permission::WalletDepositManual.as_str(),
        Permission::WalletDepositAuto.as_str(),
        Permission::WalletDepositWebhook.as_str(),
        Permission::WalletWithdrawOwn.as_str(),
        Permission::WalletWithdrawValidate.as_str(),
        Permission::WalletWithdrawApprove.as_str(),
        Permission::WalletWithdrawReject.as_str(),
        Permission::WalletWithdrawComplete.as_str(),
        Permission::WalletEscrowCreate.as_str(),
        Permission::WalletEscrowRelease.as_str(),
        Permission::WalletEscrowRefund.as_str(),
        Permission::WalletEscrowEarlyRelease.as_str(),
        Permission::WalletEscrowAutoRelease.as_str(),
        Permission::WalletFreeze.as_str(),
        Permission::WalletUnfreeze.as_str(),
        Permission::WalletDebitAdmin.as_str(),
        Permission::WalletCreditAdmin.as_str(),
        Permission::WalletSetCommission.as_str(),
        Permission::WalletReconcile.as_str(),
        Permission::WalletDashboard.as_str(),
        Permission::WalletLogs.as_str(),
        Permission::WalletCronManage.as_str(),
        // Dispute
        Permission::DisputeCreateOwn.as_str(),
        Permission::DisputeReadOwn.as_str(),
        Permission::DisputeReadAll.as_str(),
        Permission::DisputeListOwn.as_str(),
        Permission::DisputeListAll.as_str(),
        Permission::DisputeRespondSeller.as_str(),
        Permission::DisputeRespondBuyer.as_str(),
        Permission::DisputeEscalate.as_str(),
        Permission::DisputeResolveRefund.as_str(),
        Permission::DisputeResolveRelease.as_str(),
        Permission::DisputePartialRefund.as_str(),
        Permission::DisputeExtendDeadline.as_str(),
        Permission::DisputeAutoEscalate.as_str(),
        // User
        Permission::UserCreate.as_str(),
        Permission::UserReadOwn.as_str(),
        Permission::UserReadAll.as_str(),
        Permission::UserUpdateOwn.as_str(),
        Permission::UserUpdateAll.as_str(),
        Permission::UserDeleteOwn.as_str(),
        Permission::UserDeleteAll.as_str(),
        Permission::UserListAll.as_str(),
        Permission::UserSuspend.as_str(),
        Permission::UserActivate.as_str(),
        Permission::UserVerifyEmail.as_str(),
        Permission::UserResetPassword.as_str(),
        Permission::UserAssignRoles.as_str(),
        Permission::UserViewRoles.as_str(),
        // Role
        Permission::RoleCreate.as_str(),
        Permission::RoleRead.as_str(),
        Permission::RoleUpdate.as_str(),
        Permission::RoleDelete.as_str(),
        Permission::RoleList.as_str(),
        Permission::RoleAssignPermissions.as_str(),
        // Shop
        Permission::ShopCreateOwn.as_str(),
        Permission::ShopReadOwn.as_str(),
        Permission::ShopReadAll.as_str(),
        Permission::ShopUpdateOwn.as_str(),
        Permission::ShopUpdateAll.as_str(),
        Permission::ShopDeleteOwn.as_str(),
        Permission::ShopDeleteAll.as_str(),
        Permission::ShopListAll.as_str(),
        Permission::ShopVerifyTelegram.as_str(),
        Permission::ShopSuspend.as_str(),
        Permission::ShopActivate.as_str(),
        Permission::ShopSetCommission.as_str(),
        Permission::ShopUploadLogo.as_str(),
        Permission::ShopUploadBanner.as_str(),
        Permission::ShopUpdatePolicies.as_str(),
        Permission::ShopViewStats.as_str(),
        // Category
        Permission::CategoryCreate.as_str(),
        Permission::CategoryRead.as_str(),
        Permission::CategoryUpdate.as_str(),
        Permission::CategoryDelete.as_str(),
        Permission::CategoryList.as_str(),
        Permission::CategoryReorder.as_str(),
        Permission::CategoryTree.as_str(),
        // Admin
        Permission::AdminFull.as_str(),
        Permission::AdminRead.as_str(),
        Permission::AdminWrite.as_str(),
        Permission::AdminSystemConfig.as_str(),
        Permission::AdminAuditLogs.as_str(),
    ]
}

/// Get all available permissions as HashSet for efficient lookup
pub fn all_permissions_set() -> HashSet<&'static str> {
    all_permissions().into_iter().collect()
}

/// Validate if a permission string is valid
///
/// # Arguments
/// * `permission` - Permission string to validate
///
/// # Returns
/// * `bool` - true if valid, false otherwise
pub fn is_valid_permission(permission: &str) -> bool {
    Permission::from_str(permission).is_some()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_permission_as_str() {
        assert_eq!(Permission::ProductCreateOwn.as_str(), "product:create:own");
        assert_eq!(Permission::OrderReadAll.as_str(), "order:read:all");
        assert_eq!(
            Permission::WalletWithdrawOwn.as_str(),
            "wallet:withdraw:own"
        );
        assert_eq!(
            Permission::WalletEscrowCreate.as_str(),
            "wallet:escrow:create"
        );
        assert_eq!(
            Permission::DisputeResolveRefund.as_str(),
            "dispute:resolve:refund"
        );
    }

    #[test]
    fn test_permission_from_str() {
        assert_eq!(
            Permission::from_str("product:create:own"),
            Some(Permission::ProductCreateOwn)
        );
        assert_eq!(
            Permission::from_str("wallet:escrow:release"),
            Some(Permission::WalletEscrowRelease)
        );
        assert_eq!(
            Permission::from_str("dispute:respond:seller"),
            Some(Permission::DisputeRespondSeller)
        );
        assert_eq!(Permission::from_str("invalid:permission"), None);
        assert_eq!(Permission::from_str("not_valid"), None);
    }

    #[test]
    fn test_permission_resource_action_scope() {
        assert_eq!(Permission::ProductCreateOwn.resource(), "product");
        assert_eq!(Permission::ProductCreateOwn.action(), "create");
        assert_eq!(Permission::ProductCreateOwn.scope(), Some("own"));

        assert_eq!(Permission::WalletEscrowCreate.resource(), "wallet");
        assert_eq!(Permission::WalletEscrowCreate.action(), "escrow");
        assert_eq!(Permission::WalletEscrowCreate.scope(), Some("create"));

        assert_eq!(Permission::OrderCancel.resource(), "order");
        assert_eq!(Permission::OrderCancel.action(), "cancel");
        assert_eq!(Permission::OrderCancel.scope(), None);
    }

    #[test]
    fn test_all_permissions() {
        let perms = all_permissions();
        assert!(perms.contains(&"product:create:own"));
        assert!(perms.contains(&"order:read:all"));
        assert!(perms.contains(&"wallet:withdraw:own"));
        assert!(perms.contains(&"wallet:escrow:create"));
        assert!(perms.contains(&"dispute:resolve:refund"));
        assert!(perms.contains(&"shop:verify:telegram"));
        assert!(perms.contains(&"admin:full"));
    }

    #[test]
    fn test_all_permissions_count() {
        let perms = all_permissions();
        assert_eq!(perms.len(), 113);
    }

    #[test]
    fn test_is_valid_permission() {
        assert!(is_valid_permission("product:create:own"));
        assert!(is_valid_permission("wallet:escrow:release"));
        assert!(is_valid_permission("role:assign_permissions"));
        assert!(is_valid_permission("admin:system:config"));
        assert!(!is_valid_permission("invalid:permission"));
        assert!(!is_valid_permission("product:create"));
        assert!(!is_valid_permission("not_valid"));
    }

    #[test]
    fn test_wallet_permissions_granularity() {
        assert!(is_valid_permission("wallet:read:own"));
        assert!(is_valid_permission("wallet:read:all"));
        assert!(is_valid_permission("wallet:deposit:own"));
        assert!(is_valid_permission("wallet:deposit:manual"));
        assert!(is_valid_permission("wallet:withdraw:own"));
        assert!(is_valid_permission("wallet:withdraw:approve"));
        assert!(is_valid_permission("wallet:escrow:create"));
        assert!(is_valid_permission("wallet:escrow:release"));
        assert!(is_valid_permission("wallet:freeze"));
        assert!(is_valid_permission("wallet:debit:admin"));
    }

    #[test]
    fn test_dispute_permissions_granularity() {
        assert!(is_valid_permission("dispute:create:own"));
        assert!(is_valid_permission("dispute:read:own"));
        assert!(is_valid_permission("dispute:read:all"));
        assert!(is_valid_permission("dispute:respond:seller"));
        assert!(is_valid_permission("dispute:respond:buyer"));
        assert!(is_valid_permission("dispute:resolve:refund"));
        assert!(is_valid_permission("dispute:resolve:release"));
        assert!(is_valid_permission("dispute:partial_refund"));
    }

    #[test]
    fn test_shop_permissions_granularity() {
        assert!(is_valid_permission("shop:create:own"));
        assert!(is_valid_permission("shop:read:own"));
        assert!(is_valid_permission("shop:read:all"));
        assert!(is_valid_permission("shop:verify:telegram"));
        assert!(is_valid_permission("shop:upload:logo"));
        assert!(is_valid_permission("shop:upload:banner"));
        assert!(is_valid_permission("shop:update:policies"));
        assert!(is_valid_permission("shop:view:stats"));
    }
}
