// MongoDB Migration Script: Initialize Authorization System V2
//
// This script sets up the initial permissions and roles for the authorization system.
//
// Usage:
//   mongo <connection_string> migrations/001_init_permissions.js
//
// Or via mongosh:
//   mongosh <connection_string> migrations/001_init_permissions.js

// Switch to the application database
use mmo_api;

print("========================================");
print("MMO API - Authorization System V2 Setup");
print("========================================");

// ========================================================================
// 1. CREATE PERMISSIONS COLLECTION
// ========================================================================

print("\n1. Creating permissions...");

const permissions = [
  // User Management Permissions
  {
    name: "users:read",
    display_name: "Read Users",
    description: "View user information",
    resource: "users",
    action: "read",
    category: "user_management",
    is_active: true,
    created_at: new Date(),
    updated_at: new Date()
  },
  {
    name: "users:create",
    display_name: "Create Users",
    description: "Create new users",
    resource: "users",
    action: "create",
    category: "user_management",
    is_active: true,
    created_at: new Date(),
    updated_at: new Date()
  },
  {
    name: "users:update",
    display_name: "Update Users",
    description: "Update user information",
    resource: "users",
    action: "update",
    category: "user_management",
    is_active: true,
    created_at: new Date(),
    updated_at: new Date()
  },
  {
    name: "users:delete",
    display_name: "Delete Users",
    description: "Delete users",
    resource: "users",
    action: "delete",
    category: "user_management",
    is_active: true,
    created_at: new Date(),
    updated_at: new Date()
  },

  // Product Management Permissions
  {
    name: "products:read",
    display_name: "Read Products",
    description: "View products",
    resource: "products",
    action: "read",
    category: "product_management",
    is_active: true,
    created_at: new Date(),
    updated_at: new Date()
  },
  {
    name: "products:create",
    display_name: "Create Products",
    description: "Create new products",
    resource: "products",
    action: "create",
    category: "product_management",
    is_active: true,
    created_at: new Date(),
    updated_at: new Date()
  },
  {
    name: "products:update",
    display_name: "Update Products",
    description: "Update product information",
    resource: "products",
    action: "update",
    category: "product_management",
    is_active: true,
    created_at: new Date(),
    updated_at: new Date()
  },
  {
    name: "products:delete",
    display_name: "Delete Products",
    description: "Delete products",
    resource: "products",
    action: "delete",
    category: "product_management",
    is_active: true,
    created_at: new Date(),
    updated_at: new Date()
  },

  // Order Management Permissions
  {
    name: "orders:read",
    display_name: "Read Orders",
    description: "View orders",
    resource: "orders",
    action: "read",
    category: "order_management",
    is_active: true,
    created_at: new Date(),
    updated_at: new Date()
  },
  {
    name: "orders:create",
    display_name: "Create Orders",
    description: "Create new orders",
    resource: "orders",
    action: "create",
    category: "order_management",
    is_active: true,
    created_at: new Date(),
    updated_at: new Date()
  },
  {
    name: "orders:update",
    display_name: "Update Orders",
    description: "Update order status",
    resource: "orders",
    action: "update",
    category: "order_management",
    is_active: true,
    created_at: new Date(),
    updated_at: new Date()
  },
  {
    name: "orders:delete",
    display_name: "Delete Orders",
    description: "Cancel/delete orders",
    resource: "orders",
    action: "delete",
    category: "order_management",
    is_active: true,
    created_at: new Date(),
    updated_at: new Date()
  },

  // Wallet Management Permissions
  {
    name: "wallets:read",
    display_name: "Read Wallets",
    description: "View wallet information",
    resource: "wallets",
    action: "read",
    category: "wallet_management",
    is_active: true,
    created_at: new Date(),
    updated_at: new Date()
  },
  {
    name: "wallets:manage",
    display_name: "Manage Wallets",
    description: "Manage wallet operations",
    resource: "wallets",
    action: "manage",
    category: "wallet_management",
    is_active: true,
    created_at: new Date(),
    updated_at: new Date()
  },

  // Admin Permissions
  {
    name: "admin:users",
    display_name: "Admin Users",
    description: "Full administrative access to user management",
    resource: "admin",
    action: "users",
    category: "administration",
    is_active: true,
    created_at: new Date(),
    updated_at: new Date()
  },
  {
    name: "admin:system",
    display_name: "Admin System",
    description: "Full system administration access",
    resource: "admin",
    action: "system",
    category: "administration",
    is_active: true,
    created_at: new Date(),
    updated_at: new Date()
  },
  {
    name: "admin:logs",
    display_name: "Admin Logs",
    description: "View system logs and audits",
    resource: "admin",
    action: "logs",
    category: "administration",
    is_active: true,
    created_at: new Date(),
    updated_at: new Date()
  }
];

