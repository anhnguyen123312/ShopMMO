# Auth Module Context

## Status: ✅ Implemented (Reference Implementation)

## Files
```
src/modules/auth/
├── mod.rs
├── domain.rs      # User, RefreshToken models
├── dto.rs         # RegisterReq, LoginReq, TokenRes
├── handler.rs     # register, login, refresh, me, logout
├── service.rs     # AuthService
├── repository.rs  # UserRepo, RefreshTokenRepo
└── routes.rs
```

## Data Models
```rust
// domain.rs
User {
    id: ObjectId,
    email: String,           // unique, indexed
    password_hash: String,
    name: String,
    role: UserRole,          // Buyer | Vendor | Reseller | Admin
    is_active: bool,
    two_factor_enabled: bool,
    two_factor_secret: Option<String>,
    created_at: DateTime,
    updated_at: DateTime,
}
```

## Endpoints
| Method | Path | Handler | Auth |
|--------|------|---------|------|
| POST | /api/auth/register | register | - |
| POST | /api/auth/login | login | - |
| POST | /api/auth/refresh | refresh | - |
| GET | /api/auth/me | me | Bearer |
| POST | /api/auth/logout | logout | Bearer |

## Refs
- V1 Flow: [docs/v1/01-authentication.md](../../docs/v1/01-authentication.md)
- V1 Roles: [docs/v1/02-user-roles.md](../../docs/v1/02-user-roles.md)

## TODO V2
- [ ] 2FA TOTP verification
- [ ] Email verification flow
- [ ] Password reset
- [ ] Login rate limiting

## Notes
- JWT Access Token: 15min
- Refresh Token: 7 days, stored in MongoDB
- Password: bcrypt with default cost
