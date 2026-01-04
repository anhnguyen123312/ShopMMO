-- check_permission.lua
-- Atomic permission check with version validation
--
-- KEYS[1] = user:{user_id}:permissions (SET)
-- KEYS[2] = user:{user_id}:perm_version (STRING)
-- ARGV[1] = required_permission (STRING)
-- ARGV[2] = jwt_perm_version (NUMBER)
--
-- Returns:
--   1 = authorized
--   0 = denied
--  -1 = cache miss or stale (needs refresh)

local cached_version = redis.call('GET', KEYS[2])

if not cached_version then
    return -1  -- Cache miss
end

if tonumber(cached_version) ~= tonumber(ARGV[2]) then
    return -1  -- Version mismatch, need refresh
end

return redis.call('SISMEMBER', KEYS[1], ARGV[1])