// Insert permissions
db.permissions.insertMany(permissions);
print(`   Inserted ${permissions.length} permissions`);

// ========================================================================
// 2. CREATE ROLES COLLECTION
// ========================================================================

print("\n2. Creating roles...");

// First, get all permission IDs
const buyerPerms = db.permissions.find({ category: { $in: ["product_management", "order_management", "wallet_management"] } }, { _id: 1 }).toArray();
const sellerPerms = db.permissions.find({ category: { $in: ["product_management", "order_management", "wallet_management"] } }, { _id: 1 }).toArray();
const adminPerms = db.permissions.find({}, { _id: 1 }).toArray();

const roles = [
  {
    name: "BUYER",
    display_name: "Buyer",
    description: "Regular buyer role - can browse products and place orders",
    level: 0,
    inherits_from: [],
    direct_permissions: buyerPerms.map(p => p._id),
    flattened_permissions: buyerPerms.map(p => p.name),
    is_system: true,
    is_active: true,
    version: 1,
    created_at: new Date(),
    updated_at: new Date()
  },
  {
    name: "SELLER",
    display_name: "Seller",
    description: "Seller role - can manage products and orders",
    level: 1,
    inherits_from: ["BUYER"],
    direct_permissions: sellerPerms.map(p => p._id),
    flattened_permissions: sellerPerms.map(p => p.name),
    is_system: true,
    is_active: true,
    version: 1,
    created_at: new Date(),
    updated_at: new Date()
  },
  {
    name: "ADMIN",
    display_name: "Administrator",
    description: "Administrator role - full access except user management",
    level: 2,
    inherits_from: ["SELLER"],
    direct_permissions: adminPerms.map(p => p._id),
    flattened_permissions: adminPerms.filter(p => p.category !== 'administration' || p.action !== 'system').map(p => p.name),
    is_system: true,
    is_active: true,
    version: 1,
    created_at: new Date(),
    updated_at: new Date()
  },
  {
    name: "SUPER_ADMIN",
    display_name: "Super Administrator",
    description: "Super administrator role - full system access",
    level: 3,
    inherits_from: [],
    direct_permissions: adminPerms.map(p => p._id),
    flattened_permissions: adminPerms.map(p => p.name),
    is_system: true,
    is_active: true,
    version: 1,
    created_at: new Date(),
    updated_at: new Date()
  }
];

// Insert roles
db.roles.insertMany(roles);
print(`   Inserted ${roles.length} roles`);

// ========================================================================
// 3. CREATE INDEXES
// ========================================================================

print("\n3. Creating indexes...");

// Permissions indexes
db.permissions.createIndex({ name: 1 }, { unique: true });
db.permissions.createIndex({ resource: 1, action: 1 });
db.permissions.createIndex({ category: 1 });
db.permissions.createIndex({ is_active: 1 });
print("   Permissions indexes created");

// Roles indexes
db.roles.createIndex({ name: 1 }, { unique: true });
db.roles.createIndex({ level: 1 });
db.roles.createIndex({ is_active: 1 });
db.roles.createIndex({ is_system: 1 });
print("   Roles indexes created");

// ========================================================================
// 4. CREATE ROLE_ASSIGNMENTS COLLECTION (for future use)
// ========================================================================

print("\n4. Creating role_assignments collection...");

db.createCollection("role_assignments");
db.role_assignments.createIndex({ user_id: 1 });
db.role_assignments.createIndex({ role_name: 1 });
db.role_assignments.createIndex({ user_id: 1, role_name: 1 }, { unique: true });
print("   role_assignments collection created");

// ========================================================================
// SUMMARY
// ========================================================================

print("\n========================================");
print("Migration completed successfully!");
print("========================================");
print(`Permissions: ${db.permissions.countDocuments({})}`);
print(`Roles: ${db.roles.countDocuments({})}`);
print("\nRole hierarchy:");
print("  SUPER_ADMIN (level 3)");
print("    └─ ADMIN (level 2)");
print("         └─ SELLER (level 1)");
print("              └─ BUYER (level 0)");
print("\nTo add roles to users, use the following pattern:");
print("  db.users.updateOne(");
print("    { _id: ObjectId('user_id') },");
print("    {");
print("      $set: {");
print("        roles: ['BUYER', 'SELLER'],");
print("        perm_version: 1");
print("      }");
print("    }");
print("  );");
print("\n========================================");
